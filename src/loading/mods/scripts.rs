use crate::GameState;
use crate::ui::*;
use bevy::prelude::*;
use bevy_mod_scripting::rhai::RhaiRuntime;
use bevy_mod_scripting::core::{
    asset::{ScriptAsset, ScriptAssetLoader},
    bindings::{AppReflectAllocator, ReflectReference},
    callback_labels,
    event::ScriptCallbackEvent,
    handler::event_handler,
    script::ScriptComponent,
};
use bevy_mod_scripting::rhai::RhaiScriptingPlugin;
use bevy_mod_scripting::ScriptFunctionsPlugin;

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((RhaiScriptingPlugin::default(), ScriptFunctionsPlugin))
            .add_event::<ScriptCallbackEvent>()
            .init_resource::<AppReflectAllocator>()
            .add_systems(OnEnter(GameState::Playing), (load_scripts, ))
            .register_type::<PlayerMovement>()
            .add_systems(
                Update,
                (send_event_::<PlayerMovement>, event_handler::<OnEvent, RhaiScriptingPlugin>).chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)]
pub struct ScriptCentral(pub Vec<Handle<ScriptAsset>>);

pub fn load_scripts(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let script = asset_server.load::<ScriptAsset>("mods/tyconic/assets/scripts/hello_world.rhai");

    cmd.spawn((
        ScriptCentral(vec![script]),
        StateScoped(GameState::Playing),
        ScriptComponent(vec!["mods/tyconic/assets/scripts/hello_world.rhai".into()]),
    ));
}

#[derive(Reflect, Default)]
pub struct MyReflectType;

#[derive(Reflect, Default)]
pub struct PlayerMovement {
    pub acceleration: f32,
    pub max_speed: f32,
}


// define the label, you can define as many as you like here
callback_labels!(OnEvent => "on_event");

// trigger the event
fn send_event(
    mut writer: EventWriter<ScriptCallbackEvent>,
    allocator: ResMut<AppReflectAllocator>,

) {
    let mut allocator = allocator.write();
    let my_reflect_payload = ReflectReference::new_allocated(MyReflectType, &mut allocator);

    writer.send(ScriptCallbackEvent::new_for_all(
        OnEvent,
        vec![my_reflect_payload.into()],
    ));
}

fn send_event_<T: Reflect + Default>(
    mut writer: EventWriter<ScriptCallbackEvent>,
    allocator: ResMut<AppReflectAllocator>,
    type_registry: Res<AppTypeRegistry>,


    mut notification_channel: EventWriter<NotificationEvent>,
) {
    let registry = type_registry.read();

    if !registry.contains(std::any::TypeId::of::<T>()) {
        error!("Attempted to send event with unregistered type: {}", std::any::type_name::<T>());

        Notification {
            level: NotificationLevel::Error,
            title: "Unregistered type failure".into(),
            description: format!("Attempted to send event with unregistered type: {}", std::any::type_name::<T>()),
        }.queue(None, &mut notification_channel);
    }
    //assert!(registry.contains(std::any::TypeId::of::<T>()));

    let mut allocator = allocator.write();
    let reflect_payload = ReflectReference::new_allocated(T::default(), &mut allocator); // Ensure T: Default if needed

    writer.send(ScriptCallbackEvent::new_for_all(
        OnEvent,
        vec![reflect_payload.into()],
    ));
}
