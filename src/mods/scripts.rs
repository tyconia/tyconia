use crate::*;
use bevy::prelude::*;

use bevy_mod_scripting::{
    core::{
        asset::ScriptAsset,
        bindings::{AppReflectAllocator, ReflectReference},
        callback_labels,
        event::{IntoCallbackLabel, Recipients, ScriptCallbackEvent},
        handler::event_handler,
        script::ScriptComponent,
    },
    rhai::RhaiScriptingPlugin,
    script_bindings, ScriptFunctionsPlugin,
};

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            (RhaiScriptingPlugin::default(), ScriptFunctionsPlugin),
            LogBindingsPlugin,
        ))
        .add_systems(
            OnEnter(GameState::Playing),
            (load_scripts,).after(load_mods_from_profile).chain(),
        )
        .register_type::<PlayerMovement>()
        .add_systems(
            Update,
            (
                trigger_callback(
                    OnEvent,
                    PlayerMovement {
                        max_speed: 60.,
                        acceleration: 4.,
                    },
                ),
                event_handler::<OnEvent, RhaiScriptingPlugin>,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component)]
pub struct ScriptHandles(pub Vec<Handle<ScriptAsset>>);

pub fn load_scripts(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    loaded_mods: Query<(Entity, &super::ModProfile), With<Level>>,
) {
    let script_location = "assets/scripts";
    let entry_point = "main.rhai";

    let (level_entity, loaded_mods) = loaded_mods.single();
    for (mod_, path) in loaded_mods.0.iter() {
        let script_path_buf = path.join(script_location).join(entry_point);
        let script_path = script_path_buf.to_string_lossy().into_owned();
        let script = asset_server.load::<ScriptAsset>(&*script_path);

        cmd.entity(level_entity).with_children(|parent| {
            parent.spawn((
                ScriptHandles(vec![script]),
                ScriptComponent(vec![script_path.into()]),
            ));
        });

        info!("loading scripts of mod {}", mod_.mod_id);
    }
}

#[derive(Reflect, Default, Clone)]
pub struct PlayerMovement {
    pub acceleration: f32,
    pub max_speed: f32,
}

// define the label, you can define as many as you like here
callback_labels!(
    OnEvent => "on_event"
);

impl Default for OnEvent {
    fn default() -> Self {
        Self
    }
}

impl Clone for OnEvent {
    fn clone(&self) -> Self {
        Self
    }
}

pub const fn trigger_callback<T: Reflect + Clone, E: IntoCallbackLabel + Clone>(
    event: E,
    payload: T,
) -> impl Fn(EventWriter<ScriptCallbackEvent>, ResMut<AppReflectAllocator>) {
    move |mut writer: EventWriter<ScriptCallbackEvent>, allocator: ResMut<AppReflectAllocator>| {
        let mut allocator = allocator.write();
        let reflect_payload = ReflectReference::new_allocated(payload.clone(), &mut allocator);

        writer.send(ScriptCallbackEvent::new(
            event.clone(),
            vec![reflect_payload.into(), "meowzer".into()],
            Recipients::All,
        ));
    }
}

// log namespace
#[derive(Reflect)]
pub struct Log;

#[allow(dead_code)]
#[script_bindings(name = "log")]
impl Log {
    fn info(text: String) {
        info!("{}", text);
    }

    fn err(text: String) {
        error!("{}", text);
    }

    fn warn(text: String) {
        warn!("{}", text);
    }

    fn debug(text: String) {
        debug!("{}", text);
    }
}

pub struct LogBindingsPlugin;

impl Plugin for LogBindingsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Log>()
            .add_systems(Startup, register_log);
    }
}
