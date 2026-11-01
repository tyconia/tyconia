use super::SettingsTabsState;
use crate::loading::{FontAssets, UiAssets};
use crate::settings::{mods_dir, SettingsConfig};
use crate::ui::*;
use bevy::prelude::*;

#[derive(Debug, Component)]
pub struct ModToggle {
    pub id: String,
}

pub fn setup_enabled(
    mut cmd: Commands,
    backdrop: super::SettingsBackdropQuery,
    fonts: Res<FontAssets>,
    ui: Res<UiAssets>,
    settings: Res<SettingsConfig>,
    type_registry: Res<AppTypeRegistry>,
) {
    let Ok(backdrop) = backdrop.get_single() else {
        return;
    };

    let discovered = discover_mods(&type_registry);

    cmd.entity(backdrop).with_children(|parent| {
        parent
            .spawn((
                StateScoped(SettingsTabsState::ModsEnabled),
                Node {
                    width: Val::Percent(100.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(UI_SCALE),
                    ..default()
                },
                BackgroundColor(Color::srgba_u8(255, 255, 255, 150)),
            ))
            .with_children(|parent| {
                title_text("Enabled Mods", parent, &fonts);

                if discovered.is_empty() {
                    body_text(
                        "No mods found. Place mod folders in the configured mods directory.",
                        parent,
                        &fonts,
                    );
                }

                for (id, display_name) in discovered {
                    let enabled = settings.mods.enabled.iter().any(|entry| entry == &id);
                    parent
                        .spawn(Node {
                            width: Val::Percent(100.),
                            max_width: Val::Px(520.),
                            height: Val::Px(UI_SCALE * 5.),
                            padding: UiRect::axes(Val::Px(UI_SCALE), Val::Px(0.)),
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|parent| {
                            body_text(&display_name, parent, &fonts);
                            parent
                                .spawn(Node {
                                    height: Val::Px(UI_SCALE * 4.),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    spawn_checkbox(
                                        if enabled {
                                            CheckboxState::Active
                                        } else {
                                            CheckboxState::Inactive
                                        },
                                        ModToggle { id: id.clone() },
                                        parent,
                                        &fonts,
                                        &ui,
                                    );
                                });
                        });
                }
            });
    });
}

pub fn setup_workshop(
    mut cmd: Commands,
    backdrop: super::SettingsBackdropQuery,
    fonts: Res<FontAssets>,
) {
    let Ok(backdrop) = backdrop.get_single() else {
        return;
    };

    let path = mods_dir()
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_else(|| "unavailable".into());

    cmd.entity(backdrop).with_children(|parent| {
        parent
            .spawn((
                StateScoped(SettingsTabsState::ModsWorkshop),
                Node {
                    width: Val::Percent(100.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(UI_SCALE),
                    ..default()
                },
                BackgroundColor(Color::srgba_u8(255, 255, 255, 150)),
            ))
            .with_children(|parent| {
                title_text("Workshop", parent, &fonts);
                body_text("Local mod directory:", parent, &fonts);
                body_text(&path, parent, &fonts);
                body_text(
                    "Workshop downloads are not configured in this build.",
                    parent,
                    &fonts,
                );
            });
    });
}

pub fn configure_mods(
    mut settings: ResMut<SettingsConfig>,
    checkboxes: Query<(&CheckboxState, &ModToggle), Changed<CheckboxState>>,
) {
    for (state, toggle) in checkboxes.iter() {
        if state.is_active() {
            if !settings
                .mods
                .enabled
                .iter()
                .any(|entry| entry == &toggle.id)
            {
                settings.mods.enabled.push(toggle.id.clone());
            }
        } else {
            settings.mods.enabled.retain(|entry| entry != &toggle.id);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn discover_mods(type_registry: &AppTypeRegistry) -> Vec<(String, String)> {
    let Some(path) = mods_dir() else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(path) else {
        return Vec::new();
    };

    let mut mods = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
            let meta_path = entry.path().join("meta.ron");
            let ron = std::fs::read_to_string(&meta_path).ok()?;
            let mut deserializer = ron::de::Deserializer::from_str(&ron).ok()?;
            let registry = type_registry.read();
            let reflect_deserializer = bevy::reflect::serde::ReflectDeserializer::new(&registry);
            let value =
                serde::de::DeserializeSeed::deserialize(reflect_deserializer, &mut deserializer)
                    .ok()?;
            let pack = <crate::ModPack as bevy::reflect::FromReflect>::from_reflect(&*value)?;
            Some((
                pack.mod_id.to_string(),
                pack.descriptor.display_name.clone(),
            ))
        })
        .collect::<Vec<_>>();
    mods.sort_by(|a, b| a.0.cmp(&b.0));
    mods
}

#[cfg(target_arch = "wasm32")]
fn discover_mods(_type_registry: &AppTypeRegistry) -> Vec<(String, String)> {
    Vec::new()
}
