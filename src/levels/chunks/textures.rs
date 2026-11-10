use bevy::prelude::*;

pub struct TextureMapPlugin;

impl Plugin for TextureMapPlugin {
    fn build(&self, _app: &mut App) {}
}

pub trait TextureVector {
    fn to_vec(&self) -> Vec<Handle<Image>>;
}

impl TextureVector for crate::loading::ItemTextureMap {
    fn to_vec(&self) -> Vec<Handle<Image>> {
        self.0.iter().map(|(_, handle)| handle.clone()).collect()
    }
}

pub fn spawn_texture_map(_cmd: Commands) {}
