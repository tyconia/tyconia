use bevy::prelude::*;

#[derive(Reflect, Debug, Clone)]
#[type_path = "tyconia::levels"]
pub struct SceneDeclared {
    pub asset: String,
    pub position: Vec2,
    pub scale: f32,
    pub z: f32,
    #[reflect(default)]
    pub elevation: f32,
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
