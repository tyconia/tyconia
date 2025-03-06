mod buildings;
mod built_in;
mod items;

pub use buildings::*;
pub use built_in::*;
pub use items::*;

use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Default)]
pub enum FieldValue {
    Int(i32),
    Float(f32),
    Text(String),
    Nest(DynamicFields),

    // add more variants as needed
    #[default]
    None,
}

#[derive(Default, Component)]
pub struct DynamicFields {
    pub fields: HashMap<String, FieldValue>,
}

#[derive(Reflect, Asset)]
pub struct ModAssets;
