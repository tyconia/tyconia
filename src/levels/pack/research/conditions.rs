use bevy::prelude::*;

#[derive(Debug, Reflect, Clone)]
#[reflect(no_field_bounds)]
pub enum ConditionFlag {
    SatisfyAll(Vec<ConditionFlag>),
    SatisfyAny(Vec<ConditionFlag>),
    ReachedMetric { metric: String, value: f32 },
    UnderMetric { metric: String, value: f32 },
}
