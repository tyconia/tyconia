use crate::loading::{FontAssets, UiAssets};
use crate::ui::*;
use crate::{GameState, SaveCatalog, SelectedSave};
use bevy::prelude::*;

pub(crate) fn setup(
    mut cmd: Commands,
    backdrop: super::LoadGameBackdropQuery,
    fonts: Res<FontAssets>,
    ui: Res<UiAssets>,
    saves: Res<SaveCatalog>,
    selected: Res<SelectedSave>,
) {
    let Ok(backdrop) = backdrop.get_single() else {
        return;
    };

    let save = selected.0.and_then(|index| saves.get(index));

    cmd.entity(backdrop).with_children(|parent| {
        parent
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.),
                row_gap: Val::Px(UI_SCALE),
                padding: UiRect::all(Val::Px(UI_SCALE * 2.)),
                ..default()
            })
            .with_children(|parent| {
                section_text("Save Details", parent, &fonts);

                let Some(save) = save else {
                    body_text(
                        "Select a save from the Saves tab to view details.",
                        parent,
                        &fonts,
                    );
                    return;
                };

                body_text(&format!("Name: {}", save.metadata.name), parent, &fonts);
                body_text(&format!("World: {}", save.metadata.world), parent, &fonts);
                body_text(&format!("Seed: {}", save.metadata.seed), parent, &fonts);
                body_text(
                    &format!("Playtime: {}", save.metadata.playtime_label()),
                    parent,
                    &fonts,
                );
                body_text(
                    &format!("Last played: {}", save.metadata.last_played),
                    parent,
                    &fonts,
                );
                body_text(
                    &format!("Version: {}", save.metadata.game_version),
                    parent,
                    &fonts,
                );
                body_text(
                    &format!(
                        "Mod profile: {}",
                        if save.metadata.mod_profile.is_empty() {
                            "vanilla".into()
                        } else {
                            save.metadata.mod_profile.join(", ")
                        }
                    ),
                    parent,
                    &fonts,
                );
                body_text(
                    &format!("File: {}", save.path.to_string_lossy()),
                    parent,
                    &fonts,
                );

                parent
                    .spawn(Node {
                        width: Val::Percent(100.),
                        height: Val::Px(UI_SCALE * 7.),
                        ..default()
                    })
                    .with_children(|parent| {
                        spawn_button(
                            ButtonType::Text {
                                text: "Load selected save".into(),
                                font_size: UI_SCALE * 2.4,
                            },
                            crate::ChangeStates(GameState::Playing),
                            parent,
                            &fonts,
                            &ui,
                        );
                    });
            });
    });
}
