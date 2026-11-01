use bevy::prelude::*;
use bevy::ui::UiSystem;

use crate::settings::SettingsConfig;

use super::{DepressButton, Dropdown, ScrollPosition, Slider};

pub struct FocusPlugin;

#[derive(Component, Default, Debug)]
pub struct Focusable;

#[derive(Component, Debug)]
pub struct Disabled;

#[derive(Component, Debug)]
struct KeyboardActivation;

#[derive(Resource, Default, Debug)]
pub struct FocusState {
    pub current: Option<Entity>,
    pub suspended: bool,
    pub escape_consumed: bool,
}

impl Plugin for FocusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FocusState>()
            .add_observer(register_focusable)
            .add_systems(First, clear_escape_consumed)
            .add_systems(
                Update,
                (
                    pointer_focus,
                    keyboard_navigation,
                    activate_focused,
                    sync_focus,
                    update_focus_ring,
                )
                    .chain(),
            )
            .add_systems(
                PostUpdate,
                (reset_keyboard_activation, scroll_focused_into_view)
                    .chain()
                    .after(UiSystem::Layout),
            );
    }
}

fn clear_escape_consumed(mut focus: ResMut<FocusState>) {
    focus.escape_consumed = false;
}

fn register_focusable(trigger: Trigger<OnAdd, Button>, mut commands: Commands) {
    commands.entity(trigger.entity()).insert((
        Focusable,
        Outline::new(Val::Px(2.0), Val::Px(2.0), Color::NONE),
    ));
}

fn pointer_focus(
    interactions: Query<
        (Entity, &Interaction, &Focusable, Option<&Disabled>),
        Changed<Interaction>,
    >,
    mut focus: ResMut<FocusState>,
) {
    for (entity, interaction, _, disabled) in interactions.iter() {
        if disabled.is_some() {
            continue;
        }
        if matches!(*interaction, Interaction::Hovered | Interaction::Pressed) {
            focus.current = Some(entity);
        }
    }
}

fn keyboard_navigation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut focus: ResMut<FocusState>,
    focusables: Query<
        (
            Entity,
            &GlobalTransform,
            &ComputedNode,
            Option<&InheritedVisibility>,
        ),
        (With<Focusable>, Without<Disabled>),
    >,
    dropdowns: Query<&Dropdown>,
    mut sliders: Query<&mut Slider>,
) {
    if focus.suspended {
        return;
    }

    let dropdown_open = dropdowns.iter().any(|dropdown| dropdown.expanded);
    let tab_pressed = keyboard.just_pressed(KeyCode::Tab);
    let direction = if !dropdown_open {
        if keyboard.just_pressed(KeyCode::ArrowDown) {
            Some(Vec2::Y)
        } else if keyboard.just_pressed(KeyCode::ArrowUp) {
            Some(Vec2::NEG_Y)
        } else if keyboard.just_pressed(KeyCode::ArrowRight) {
            Some(Vec2::X)
        } else if keyboard.just_pressed(KeyCode::ArrowLeft) {
            Some(Vec2::NEG_X)
        } else {
            None
        }
    } else {
        None
    };

    if !tab_pressed && direction.is_none() {
        return;
    }

    let mut candidates: Vec<(Entity, Vec2, Vec2)> = focusables
        .iter()
        .filter(|(_, _, _, visibility)| {
            visibility
                .map(|visibility| visibility.get())
                .unwrap_or(true)
        })
        .map(|(entity, transform, computed, _)| {
            let scale = computed.inverse_scale_factor();
            let center = transform.translation().truncate() * scale;
            let size = computed.size() * scale;
            (entity, center, size)
        })
        .collect();

    if candidates.is_empty() {
        focus.current = None;
        return;
    }

    candidates.sort_by(|a, b| {
        a.1.y
            .total_cmp(&b.1.y)
            .then_with(|| a.1.x.total_cmp(&b.1.x))
    });

    if tab_pressed {
        let current_index = focus.current.and_then(|current| {
            candidates
                .iter()
                .position(|(entity, _, _)| *entity == current)
        });
        let next = match current_index {
            Some(index)
                if keyboard.pressed(KeyCode::ShiftLeft)
                    || keyboard.pressed(KeyCode::ShiftRight) =>
            {
                if index == 0 {
                    candidates.len() - 1
                } else {
                    index - 1
                }
            }
            Some(index) => (index + 1) % candidates.len(),
            None => 0,
        };
        focus.current = Some(candidates[next].0);
        return;
    }

    let Some(direction) = direction else {
        return;
    };

    if let Some(current) = focus.current {
        if let Ok(mut slider) = sliders.get_mut(current) {
            let increment = if slider.steps > 1 {
                100.0 / (slider.steps - 1) as f32
            } else {
                1.0
            };
            let delta = if direction.x > 0.0 || direction.y > 0.0 {
                increment
            } else {
                -increment
            };
            let percentage = slider.percentage + delta;
            slider.set_percentage(percentage);
            return;
        }
    }

    let Some(current) = focus.current else {
        focus.current = Some(candidates[0].0);
        return;
    };

    let Some((_, current_center, _)) = candidates.iter().find(|(entity, _, _)| *entity == current)
    else {
        focus.current = Some(candidates[0].0);
        return;
    };

    let next = candidates
        .iter()
        .filter(|(entity, _, _)| *entity != current)
        .filter_map(|(entity, center, size)| {
            let delta = *center - *current_center;
            let forward = delta.dot(direction);
            if forward <= 1.0 {
                return None;
            }
            let perpendicular = delta.dot(Vec2::new(-direction.y, direction.x)).abs();
            let score = forward + perpendicular * 2.0 + size.length() * 0.05;
            Some((*entity, score))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(entity, _)| entity);

    if let Some(next) = next {
        focus.current = Some(next);
    }
}

fn activate_focused(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    focus: Res<FocusState>,
    mut targets: Query<(
        Option<&mut DepressButton>,
        Option<&mut Interaction>,
        Option<&Disabled>,
    )>,
    dropdowns: Query<&Dropdown>,
) {
    if focus.suspended
        || !(keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space))
    {
        return;
    }

    let Some(entity) = focus.current else {
        return;
    };

    if dropdowns.iter().any(|dropdown| dropdown.expanded) || dropdowns.get(entity).is_ok() {
        return;
    }

    let Ok((depress, interaction, disabled)) = targets.get_mut(entity) else {
        return;
    };

    if disabled.is_some() {
        return;
    }

    if let Some(mut depress) = depress {
        depress.invoked = true;
        depress.pressed = false;
    }

    if let Some(mut interaction) = interaction {
        *interaction = Interaction::Pressed;
    }

    commands.entity(entity).insert(KeyboardActivation);
}

