use super::Action;
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};

/// TODO: include gamepad mapping
pub struct InputMappingEntry {
    pub keys: HashSet<KeyCode>,
}

#[derive(Resource)]
pub struct InputMappingManager {
    entries: HashMap<Action, (InputMappingEntry)>,
}

impl InputMappingManager {
    fn input_map_actions(manager: Res<InputMappingManager>) {}
}
