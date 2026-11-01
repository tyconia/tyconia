//! Textured dropdown menu for multiple choice

use super::*;
use crate::loading;
use bevy::ui::GlobalZIndex;

#[derive(Component, Debug)]
#[require(Node)]
pub struct Dropdown {
    pub selected: Option<usize>,
    pub options: Vec<String>,
    pub expanded: bool,
    pub placeholder: String,
}

/// Marks an option row in the expanded list and points back at the owning
/// dropdown button entity.
#[derive(Component)]
pub struct DropdownOption {
    pub index: usize,
    pub owner: Entity,
}

/// Marks the floating list spawned beneath a dropdown button.
#[derive(Component)]
pub struct DropdownList {
    pub owner: Entity,
}

pub struct DropdownPlugin;

impl Plugin for DropdownPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                dropdown_toggle,
                dropdown_select_option,
                dropdown_keyboard_nav,
                dropdown_close_on_outside_click,
                dropdown_cleanup_orphans,
            )
                .chain()
                .run_if(resource_exists::<loading::UiAssets>)
                .run_if(resource_exists::<loading::FontAssets>),
        );
    }
}

pub fn spawn_dropdown(
    parent: &mut ChildBuilder,
    ui: &Res<loading::UiAssets>,
    fonts: &Res<loading::FontAssets>,
    options: &[String],
    placeholder: String,
    selected: Option<usize>,
    components: impl Bundle,
) {
    let display_text = selected
        .and_then(|i| options.get(i).cloned())
        .unwrap_or_else(|| placeholder.clone());

    spawn_button(
        ButtonType::LabeledIconReversed {
            text: display_text,
            icon: ui.dropdown_ico.clone(),
            font_size: UI_SCALE * 2.,
            image_size: Val::Px(UI_SCALE * 2.),
        },
        (
            Dropdown {
                selected,
                options: options.to_vec(),
                expanded: false,
                placeholder,
            },
            components,
        ),
        parent,
        fonts,
        ui,
    );
}

fn dropdown_toggle(
    mut commands: Commands,
    mut opened: Query<
        (
            Entity,
            &Interaction,
            &mut Dropdown,
            &GlobalTransform,
            &ComputedNode,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    existing_lists: Query<(Entity, &DropdownList)>,
    ui: Res<loading::UiAssets>,
    fonts: Res<loading::FontAssets>,
) {
    for (entity, interaction, mut dropdown, transform, node) in opened.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        dropdown.expanded = !dropdown.expanded;

        for (list_entity, list) in existing_lists.iter() {
            if list.owner == entity {
                commands.entity(list_entity).despawn_recursive();
            }
        }

        if dropdown.expanded {
            spawn_dropdown_list(
                entity,
                &mut commands,
                &dropdown,
                transform,
                node,
                &ui,
                &fonts,
            );
        }
    }
}

fn spawn_dropdown_list(
    button_entity: Entity,
    commands: &mut Commands,
    dropdown: &Dropdown,
    transform: &GlobalTransform,
    node: &ComputedNode,
    ui: &Res<loading::UiAssets>,
    fonts: &Res<loading::FontAssets>,
) {
    let scale = node.inverse_scale_factor();
    let size = node.size() * scale;
    let center = transform.translation().truncate() * scale;
    let left = center.x - size.x * 0.5;
    let top = center.y + size.y * 0.5;

    commands
        .spawn((
            DropdownList {
                owner: button_entity,
            },
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(top),
                left: Val::Px(left),
                width: Val::Px(size.x.max(120.0)),
                flex_direction: FlexDirection::Column,
                row_gap: spacing_xs(),
                padding: Spacing::Xs.rect_all(),
                max_height: Val::Px(DROPDOWN_MAX_HEIGHT),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.98)),
            GlobalZIndex(ZIndices::Tooltip as i32 + 1),
            ImageNode {
                image: ui.button_alpha.clone(),
                image_mode: BUTTON_IMG_MODE_SLICED,
                ..default()
            },
        ))
        .with_children(|parent| {
            for (index, option) in dropdown.options.iter().enumerate() {
                let is_selected = dropdown.selected == Some(index);
                parent
                    .spawn((
                        Button,
                        Interaction::default(),
                        DropdownOption {
                            index,
                            owner: button_entity,
                        },
                        Node {
                            width: Val::Percent(100.0),
                            padding: UiRect::axes(spacing_sm(), spacing_xs()),
                            ..default()
                        },
                        BackgroundColor(if is_selected {
                            Color::srgba(0.3, 0.4, 0.6, 1.0)
                        } else {
                            Color::NONE
                        }),
                    ))
                    .with_children(|parent| {
                        body_text(option, parent, fonts);
                    });
            }
        });
}

fn dropdown_select_option(
    mut commands: Commands,
    query: Query<(&Interaction, &DropdownOption), (Changed<Interaction>, With<Button>)>,
    mut dropdown_query: Query<(&mut Dropdown, Option<&Children>)>,
    lists: Query<(Entity, &DropdownList)>,
    text_query: Query<(), With<Text>>,
) {
    for (interaction, option) in query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Ok((mut dropdown, children)) = dropdown_query.get_mut(option.owner) else {
            continue;
        };

        dropdown.selected = Some(option.index);
        dropdown.expanded = false;

        if let Some(selected_text) = dropdown.options.get(option.index).cloned() {
            refresh_dropdown_label(
                &mut commands,
                option.owner,
                &selected_text,
                children,
                &text_query,
            );
        }

        for (list_entity, list) in lists.iter() {
            if list.owner == option.owner {
                commands.entity(list_entity).despawn_recursive();
            }
        }
    }
}

