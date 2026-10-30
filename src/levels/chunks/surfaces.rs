//! Surfaces facilitate a confined world in its own 'dimension' as abstract as a say, restaurant's
//! interior to the city where the said restaurant resides in, except represented as a single
//! building

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use std::ops::Range;

pub struct SurfacesPlugin;

impl Plugin for SurfacesPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(target_arch = "wasm32"))]
        app.add_sub_state::<SurfaceState>()
            .register_type::<SurfaceBuilder>();
    }
}

#[derive(Default, Clone, Eq, PartialEq, Hash, Debug, Reflect)]
pub struct SurfaceBuilder {
    pub size: (usize, usize),
    pub texture_indices: Vec<usize>,
}

impl SurfaceBuilder {
    fn create_map(cmd: &mut Commands) -> TilemapId {
        let tilemap = cmd.spawn_empty().id();

        TilemapId(tilemap)
    }

    fn to_surface<T: Into<Vec<TilemapId>>>(self, maps: T) -> Surface {
        Surface { maps: maps.into() }
    }
}

#[derive(Default, Clone, Eq, PartialEq, Hash, Debug, Reflect)]
pub struct Surface {
    pub maps: Vec<TilemapId>,
}

use crate::GameState;

#[derive(Default, Clone, Eq, PartialEq, Hash, Debug, SubStates, Reflect)]
#[source(GameState = GameState::Playing) ]
pub enum SurfaceState {
    #[default]
    Loading,
    Loaded {
        current: Option<Surface>,
    },
}

impl SurfaceState {
    //pub fn add_new_surface(
    //    &mut self,
    //    override_prev: bool,
    //    surface_label: String,
    //    surface: Surface,
    //) -> Result<(), ()> {
    //    if override_prev {
    //        self.surfaces.insert(surface_label, surface);
    //    } else if !self.surfaces.contains_key(&surface_label) {
    //        self.surfaces.insert(surface_label, surface);
    //    }
    //
    //    Ok(())
    //}
}

