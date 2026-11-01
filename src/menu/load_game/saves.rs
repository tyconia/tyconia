use super::LoadGameTabs;
use crate::loading::{FontAssets, UiAssets};
use crate::ui::*;
use crate::{SaveCatalog, SelectedSave};
use bevy::prelude::*;

#[derive(Component)]
pub struct SaveEntryButton(pub usize);

pub(crate) fn setup(
    mut cmd: Commands,
    backdrop: super::LoadGameBackdropQuery,
    fonts: Res<FontAssets>,
    ui: Res<UiAssets>,
    saves: Res<SaveCatalog>,
) {
    let Ok(backdrop) = backdrop.get_single() else {
        return;
    };

    cmd.entity(backdrop).with_children(|parent| {
        parent
            .spawn(Node {
                width: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(UI_SCALE),
                ..default()
            })
            .with_children(|parent| {
                if saves.0.is_empty() {
                    section_text("No saves found", parent, &fonts);
                    body_text(
                        "Start a new game to create your first save.",
                        parent,
                        &fonts,
                    );
                    return;
                }

                section_text("Local saves", parent, &fonts);
                for (index, save) in saves.0.iter().enumerate() {
                    parent
                        .spawn(Node {
                            width: Val::Percent(100.),
                            height: Val::Px(UI_SCALE * 7.),
                            ..default()
                        })
                        .with_children(|parent| {
                            spawn_button(
                                ButtonType::Text {
                                    text: format!(
                                        "{}  -  {}  -  {}",
                                        save.metadata.name,
                                        save.metadata.world,
                                        save.metadata.playtime_label()
                                    ),
                                    font_size: UI_SCALE * 2.,
                                },
                                SaveEntryButton(index),
                                parent,
                                &fonts,
                                &ui,
                            );
                        });
                }
            });
    });
}

pub(crate) fn select_save(
    buttons: Query<(&DepressButton, &SaveEntryButton), Changed<DepressButton>>,
    mut selected: ResMut<SelectedSave>,
    mut next_tab: ResMut<NextState<LoadGameTabs>>,
) {
    for (button, save) in buttons.iter() {
        if button.invoked() {
            selected.0 = Some(save.0);
            next_tab.set(LoadGameTabs::Details);
        }
    }
}
