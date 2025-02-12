//! Surfaces facilitate a confined world in its own 'dimension' as abstract as a say, restaurant's
//! interior to the city where the said restaurant resides in, except represented as a single
//! building

use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_ecs_tilemap::prelude::*;
use std::borrow::Cow;

#[cfg(not(target_arch = "wasm32"))]
use crate::scripts::callbacks;

#[cfg(not(target_arch = "wasm32"))]
use bevy_mod_scripting::{core::handler::event_handler, rhai::RhaiScriptingPlugin};

pub struct SurfacesPlugin;

impl Plugin for SurfacesPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<SurfaceState>();

        #[cfg(not(target_arch = "wasm32"))]
        app.register_type::<SurfaceBuilder>()
            .add_systems(
                OnEnter(SurfaceState::Loading),
                event_handler::<callbacks::OnSurfaceCreate, RhaiScriptingPlugin>,
            )
            .add_systems(
                Update,
                (
                    //|| info!("entered surfaces"),
                    crate::mods::trigger_callback::<SurfaceBuilder, callbacks::OnSurfaceCreate>,
                )
                    .after(crate::mods::load_scripts)
                    .chain(),
            )
            .add_plugins(scripts::SurfacesScriptingPlugin);
    }
}

#[derive(Default, Clone, Eq, PartialEq, Hash, Debug, Reflect)]
pub struct SurfaceBuilder;

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

    use bevy::prelude::*;
    pub struct SurfacesScriptingPlugin;

    impl Plugin for SurfacesScriptingPlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(Startup, api::assign_surfaces_fn);
        }
    }

    pub mod api {
        use crate::GameState;
        use crate::SurfaceState;
        use bevy::prelude::*;
        use bevy_mod_scripting::core::{
            bindings::function::{namespace::*, script_function::FunctionCallContext},
            error::InteropError,
        };

        #[derive(Reflect)]
        pub struct Surfaces;

        pub fn assign_surfaces_fn(mut world: &mut World) {
            NamespaceBuilder::<Surfaces>::new_unregistered(&mut world)
                .register("add_surface", |s: String| {
                    info!("Adding new surface {}", s);
                })
                .register(
                    "trigger_resurface",
                    |ctx: FunctionCallContext| -> Result<(), InteropError> {
                        let world = ctx.world()?;
                        let next_state_id = world
                            .get_component_id(std::any::TypeId::of::<NextState<SurfaceState>>())?;
                        let next_state = world.get_resource(next_state_id.unwrap())?;
                        let next_state = next_state.unwrap().set(Box::new(SurfaceState::Loading));

                        Ok(())
                    },
                )
                .register(
                    "trigger_transition_surface",
                    |ctx: FunctionCallContext| -> Result<(), InteropError> {
                        let world = ctx.world()?;
                        let next_state_id = world
                            .get_component_id(std::any::TypeId::of::<NextState<SurfaceState>>())?;
                        let next_state = world.get_resource(next_state_id.unwrap())?;
                        let next_state = next_state
                            .unwrap()
                            .set(Box::new(SurfaceState::Loaded { current: None }));

                        Ok(())
                    },
                );
        }
    }

    pub mod callbacks {
        use bevy::prelude::*;
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

        pub fn ds() {}
    }
}
