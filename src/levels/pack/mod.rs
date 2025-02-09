mod base;
mod item;
mod namespace;
mod recipe;
mod research;

pub use base::*;
pub use item::*;
pub use namespace::*;
pub use recipe::*;
pub use research::*;
use std::fmt;

use bevy::prelude::*;

#[derive(Debug, Resource, Reflect, Hash, PartialEq, Eq, Clone)]
pub struct Pack {
    pub meta: Meta,
    pub description: String,
    pub items: Vec<ItemId>,
    pub research: Vec<ResearchId>,
    pub recipes: Vec<RecipeId>,
}

#[derive(Debug, Resource, Reflect, Hash, PartialEq, Eq, Clone)]
pub struct ItemPack(pub Vec<ItemId>);

#[derive(Debug, Resource, Reflect, Hash, PartialEq, Eq, Clone)]
pub struct ResearchPack(pub Vec<ResearchId>);

#[derive(Debug, Resource, Reflect, Hash, PartialEq, Eq, Clone)]
pub struct RecipePack(pub Vec<RecipeId>);

/// for discrimination
#[derive(Debug, Resource, Reflect, Hash, PartialEq, Eq, Clone)]
pub struct Meta {
    pub mod_name: String,
    pub namespace: Namespace,
    pub version: SemVer,
}

#[derive(Debug, Resource, Reflect, Hash, PartialEq, Eq, Clone)]
pub struct MetaDescriptor {
    pub display_name: String,
    pub thumbnail: Option<std::path::PathBuf>,
    pub cover_art: Option<std::path::PathBuf>,
    pub descripion: String,
    pub dependencies: Vec<MetaSource>,
}

#[derive(Debug, Resource, Reflect, Hash, PartialEq, Eq, Clone)]
pub struct MetaAttributions {
    pub authors: Vec<String>,
    pub licenses: Vec<String>,
    pub credits: Vec<String>,
}

#[derive(Debug, Resource, Reflect, Hash, PartialEq, Eq, Clone)]
pub struct ModPack {
    pub meta: Meta,
    pub descriptor: MetaDescriptor,
    pub attributions: MetaAttributions,
}

#[derive(Debug, Reflect, Hash, PartialEq, Eq, Clone, Copy)]
pub struct SemVer {
    major: u8,
    minor: u8,
    patch: u8,
    stage: SemverStage,
}

#[derive(Debug, Reflect, Hash, PartialEq, Eq, Clone, Copy)]
pub enum SemverStage {
    Dev,
    Nightly,
    ReleaseCandidate(u8),
    Stable,
}

impl fmt::Display for Meta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}_{}#{}", self.mod_name, self.version, self.namespace)
    }
}

impl<T: Into<String>> From<T> for MetaShorthand {
    fn from(value: T) -> Self {
        MetaShorthand(value.into())
    }
}

#[derive(Debug, Reflect, Hash, PartialEq, Eq, Clone)]
pub struct MetaSource {
    pub id: MetaShorthand,
    pub sources: Vec<MetaSources>,
}

#[derive(Debug, Reflect, Hash, PartialEq, Eq, Clone)]
pub struct MetaShorthand(pub String);

#[derive(Debug, Reflect, Hash, PartialEq, Eq, Clone)]
pub enum MetaSources {
    Path(std::path::PathBuf),
    Git(String),
}

impl fmt::Display for SemVer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.stage {
            SemverStage::Stable => write!(f, "{}.{}.{}", self.major, self.minor, self.patch),
            SemverStage::Dev => write!(f, "{}.{}.{}-dev", self.major, self.minor, self.patch),
            SemverStage::ReleaseCandidate(rc) => {
                write!(f, "{}.{}.{}-rc{}", self.major, self.minor, self.patch, rc)
            }
            SemverStage::Nightly => {
                write!(f, "{}.{}.{}-nightly", self.major, self.minor, self.patch)
            }
        }
    }
}

impl From<(u8, u8, u8)> for SemVer {
    fn from((major, minor, patch): (u8, u8, u8)) -> Self {
        Self {
            major,
            minor,
            patch,
            stage: SemverStage::Dev,
        }
    }
}

pub fn to_snake_case(input: &str) -> String {
    let mut result = String::new();
    let mut prev_was_upper = false;
    let mut prev_was_underscore = false;

    for (i, c) in input.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !prev_was_upper && !prev_was_underscore {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
            prev_was_upper = true;
            prev_was_underscore = false;
        } else if c.is_whitespace() {
            if !prev_was_underscore {
                result.push('_');
                prev_was_underscore = true;
            }
            prev_was_upper = false;
        } else {
            result.push(c);
            prev_was_upper = false;
            prev_was_underscore = false;
        }
    }

    result
}
