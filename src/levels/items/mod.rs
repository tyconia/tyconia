mod editor;
mod item;
mod namespace;
mod recipe;
mod research;

pub use editor::*;
pub use item::*;
pub use namespace::*;
pub use recipe::*;
pub use research::*;

use bevy::prelude::*;

#[derive(Debug, Resource, Reflect, Hash, PartialEq, Eq, Clone)]
pub struct ItemPack {
    meta: Meta,
    items: Vec<ItemId>,
    research: Vec<ResearchId>,
}

/// for discrimination
#[derive(Debug, Resource, Reflect, Hash, PartialEq, Eq, Clone)]
pub struct Meta {
    mod_name: String,
    namespace: Namespace,
    version: String,
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
