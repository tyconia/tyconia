//! Surfaces facilitate a confined world in its own 'dimension' as abstract as a say, restaurant's
//! interior to the city where the said restaurant resides in, except represented as a single
//! building

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use std::ops::Range;

#[cfg(not(target_arch = "wasm32"))]
use crate::scripts::callbacks;

#[cfg(not(target_arch = "wasm32"))]
use bevy_mod_scripting::{core::handler::event_handler, rhai::RhaiScriptingPlugin};

pub struct SurfacesPlugin;

impl Plugin for SurfacesPlugin {
    fn build(&self, app: &mut App) {

        #[cfg(not(target_arch = "wasm32"))]
        app.add_sub_state::<SurfaceState>()
            .insert_resource(scripts::Surfaces { index: 10 });

        #[cfg(not(target_arch = "wasm32"))]
        app.register_type::<SurfaceBuilder>()
            //.add_systems(
            //    OnEnter(SurfaceState::Loading),
            //    event_handler::<scripts::callbacks::OnSurfaceCreate, RhaiScriptingPlugin>,
            //)
            //.add_systems(
            //    Update,
            //    (
            //        //|| info!("entered surfaces"),
            //        crate::mods::trigger_callback(
            //            scripts::callbacks::OnSurfaceCreate,
            //            SurfaceBuilder::default(),
            //        ),
            //    )
            //        .after(crate::mods::load_scripts)
            //        .chain(),
            //)
            //.add_plugins(scripts::SurfacesScriptingPlugin)
            ;
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

#[cfg(not(target_arch = "wasm32"))]
pub mod scripts {

    use crate::GameState;
    use bevy::prelude::*;
    use bevy_mod_scripting::{
        core::{
            bindings::{function::script_function::*, ReflectReference},
            error::InteropError,
        },
        script_bindings,
    };
    pub struct SurfacesScriptingPlugin;

    impl Plugin for SurfacesScriptingPlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(Startup, register_surfaces);
        }
    }

    #[derive(Reflect, Resource)]
    pub struct Surfaces {
        pub index: i32,
    }

    impl Surfaces {
        pub fn set(&mut self, num: i32) {
            self.index = num;
        }
    }

    #[allow(dead_code)]
    #[script_bindings(name = "surfaces")]
    impl Surfaces {
        pub fn create() {
            info!("Created surface");
        }

        pub fn add_to(num_1: i32, num_2: i32) -> i32 {
            num_1 + num_2
        }

        pub fn builder(ctx: FunctionCallContext) -> Result<ReflectReference, InteropError> {
            let world = ctx.world().unwrap();
            let id = world
                .get_resource_id(std::any::TypeId::of::<Surfaces>())?
                .unwrap();

            Ok(world.get_resource(id).unwrap().unwrap())
        }
    }

    pub mod callbacks {

        use bevy_mod_scripting::core::callback_labels;

        callback_labels!(
            OnTransitionSurface => "on_transition_surface",
            OnSurfaceCreate => "on_surface_create"
        );

        impl Default for OnSurfaceCreate {
            fn default() -> Self {
                Self
            }
        }

        impl Clone for OnSurfaceCreate {
            fn clone(&self) -> Self {
                Self
            }
        }
    }
}
