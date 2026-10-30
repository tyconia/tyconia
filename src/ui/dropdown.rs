//! Textured dropdown menu for multiple choice

use super::*;
use crate::loading;

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
            )
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
        Dropdown {
            selected,
            options: options.to_vec(),
            expanded: false,
            placeholder,
        },
        parent,
        fonts,
        ui,
    );
}

fn dropdown_toggle(
    mut commands: Commands,
    mut query: Query<(Entity, &Interaction, &mut Dropdown), (Changed<Interaction>, With<Button>)>,
    existing_lists: Query<(Entity, &DropdownList)>,
    ui: Res<loading::UiAssets>,
    fonts: Res<loading::FontAssets>,
) {
    for (entity, interaction, mut dropdown) in query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        dropdown.expanded = !dropdown.expanded;

        // Remove any list that already belongs to this dropdown.
        for (list_entity, list) in existing_lists.iter() {
            if list.owner == entity {
                commands.entity(list_entity).despawn_recursive();
            }
        }

        if dropdown.expanded {
            spawn_dropdown_list(entity, &mut commands, &dropdown, &ui, &fonts);
        }
    }
}

fn spawn_dropdown_list(
    button_entity: Entity,
    commands: &mut Commands,
    dropdown: &Dropdown,
    ui: &Res<loading::UiAssets>,
    fonts: &Res<loading::FontAssets>,
) {
    commands.entity(button_entity).with_children(|parent| {
        parent
            .spawn((
                DropdownList {
                    owner: button_entity,
                },
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(100.0),
                    left: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: spacing_xs(),
                    padding: Spacing::Xs.rect_all(),
                    max_height: Val::Px(DROPDOWN_MAX_HEIGHT),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.98)),
                ZIndex::from(ZIndices::Tooltip),
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
    });
}

fn dropdown_select_option(
    mut commands: Commands,
    query: Query<
        (&Interaction, &DropdownOption),
        (Changed<Interaction>, With<Button>),
    >,
    mut dropdown_query: Query<&mut Dropdown>,
    lists: Query<(Entity, &DropdownList)>,
    children_query: Query<&Children>,
    text_query: Query<(), With<Text>>,
) {
    for (interaction, option) in query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Ok(mut dropdown) = dropdown_query.get_mut(option.owner) else {
            continue;
        };

        dropdown.selected = Some(option.index);
        dropdown.expanded = false;

        if let Some(selected_text) = dropdown.options.get(option.index).cloned() {
            // Update the label text of the owning button without touching the
            // icon child.
            if let Ok(children) = children_query.get(option.owner) {
                for child in children.iter() {
                    if text_query.get(*child).is_ok() {
                        commands.entity(*child).insert(Text::new(selected_text.clone()));
                        break;
                    }
                }
            }
        }

        // Collapse the owning list.
        for (list_entity, list) in lists.iter() {
            if list.owner == option.owner {
                commands.entity(list_entity).despawn_recursive();
            }
        }
    }
}

fn dropdown_keyboard_nav(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(Entity, &mut Dropdown), With<Dropdown>>,
    mut commands: Commands,
    lists: Query<(Entity, &DropdownList)>,
) {
    let mut close_requested = false;

    for (entity, mut dropdown) in query.iter_mut() {
        if !dropdown.expanded {
            continue;
        }

        if keys.just_pressed(KeyCode::Escape) {
            dropdown.expanded = false;
            close_requested = true;
        }

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

        if keys.just_pressed(KeyCode::Enter) {
            dropdown.expanded = false;
            close_requested = true;
        }

        if close_requested {
            for (list_entity, list) in lists.iter() {
                if list.owner == entity {
                    commands.entity(list_entity).despawn_recursive();
                }
            }
        }
    }
}

fn dropdown_close_on_outside_click(
    mouse: Res<ButtonInput<MouseButton>>,
    mut query: Query<(Entity, &mut Dropdown), With<Dropdown>>,
    lists: Query<(Entity, &DropdownList)>,
    pressed: Query<
        &Interaction,
        (
            With<Button>,
            Without<Dropdown>,
            Without<DropdownOption>,
            Changed<Interaction>,
        ),
    >,
    mut commands: Commands,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let clicked_outside = pressed.iter().any(|i| *i == Interaction::Pressed);
    if !clicked_outside {
        return;
    }

    for (entity, mut dropdown) in query.iter_mut() {
        if !dropdown.expanded {
            continue;
        }
        dropdown.expanded = false;

        for (list_entity, list) in lists.iter() {
            if list.owner == entity {
                commands.entity(list_entity).despawn_recursive();
            }
        }
    }
}
