//! windowed editor for making catalogue of research progression and unlocking recipes which then unlocks
//! new items

use crate::hud::*;
use crate::loading::*;
use crate::ui::*;
use bevy::prelude::*;

pub struct ResearchEditorPlugin;

#[derive(Debug, Component)]
pub struct ResearchEditor(pub crate::Meta);

impl Plugin for ResearchEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(EnableHUD::ENABLED),
            spawn_editor_window
                .after(spawn_hud_backdrop)
                .run_if(in_state(crate::DeveloperMode(true))),
        );
    }
}

pub fn spawn_editor_window(
    mut cmd: Commands,
    ui: Res<UiAssets>,
    fonts: Res<FontAssets>,
    hud_backdrop: HUDBackdropQuery,
) {
    let pack = crate::levels::pack::base_mod();

    cmd.entity(hud_backdrop.single())
        .with_children(|mut parent| {
            spawn_window(
                &mut parent,
                ResearchEditor(pack.meta.clone()),
                (),
                &ui,
                &fonts,
                WindowMeta::new(
                    format!(
                        "research editor `{}_{}`",
                        pack.meta.mod_name, pack.meta.version
                    ),
                    400.,
                    9. / 16.,
                ),
                |parent| {},
            );
        });
}
