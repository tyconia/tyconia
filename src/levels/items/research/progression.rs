use bevy::prelude::*;

#[derive(Component, Debug, Reflect, PartialEq, Eq, Clone, Hash)]
pub struct ResearchId {
    pub display_name: String,
    pub snake_name: String,
}

impl Default for ResearchId {
    fn default() -> Self {
        Self {
            display_name: "Unnamed Research".into(),
            snake_name: "unnamed_research".into(),
        }
    }
}

impl From<&str> for ResearchId {
    fn from(value: &str) -> Self {
        Self {
            display_name: value.to_string(),
            snake_name: super::super::to_snake_case(&value),
        }
    }
}

mod tests {
    #[test]
    fn snake_case_research_name() {
        let reseach_name = "Moving Baskets of Grain II";
        let research = super::ResearchId::from(reseach_name);

        assert_eq!(research.snake_name, "moving_baskets_of_grain_ii");
    }
}
