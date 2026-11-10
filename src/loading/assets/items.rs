use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component, Reflect)]
pub struct ItemTextureMap(pub HashMap<String, Handle<Image>>);
