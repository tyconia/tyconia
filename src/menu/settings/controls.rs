use super::SettingsTabsState;
use crate::loading::*;
use crate::ui::*;
use bevy::prelude::*;

use crate::actions::*;

pub fn setup(
    mut cmd: Commands,
    backdrop: super::SettingsBackdropQuery,
    fonts: Res<FontAssets>,
    ui: Res<UiAssets>,
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
                input_map_entry("Summon menu".into(), parent, &fonts, &ui);
                input_map_entry("Next hotbar slot".into(), parent, &fonts, &ui);
                input_map_entry("Previous hotbar slot".into(), parent, &fonts, &ui);
                separator(parent);

                section_text("Movement", parent, &fonts);
                input_map_entry("North".into(), parent, &fonts, &ui);
                input_map_entry("South menu".into(), parent, &fonts, &ui);
                input_map_entry("East".into(), parent, &fonts, &ui);
                input_map_entry("West".into(), parent, &fonts, &ui);
                separator(parent);

                section_text("Interface", parent, &fonts);
                input_map_entry("Zoom".into(), parent, &fonts, &ui);
                input_map_entry("Menu".into(), parent, &fonts, &ui);
                separator(parent);

                section_text("Custom", parent, &fonts);
            });
    });
}

#[derive(Debug, Component, Default)]
#[require(Button)]
pub struct RemapButton;

#[derive(Debug, Component, Default)]
#[require(Button)]
pub struct RemapButtonActive {}

fn input_map_entry(
    action: String,
    parent: &mut ChildBuilder,
    fonts: &Res<FontAssets>,
    ui: &Res<UiAssets>,
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
            body_text(&action, parent, &fonts);
            //spawn_button("".into(), (), parent, &fonts, &ui);

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
                                    image: ui.undo_ico.clone(),
                                    image_size: Val::Px(16.),
                                },
                                (),
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
                                    text: "<unbounded>".into(),
                                    font_size: UI_SCALE * 3.,
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
                                    text: "<unbounded>".into(),
                                    font_size: UI_SCALE * 3.,
                                },
                                RemapButton::default(),
                                parent,
                                &fonts,
                                &ui,
                            );
                        });
                });
        });
}

pub fn configure() {}
