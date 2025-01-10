use super::*;
use crate::GameState;
use bevy::input::gamepad::GamepadInput;
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};

/// Enables systems to dispatch actions based on [`InputMappings`] [`Resource`]
pub(crate) struct InputMappingsPlugin;

pub type MouseScrollEvent<'a, 'b> = EventReader<'a, 'b, MouseWheel>;
pub type MouseButtonResource<'a> = Res<'a, ButtonInput<MouseButton>>;
pub type KeyButtonResource<'a> = Res<'a, ButtonInput<KeyCode>>;

impl Plugin for InputMappingsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<InputMappings>()
            .init_resource::<InputMappings>()
            .init_resource::<IsometricMovement>()
            .add_systems(
                Update,
                (
                    dispatch_movement_actions,
                    print_events::<UiAction>,
                    dispatch_ui_actions,
                    dispatch_inter_actions,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

pub fn print_events<E: Event + std::fmt::Debug>(mut event: EventReader<E>) {
    event.read().for_each(|e| {
        info!("{} broadcasted: {:?}", std::any::type_name::<E>(), e);
    });
}

/// Control keys to be paired with actions
/// TODO: include gamepad mapping
#[derive(Reflect, Eq, PartialEq, Clone, Debug)]
pub struct InputMappingEntry {
    pub primary: Vec<DesktopControl>,
    pub secondary: Vec<DesktopControl>,
    pub primary_gamepad: Vec<GamepadInput>,
    pub secondary_gamepad: Vec<GamepadInput>,
}

impl InputMappingEntry {
    pub fn just_pressed<T>(
        &self,
        action: T,
        (mouse_button, key_button, mouse_scroll): (
            &MouseButtonResource,
            &KeyButtonResource,
            &MouseScrollEvent,
        ),
    ) -> Option<T> {
        if self.primary.iter().fold(true, |acc, ctrl| {
            acc && match ctrl {
                DesktopControl::Mouse(m) => match m {
                    Mouse::Button(mb) => mouse_button.just_pressed(*mb),
                    Mouse::Scroll { y, .. } => *y != 0,
                },
                DesktopControl::Key(k) => key_button.just_pressed(*k),
            }
        }) {
            return Some(action);
        }
        None
    }

    pub fn just_released<T>(&self, action: T) -> Option<T> {
        unimplemented!()
    }

    pub fn pressed<T>(&self, action: T) -> Option<T> {
        unimplemented!()
    }
}

impl Default for InputMappingEntry {
    fn default() -> Self {
        Self {
            primary: Vec::with_capacity(2),
            secondary: Vec::with_capacity(2),
            primary_gamepad: Vec::with_capacity(2),
            secondary_gamepad: Vec::with_capacity(2),
        }
    }
}

/// Mouse controls
#[derive(Reflect, Eq, PartialEq, Clone, Debug, Copy)]
pub enum Mouse {
    Button(MouseButton),
    Scroll { x: isize, y: isize },
}

/// Typical controls for desktop setup
#[derive(Reflect, Eq, PartialEq, Clone, Debug, Copy)]
pub enum DesktopControl {
    Key(KeyCode),
    Mouse(Mouse),
}

/// [`Resource`] stores input-actions mappings for configuration
#[derive(Resource, Reflect, Eq, PartialEq, Default, Clone, Debug, Asset)]
//#[reflect(from_reflect = false)]
pub struct InputMappings {
    pub ui_actions: HashMap<UiAction, InputMappingEntry>,
    pub movement_actions: HashMap<MovementAction, InputMappingEntry>,
    pub inter_actions: HashMap<InterAction, InputMappingEntry>,

    /// For mods and custom behavior
    pub custom_actions: HashMap<String, InputMappingEntry>,
}
