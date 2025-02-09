mod built_in;

pub use built_in::*;

use bevy::prelude::*;

#[derive(Reflect, Asset)]
pub struct ModAssets;

fn ass() {
    let asset = ModAssets;
}
