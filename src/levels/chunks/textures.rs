use crate::ItemId;
use bevy::prelude::*;
use bevy::utils::HashMap;

pub struct TextureMapPlugin;

impl Plugin for TextureMapPlugin {
    fn build(&self, app: &mut App) {}
}

pub trait TextureVector {
    fn to_vec(&self) -> Vec<Handle<Image>>;
}

impl TextureVector for crate::loading::ItemTextureMap {
    fn to_vec(&self) -> Vec<Handle<Image>> {
        self.0.iter().map(|(_, handle)| handle.clone()).collect()
    }
}

pub fn spawn_texture_map(mut cmd: Commands) {}
