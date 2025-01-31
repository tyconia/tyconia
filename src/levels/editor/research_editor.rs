//! windowed editor for making catalogue of research progression and unlocking recipes which then unlocks
//! new items

use crate::hud::*;
use crate::loading::*;
use crate::ui::*;
use crate::GameState;
use bevy::prelude::*;

pub struct ResearchEditorPlugin;

#[derive(Debug, Component)]
pub struct ResearchEditor(pub crate::Meta);

impl Plugin for ResearchEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing),
            spawn_editor_window.after(spawn_hud_backdrop),
        );
    }
}

pub fn spawn_editor_window(
    mut cmd: Commands,
    ui: Res<UiAssets>,
    fonts: Res<FontAssets>,
    hud_backdrop: HUDBackdropQuery,
) {
    let meta = crate::Meta {
        mod_name: "tyconic".into(),
        namespace: crate::Namespace::Vanilla,
        version: (0, 1, 0).into(),
    };
    cmd.entity(hud_backdrop.single())
        .with_children(|mut parent| {
            spawn_window(
                &mut parent,
                ResearchEditor(meta.clone()),
                (),
                &ui,
                &fonts,
                WindowMeta::new(
                    format!("research editor `{}_{}`", meta.mod_name, meta.version),
                    400.,
                    9. / 16.,
                ),
                |parent| {},
            );
        });
}
