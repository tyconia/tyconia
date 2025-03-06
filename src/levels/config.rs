use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Reflect)]
pub struct Config(pub HashMap<String, String>);
