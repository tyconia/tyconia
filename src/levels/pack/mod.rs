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
use std::{fmt, str::FromStr};

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
    //pub namespace: Namespace,
    pub version: SemVer,
}

impl fmt::Display for Meta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}_{}", self.mod_name, self.version)
    }
}

impl FromStr for Meta {
    type Err = ParseMetaError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.rsplitn(2, '_');
        let version_part = parts
            .next()
            .ok_or("Missing version")
            .map_err(|_| ParseMetaError)?;
        let mod_name_part = parts
            .next()
            .ok_or("Missing mod name")
            .map_err(|_| ParseMetaError)?;

        let version = version_part.parse::<SemVer>().map_err(|_| ParseMetaError)?;

        Ok(Meta {
            mod_name: mod_name_part.to_string(),
            version,
        })
    }
}

#[derive(Debug)]
pub struct ParseMetaError;

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

#[derive(Debug, Reflect, Hash, PartialEq, Eq, Clone)]
pub struct MetaSource {
    pub id: MetaShorthand,
    pub sources: Vec<MetaSources>,
}

#[derive(Debug, Reflect, Hash, PartialEq, Eq, Clone)]
pub struct MetaShorthand(pub String);

impl From<&'static str> for MetaShorthand {
    fn from(s: &'static str) -> Self {
        Self(s.into())
    }
}

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

#[derive(Debug)]
pub struct ParseSemVerError;

impl fmt::Display for ParseSemVerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid semver format")
    }
}

impl FromStr for SemVer {
    type Err = ParseSemVerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('-');
        let version_part = parts.next().ok_or(ParseSemVerError)?;
        let stage_part = parts.next(); // Optional stage

        let mut version_numbers = version_part.split('.');
        let major = version_numbers
            .next()
            .and_then(|v| v.parse().ok())
            .ok_or(ParseSemVerError)?;
        let minor = version_numbers
            .next()
            .and_then(|v| v.parse().ok())
            .ok_or(ParseSemVerError)?;
        let patch = version_numbers
            .next()
            .and_then(|v| v.parse().ok())
            .ok_or(ParseSemVerError)?;

        let stage = match stage_part {
            Some("dev") => SemverStage::Dev,
            Some("nightly") => SemverStage::Nightly,
            Some(rc) if rc.starts_with("rc") => {
                let rc_num = rc[2..].parse().map_err(|_| ParseSemVerError)?;
                SemverStage::ReleaseCandidate(rc_num)
            }
            Some(_) => return Err(ParseSemVerError),
            None => SemverStage::Stable,
        };

        Ok(SemVer {
            major,
            minor,
            patch,
            stage,
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_parsing_valid() {
        let input = "some_mod_1.2.3";
        let expected = Meta {
            mod_name: "some_mod".to_string(),
            version: SemVer {
                major: 1,
                minor: 2,
                patch: 3,
                stage: SemverStage::Stable,
            },
        };

        let parsed = input.parse::<Meta>().unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn meta_parsing_with_stage() {
        let input = "mega_factory_10.4.1-nightly";
        let expected = Meta {
            mod_name: "mega_factory".to_string(),
            version: SemVer {
                major: 10,
                minor: 4,
                patch: 1,
                stage: SemverStage::Nightly,
            },
        };

        let parsed = input.parse::<Meta>().unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn meta_formatting() {
        let meta = Meta {
            mod_name: "cooking_time".to_string(),
            version: SemVer {
                major: 2,
                minor: 0,
                patch: 0,
                stage: SemverStage::ReleaseCandidate(3),
            },
        };

        let formatted = meta.to_string();
        assert_eq!(formatted, "cooking_time_2.0.0-rc3");
    }

    #[test]
    fn meta_parsing_invalid_missing_version() {
        let input = "mod_without_version";
        assert!(input.parse::<Meta>().is_err());
    }

    #[test]
    fn meta_parsing_invalid_format_extra_underscore() {
        let input = "mod_with_extra_underscore_1.0.0";
        let parsed = input.parse::<Meta>();
        assert!(parsed.is_ok()); // Should still work since it finds the LAST `_`
        assert_eq!(parsed.unwrap().mod_name, "mod_with_extra_underscore");
    }
}
