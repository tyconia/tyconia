use bevy::prelude::*;

/// How a structure volume participates in the cutaway rules.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[type_path = "tyconia::levels"]
pub enum OcclusionRole {
    /// A facade or wall that should be ghosted only when it hides the player.
    #[default]
    Facade,
    /// A roof slab. Hidden when the player is standing inside the footprint.
    Roof,
    /// An interior ceiling or overhang. Behaves like a roof for cutaways.
    Ceiling,
    /// Interior detail that should stay readable when its enclosing roof opens.
    Interior,
    /// Small props: they keep the existing rank-based visibility.
    Prop,
}

impl OcclusionRole {
    pub fn is_roof_like(self) -> bool {
        matches!(self, Self::Roof | Self::Ceiling)
    }
}

#[derive(Reflect, Debug, Clone)]
#[type_path = "tyconia::levels"]
pub struct SceneOcclusionDeclared {
    pub z_min: f32,
    #[reflect(default = "default_collider_height")]
    pub height: f32,
    /// Local polygon points relative to `SceneDeclared::position`. When empty,
    /// a diamond from `half_extents` is used.
    #[reflect(default)]
    pub footprint: Vec<Vec2>,
    #[reflect(default = "default_occlusion_half_extents")]
    pub half_extents: Vec2,
    #[reflect(default)]
    pub role: OcclusionRole,
}

pub fn default_occlusion_half_extents() -> Vec2 {
    Vec2::splat(32.0)
}

#[derive(Reflect, Debug, Clone)]
#[type_path = "tyconia::levels"]
pub struct SceneDeclared {
    pub asset: String,
    pub position: Vec2,
    pub scale: f32,
    pub z: f32,
    #[reflect(default)]
    pub elevation: f32,
    /// Optional structure volume. Scenes without one keep the legacy
    /// rank-based visibility.
    #[reflect(default)]
    pub occlusion: Option<SceneOcclusionDeclared>,
}

#[derive(Reflect, Debug, Clone)]
#[type_path = "tyconia::levels"]
pub struct ColliderDeclared {
    pub position: Vec2,
    pub half_extents: Vec2,
    #[reflect(default)]
    pub rotation_degrees: f32,
    #[reflect(default)]
    pub screen_space: bool,
    #[reflect(default)]
    pub vertices: Vec<Vec2>,
    #[reflect(default)]
    pub z_min: f32,
    #[reflect(default = "default_collider_height")]
    pub height: f32,
    #[reflect(default)]
    pub role: OcclusionRole,
}

pub fn default_collider_height() -> f32 {
    1.0
}

pub fn default_access_width() -> f32 {
    64.0
}

impl ColliderDeclared {
    pub fn z_max(&self) -> f32 {
        self.z_min + self.height.max(0.01)
    }

    pub fn same_vertical_range(&self, other: &Self) -> bool {
        (self.z_min - other.z_min).abs() < 0.01 && (self.height - other.height).abs() < 0.01
    }

    pub fn copy_vertical_from(&mut self, other: &Self) {
        self.z_min = other.z_min;
        self.height = other.height;
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[type_path = "tyconia::levels"]
pub enum AccessKind {
    #[default]
    Ramp,
    Stairs,
}

#[derive(Reflect, Debug, Clone)]
#[type_path = "tyconia::levels"]
pub struct FloorDeclared {
    pub elevation: f32,
    #[reflect(default)]
    pub rank: i16,
    pub cells: Vec<IVec2>,
}

#[derive(Reflect, Debug, Clone)]
#[type_path = "tyconia::levels"]
pub struct AccessDeclared {
    pub from: Vec2,
    pub to: Vec2,
    #[reflect(default = "default_access_width")]
    pub width: f32,
    pub from_elevation: f32,
    pub to_elevation: f32,
    #[reflect(default)]
    pub kind: AccessKind,
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[type_path = "tyconia::levels"]
pub enum NpcKind {
    #[default]
    Civilian,
    Guard,
}

#[derive(Reflect, Debug, Clone)]
#[type_path = "tyconia::levels"]
pub struct NpcDeclared {
    pub position: Vec2,
    #[reflect(default)]
    pub elevation: f32,
    #[reflect(default)]
    pub patrol: Vec<Vec2>,
    #[reflect(default)]
    pub kind: NpcKind,
}
