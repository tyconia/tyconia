mod achievements;
mod conditions;
mod progression;

use crate::*;

pub use conditions::ConditionFlag;
pub use progression::*;

#[derive(Reflect)]
pub struct ResearchDeclared {
    pub id: ResearchId,
    pub display_name: String,
    pub flavor_text: String,
    pub unlock_condition: ConditionFlag,
    pub required_research: Vec<ResearchId>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::levels::pack;
    use bevy::prelude::*;
    use bevy::reflect::serde::{ReflectDeserializer, ReflectSerializer};
    use serde::de::DeserializeSeed;
    use std::fs;
    use std::io::{Read, Write};

    #[test]
    fn write_level() {
        let mut app = App::new();
        app.register_type::<ResearchDeclared>();
        app.add_systems(Startup, |type_registry: Res<AppTypeRegistry>| {
            let type_registry = type_registry.read();

            let research_declared = ResearchDeclared {
                id: ResearchId("foodie_i".into()),
                display_name: "Foodie I".into(),
                flavor_text: "Who knew food would sell :v".into(),
                unlock_condition: super::ConditionFlag::SatisfyAll(
                    [
                        super::ConditionFlag::ReachedMetric {
                            metric: "money".into(),
                            value: 20_000.,
                        },
                        super::ConditionFlag::SatisfyAny(
                            [
                                super::ConditionFlag::ReachedMetric {
                                    metric: "tyconic::cheese_wheel__total_produced".into(),
                                    value: 3_000.,
                                },
                                super::ConditionFlag::ReachedMetric {
                                    metric: "tyconic::beef_slab__total_produced".into(),
                                    value: 3_000.,
                                },
                            ]
                            .into(),
                        ),
                        super::ConditionFlag::UnderMetric {
                            metric: "tyconic::kitchen_waste__total_produced".into(),
                            value: 20.,
                        },
                    ]
                    .into(),
                ),
                required_research: [
                    ResearchId("lemon_stand_i".into()),
                    ResearchId("street_smart_ii".into()),
                ]
                .into(),
            };

            let reflect_serializer =
                bevy::reflect::serde::ReflectSerializer::new(&research_declared, &type_registry);

            let serialized = ron::ser::to_string_pretty(
                &reflect_serializer,
                ron::ser::PrettyConfig::new()
                    .depth_limit(4)
                    .indentor("  ".into()),
            )
            .unwrap();

            let file_path =
                std::path::Path::new("assets/mods/tyconic/assets/levels/research/foodie_i.ron");
            fs::write(&file_path, serialized).unwrap();
        });

        app.run();
    }
}
