use crate::ui::*;
use crate::GameState;
use bevy::prelude::*;
use bevy_mod_scripting::core::{
    asset::{ScriptAsset, ScriptAssetLoader},
    bindings::{function::namespace::*, AppReflectAllocator, ReflectReference},
    callback_labels,
    event::IntoCallbackLabel,
    event::ScriptCallbackEvent,
    handler::event_handler,
    script::ScriptComponent,
};
use bevy_mod_scripting::rhai::RhaiRuntime;
use bevy_mod_scripting::rhai::RhaiScriptingPlugin;
use bevy_mod_scripting::ScriptFunctionsPlugin;

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((RhaiScriptingPlugin::default(), ScriptFunctionsPlugin))
            .add_event::<ScriptCallbackEvent>()
            .init_resource::<AppReflectAllocator>()
            .register_type::<Base>()
            .add_systems(Startup, (assign_base_fn,))
            .add_systems(Startup, (load_scripts,).chain())
            .register_type::<PlayerMovement>()
            .add_systems(
                Update,
                (
                    //send_event_::<PlayerMovement>,
                    trigger_callback::<PlayerMovement, OnEvent>,
                    event_handler::<OnEvent, RhaiScriptingPlugin>,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)]
pub struct ScriptCentral(pub Vec<Handle<ScriptAsset>>);

pub fn load_scripts(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let hello_world_script = "mods/tyconic/assets/scripts/hello_world.rhai";
    let script = asset_server.load::<ScriptAsset>(hello_world_script);

    cmd.spawn((
        ScriptCentral(vec![script]),
        StateScoped(GameState::Playing),
        ScriptComponent(vec![hello_world_script.into()]),
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
callback_labels!(
    OnEvent => "on_event"
);

impl Default for OnEvent {
    fn default() -> Self {
        Self
    }
}

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

pub fn trigger_callback<T: Reflect + Default, E: IntoCallbackLabel + Default>(
    mut writer: EventWriter<ScriptCallbackEvent>,
    allocator: ResMut<AppReflectAllocator>,
    type_registry: Res<AppTypeRegistry>,

    mut notification_channel: EventWriter<NotificationEvent>,
) {
    let registry = type_registry.read();

    if !registry.contains(std::any::TypeId::of::<T>()) {
        error!(
            "Attempted to send event with unregistered type: {}",
            std::any::type_name::<T>()
        );

        Notification {
            level: NotificationLevel::Error,
            title: "Unregistered type failure".into(),
            description: format!(
                "Attempted to send event with unregistered type: {}",
                std::any::type_name::<T>()
            ),
        }
        .queue(None, &mut notification_channel);
    }
    //assert!(registry.contains(std::any::TypeId::of::<T>()));

    let mut allocator = allocator.write();
    let reflect_payload = ReflectReference::new_allocated(T::default(), &mut allocator); // Ensure T: Default if needed

    writer.send(ScriptCallbackEvent::new_for_all(
        E::default(),
        vec![reflect_payload.into()],
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
        error!(
            "Attempted to send event with unregistered type: {}",
            std::any::type_name::<T>()
        );

        Notification {
            level: NotificationLevel::Error,
            title: "Unregistered type failure".into(),
            description: format!(
                "Attempted to send event with unregistered type: {}",
                std::any::type_name::<T>()
            ),
        }
        .queue(None, &mut notification_channel);
    }
    //assert!(registry.contains(std::any::TypeId::of::<T>()));

    let mut allocator = allocator.write();
    let reflect_payload = ReflectReference::new_allocated(T::default(), &mut allocator); // Ensure T: Default if needed

    writer.send(ScriptCallbackEvent::new_for_all(
        OnEvent,
        vec![reflect_payload.into()],
    ));
}

#[derive(Reflect)]
pub struct Base;

fn assign_base_fn(mut world: &mut World) {
    NamespaceBuilder::<Base>::new(&mut world).register("hello_world", |s: String| {
        println!("welcome to the world, {}", s);
    });

    let mut builder = NamespaceBuilder::<Base>::new(world);
    builder
        .register("log_info", |text: String| {
            info!("{}", text);
        })
        .register("log_warning", |text: String| {
            info!("{}", text);
        })
        .register("log_error", |text: String| {
            error!("{}", text);
        })
        .register("log", |text: String| {
            debug!("{}", text);
        })
        .register("log_debug", |text: String| {
            debug!("{}", text);
        });
}
