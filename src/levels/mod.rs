mod chunks;
mod editor;
mod logistics;
mod pack;

use bevy::prelude::*;

pub use chunks::*;
pub use editor::*;
pub use logistics::*;
pub use pack::*;

pub struct LevelsPlugin;

impl Plugin for LevelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ChunkPlugin,
            TransportPlugin,
            ResearchEditorPlugin,
            //ModsMenuPlugin,
            //ToolBarPlugin,
        ));
    }
}
