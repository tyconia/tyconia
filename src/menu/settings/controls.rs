use super::SettingsTabsState;
use crate::actions::*;
use crate::loading::*;
use crate::ui::*;

use bevy::prelude::*;

pub fn setup(
    mut cmd: Commands,
    backdrop: super::SettingsBackdropQuery,
    fonts: Res<FontAssets>,
    ui: Res<UiAssets>,

    input_mappings: Res<InputMappings>,
) {
    cmd.entity(backdrop.single()).with_children(|parent| {
        parent
            .spawn((
                StateScoped(SettingsTabsState::Controls),
                Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
            ))
            .with_children(|parent| {
                section_text("Interaction", parent, &fonts);
                input_map_entry(InterAction::Construct, parent, &fonts, &ui, &input_mappings);
                input_map_entry(
                    InterAction::Deconstruct,
                    parent,
                    &fonts,
                    &ui,
                    &input_mappings,
                );
                input_map_entry(
                    InterAction::Distribute,
                    parent,
                    &fonts,
                    &ui,
                    &input_mappings,
                );
                input_map_entry(InterAction::Pipette, parent, &fonts, &ui, &input_mappings);
                input_map_entry(
                    InterAction::CopyConfiguration,
                    parent,
                    &fonts,
                    &ui,
                    &input_mappings,
                );
                input_map_entry(
                    InterAction::PasteConfiguration,
                    parent,
                    &fonts,
                    &ui,
                    &input_mappings,
                );
                separator(parent);

                section_text("Movement", parent, &fonts);
                input_map_entry(MovementAction::North, parent, &fonts, &ui, &input_mappings);
                input_map_entry(MovementAction::West, parent, &fonts, &ui, &input_mappings);
                input_map_entry(MovementAction::South, parent, &fonts, &ui, &input_mappings);
                input_map_entry(MovementAction::East, parent, &fonts, &ui, &input_mappings);
                separator(parent);

                section_text("Interface", parent, &fonts);
                input_map_entry(UiAction::Menu, parent, &fonts, &ui, &input_mappings);
                input_map_entry(UiAction::Zoom(2), parent, &fonts, &ui, &input_mappings);
                input_map_entry(
                    UiAction::HotbarSlotNext,
                    parent,
                    &fonts,
                    &ui,
                    &input_mappings,
                );
                input_map_entry(
                    UiAction::HotbarSlotPrevious,
                    parent,
                    &fonts,
                    &ui,
                    &input_mappings,
                );
                input_map_entry(
                    UiAction::HotbarSlot(0),
                    parent,
                    &fonts,
                    &ui,
                    &input_mappings,
                );
                input_map_entry(
                    UiAction::HotbarSlot(1),
                    parent,
                    &fonts,
                    &ui,
                    &input_mappings,
                );
                input_map_entry(
                    UiAction::HotbarSlot(2),
                    parent,
                    &fonts,
                    &ui,
                    &input_mappings,
                );
                input_map_entry(
                    UiAction::HotbarSlot(3),
                    parent,
                    &fonts,
                    &ui,
                    &input_mappings,
                );
                input_map_entry(
                    UiAction::HotbarSlot(4),
                    parent,
                    &fonts,
                    &ui,
                    &input_mappings,
                );
                input_map_entry(
                    UiAction::HotbarSlot(5),
                    parent,
                    &fonts,
                    &ui,
                    &input_mappings,
                );
                input_map_entry(
                    UiAction::HotbarSlot(6),
                    parent,
                    &fonts,
                    &ui,
                    &input_mappings,
                );

                separator(parent);

                section_text("Custom", parent, &fonts);
                separator(parent);
            });
    });
}

#[derive(Debug, Component, Default)]
#[require(Button)]
pub struct RemapButton;

#[derive(Debug, Component, Default)]
#[require(Button)]
pub struct RemapButtonActive {}

fn input_map_entry<A: InputAction>(
    action: A,
    parent: &mut ChildBuilder,
    fonts: &Res<FontAssets>,
    ui: &Res<UiAssets>,

    input_mappings: &Res<InputMappings>,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.),
            height: Val::Px(UI_SCALE * 5.),
            padding: UiRect::right(Val::Px(UI_SCALE * 1.)),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|parent| {
            body_text(&action.display(), parent, &fonts);

            let (primary, secondary) = {
                action.desktop_mapping(&input_mappings).map_or(
                    ("".to_string(), "".to_string()),
                    |mapping| {
                        (
                            mapping
                                .primary
                                .iter()
                                .map(|d| {
                                    d.to_string()
                                        .replace("Digit", "")
                                        .replace("Key", "")
                                        .replace("Arrow", "")
                                        .replace("Button", "Mouse")
                                })
                                .collect::<Vec<String>>()
                                .join(" + "),
                            mapping
                                .secondary
                                .iter()
                                .map(|d| {
                                    d.to_string()
                                        .replace("Digit", "")
                                        .replace("Key", "")
                                        .replace("Arrow", "")
                                        .replace("Button", "Mouse")
                                })
                                .collect::<Vec<String>>()
                                .join(" + "),
                        )
                    },
                )
            };

            parent
                .spawn(Node {
                    column_gap: Val::Px(UI_SCALE * 6.),
                    ..default()
                })
                .with_children(|parent| {
                    // undo button
                    parent
                        .spawn(Node {
                            height: Val::Px(UI_SCALE / 4.),
                            aspect_ratio: Some(1.0),
                            ..default()
                        })
                        .with_children(|parent| {
                            spawn_button(
                                ButtonType::Icon {
                                    image: Some(ui.undo_ico.clone()),
                                    image_size: Val::Px(16.),
                                },
                                (RemapButton::default(), Visibility::Hidden),
                                parent,
                                &fonts,
                                &ui,
                            );
                        });

                    parent
                        .spawn(Node {
                            width: Val::Px(100.),
                            height: Val::Px(UI_SCALE),
                            ..default()
                        })
                        .with_children(|parent| {
                            spawn_button(
                                ButtonType::Text {
                                    font_size: if primary.len() < 13 {
                                        UI_SCALE * 2.5
                                    } else {
                                        UI_SCALE * 1.5
                                    },
                                    text: primary,
                                },
                                RemapButton::default(),
                                parent,
                                &fonts,
                                &ui,
                            );
                        });

                    parent
                        .spawn(Node {
                            width: Val::Px(100.),
                            height: Val::Px(UI_SCALE),
                            ..default()
                        })
                        .with_children(|parent| {
                            spawn_button(
                                ButtonType::Text {
                                    font_size: if secondary.len() < 13 {
                                        UI_SCALE * 2.5
                                    } else {
                                        UI_SCALE * 1.5
                                    },
                                    text: secondary,
                                },
                                (RemapButton, CustomSkinBehavior),
                                parent,
                                &fonts,
                                &ui,
                            );
                        });
                });
        });
}

pub fn configure(
    mut cmd: Commands,
    remap: Query<(Entity, &DepressButton), (With<RemapButton>, Without<RemapButtonActive>)>,
    remap_active: Query<(Entity, &DepressButton), With<RemapButtonActive>>,
) {
    for (entity, depress) in remap.iter() {
        if depress.invoked() {
            cmd.entity(entity).insert(RemapButtonActive {});
        }
    }

    if let Ok((entity, depress)) = remap_active.get_single() {}
}