fn reset_keyboard_activation(
    mut commands: Commands,
    mut activated: Query<
        (Entity, Option<&mut DepressButton>, Option<&mut Interaction>),
        With<KeyboardActivation>,
    >,
) {
    for (entity, depress, interaction) in activated.iter_mut() {
        if let Some(mut depress) = depress {
            depress.invoked = false;
            depress.pressed = false;
        }
        if let Some(mut interaction) = interaction {
            if *interaction == Interaction::Pressed {
                *interaction = Interaction::None;
            }
        }
        commands.entity(entity).remove::<KeyboardActivation>();
    }
}

fn sync_focus(
    mut focus: ResMut<FocusState>,
    active: Query<(), (With<Focusable>, Without<Disabled>)>,
) {
    if focus
        .current
        .is_some_and(|current| active.get(current).is_err())
    {
        focus.current = None;
    }
}

fn update_focus_ring(
    focus: Res<FocusState>,
    settings: Option<Res<SettingsConfig>>,
    mut focusables: Query<(Entity, &Focusable, &mut Outline, Option<&Disabled>)>,
) {
    let high_contrast = settings
        .map(|settings| settings.interface.high_contrast)
        .unwrap_or(false);
    let color = if high_contrast {
        Color::WHITE
    } else {
        Color::srgba(1.0, 0.82, 0.18, 0.95)
    };

    for (entity, _, mut outline, disabled) in focusables.iter_mut() {
        outline.color = if disabled.is_some() || focus.current != Some(entity) {
            Color::NONE
        } else {
            color
        };
        outline.width = Val::Px(if high_contrast { 3.0 } else { 2.0 });
        outline.offset = Val::Px(2.0);
    }
}

fn scroll_focused_into_view(
    focus: Res<FocusState>,
    nodes: Query<(&GlobalTransform, &ComputedNode)>,
    parents: Query<&Parent>,
    mut scrollables: Query<(&mut ScrollPosition, &ComputedNode, &GlobalTransform)>,
) {
    let Some(current) = focus.current else {
        return;
    };
    let Ok((focus_transform, focus_node)) = nodes.get(current) else {
        return;
    };

    let focus_scale = focus_node.inverse_scale_factor();
    let focus_center = focus_transform.translation().truncate() * focus_scale;
    let focus_size = focus_node.size() * focus_scale;

    let mut ancestor = parents.get(current).ok().map(|parent| parent.get());
    while let Some(entity) = ancestor {
        if let Ok((mut scroll, node, transform)) = scrollables.get_mut(entity) {
            let scale = node.inverse_scale_factor();
            let container_position = transform.translation().truncate() * scale;
            let container_size = node.size() * scale;
            let relative_top =
                focus_center.y - container_position.y + scroll.offset_y - focus_size.y * 0.5;
            let relative_bottom = relative_top + focus_size.y;

            if relative_top < 0.0 {
                scroll.offset_y = (scroll.offset_y + relative_top).max(0.0);
            } else if relative_bottom > container_size.y {
                scroll.offset_y += relative_bottom - container_size.y;
            }
            break;
        }
        ancestor = parents.get(entity).ok().map(|parent| parent.get());
    }
}
