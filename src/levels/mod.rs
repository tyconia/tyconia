mod chunks;
mod editor;
mod items;
mod logistics;

use bevy::prelude::*;

pub use chunks::*;
pub use editor::*;
pub use items::*;
pub use logistics::*;

pub struct LevelsPlugin;

impl Plugin for LevelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ChunkPlugin,
            TransportPlugin,
            ResearchEditorPlugin,
            ToolBarPlugin,
        ));
    }
}