fn dropdown_keyboard_nav(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut dropdowns: Query<(Entity, &mut Dropdown, &GlobalTransform, &ComputedNode)>,
    lists: Query<(Entity, &DropdownList)>,
    children_query: Query<&Children>,
    text_query: Query<(), With<Text>>,
    mut focus: ResMut<FocusState>,
    ui: Res<loading::UiAssets>,
    fonts: Res<loading::FontAssets>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        for (entity, mut dropdown, _, _) in dropdowns.iter_mut() {
            if dropdown.expanded {
                dropdown.expanded = false;
                focus.escape_consumed = true;
                remove_owner_lists(entity, &lists, &mut commands);
            }
        }
        return;
    }

    for (entity, mut dropdown, transform, node) in dropdowns.iter_mut() {
        if !dropdown.expanded {
            if focus.current == Some(entity)
                && (keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space))
            {
                dropdown.expanded = true;
                spawn_dropdown_list(
                    entity,
                    &mut commands,
                    &dropdown,
                    transform,
                    node,
                    &ui,
                    &fonts,
                );
            }
            continue;
        }

        focus.current = Some(entity);

        if keys.just_pressed(KeyCode::ArrowDown) {
            dropdown.selected = match dropdown.selected {
                Some(i) if i + 1 < dropdown.options.len() => Some(i + 1),
                None if !dropdown.options.is_empty() => Some(0),
                _ => dropdown.selected,
            };
        }

        if keys.just_pressed(KeyCode::ArrowUp) {
            dropdown.selected = match dropdown.selected {
                Some(i) if i > 0 => Some(i - 1),
                None if !dropdown.options.is_empty() => Some(dropdown.options.len() - 1),
                _ => dropdown.selected,
            };
        }

        if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
            dropdown.expanded = false;
            remove_owner_lists(entity, &lists, &mut commands);

            if let Some(index) = dropdown.selected {
                if let Some(selected_text) = dropdown.options.get(index).cloned() {
                    let children = children_query.get(entity).ok();
                    refresh_dropdown_label(
                        &mut commands,
                        entity,
                        &selected_text,
                        children,
                        &text_query,
                    );
                }
            }
        }
    }
}

fn dropdown_close_on_outside_click(
    mut presses: EventReader<Pointer<Down>>,
    mut query: Query<(Entity, &mut Dropdown)>,
    dropdowns: Query<(), With<Dropdown>>,
    lists: Query<(Entity, &DropdownList)>,
    parents: Query<&Parent>,
    mut commands: Commands,
) {
    let mut clicked_outside = false;
    let mut keep_owner = None;
    for press in presses.read() {
        if press.button != PointerButton::Primary {
            continue;
        }

        let mut entity = Some(press.target);
        while let Some(candidate) = entity {
            if dropdowns.contains(candidate) {
                keep_owner = Some(candidate);
                break;
            }
            if let Some((_, list)) = lists
                .iter()
                .find(|(list_entity, _)| *list_entity == candidate)
            {
                keep_owner = Some(list.owner);
                break;
            }
            entity = parents.get(candidate).ok().map(|parent| parent.get());
        }

        if keep_owner.is_none() {
            clicked_outside = true;
            break;
        }
    }

    if !clicked_outside && keep_owner.is_none() {
        return;
    }

    for (entity, mut dropdown) in query.iter_mut() {
        if !dropdown.expanded || keep_owner == Some(entity) {
            continue;
        }
        dropdown.expanded = false;
        remove_owner_lists(entity, &lists, &mut commands);
    }
}

fn dropdown_cleanup_orphans(
    mut commands: Commands,
    lists: Query<(Entity, &DropdownList)>,
    dropdowns: Query<(), With<Dropdown>>,
    parents: Query<&InheritedVisibility>,
) {
    for (list_entity, list) in lists.iter() {
        let owner_exists = dropdowns.get(list.owner).is_ok();
        let owner_visible = parents
            .get(list.owner)
            .map(|visibility| visibility.get())
            .unwrap_or(true);
        if !owner_exists || !owner_visible {
            commands.entity(list_entity).despawn_recursive();
        }
    }
}

fn refresh_dropdown_label(
    commands: &mut Commands,
    owner: Entity,
    text: &str,
    children: Option<&Children>,
    text_query: &Query<(), With<Text>>,
) {
    let Some(children) = children else {
        return;
    };
    for child in children.iter() {
        if text_query.get(*child).is_ok() {
            commands.entity(*child).insert(Text::new(text));
            break;
        }
    }
    let _ = owner;
}

fn remove_owner_lists(
    owner: Entity,
    lists: &Query<(Entity, &DropdownList)>,
    commands: &mut Commands,
) {
    for (list_entity, list) in lists.iter() {
        if list.owner == owner {
            commands.entity(list_entity).despawn_recursive();
        }
    }
}
