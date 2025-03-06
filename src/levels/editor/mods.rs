use crate::hud::*;
use crate::loading::*;
use crate::ui::*;
use crate::GameState;
use bevy::prelude::*;

pub struct ModsMenuPlugin;

impl Plugin for ModsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing),
            spawn_mods_window
                .after(spawn_hud_backdrop)
                .run_if(in_state(crate::DeveloperMode(true))),
        );
    }
}

pub fn mod_entry(
    parent: &mut ChildBuilder,
    mod_meta: &crate::Meta,
    ui: &Res<UiAssets>,
    fonts: &Res<FontAssets>,
) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(UI_SCALE)),
                ..default()
            },
            ImageNode {
                image: ui.inventory_slot.clone(),
                image_mode: NODE_IMG_MODE_SLICED,
                ..Default::default()
            },
        ))
        .with_children(|mut parent| {
            body_text(&mod_meta.mod_name, &mut parent, &fonts);
            section_text(&format!("{}", mod_meta.version), &mut parent, &fonts);
        });
}

pub fn spawn_mods_window(
    mut cmd: Commands,
    hud_backdrop: HUDBackdropQuery,
    ui: Res<UiAssets>,
    fonts: Res<FontAssets>,
) {
    // describes features that comes with the binary like belts, inserters
    let base_mod = crate::Meta {
        mod_name: "base".into(),
        version: (0, 0, 0).into(),
    };

    // describes features that comes with the binary like belts, inserters
    let tyconic_mod = crate::Meta {
        mod_name: "tyconic".into(),
        version: (0, 0, 0).into(),
    };

    cmd.entity(hud_backdrop.single())
        .with_children(|mut parent| {
            spawn_window(
                &mut parent,
                (),
                (),
                &ui,
                &fonts,
                WindowMeta::new("mod menu".into(), 400., 9. / 16.),
                |parent| {
                    parent
                        .spawn((
                            Node {
                                margin: UiRect::all(Val::Px(UI_SCALE)),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(UI_SCALE),
                                ..default()
                            },
                            Scrollable,
                        ))
                        .with_children(|mut parent| {
                            mod_entry(&mut parent, &base_mod, &ui, &fonts);
                            mod_entry(&mut parent, &tyconic_mod, &ui, &fonts);
                        });
                },
            );
        });
}
