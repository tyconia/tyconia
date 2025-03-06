use bevy::prelude::*;

use std::collections::HashMap;

#[derive(Debug, Reflect)]
#[reflect(no_field_bounds)]
pub enum ConditionFlag {
    // Logical combinations
    SatisfyAll(Vec<ConditionFlag>),
    SatisfyAny(Vec<ConditionFlag>),
    // Basic comparisons
    ReachedMetric { metric: String, value: f32 },
    UnderMetric { metric: String, value: f32 },
}

/// Evaluates a condition against the tracked metrics.
pub fn evaluate_condition(condition: &ConditionFlag, tracked: &HashMap<String, f32>) -> bool {
    match condition {
        ConditionFlag::SatisfyAll(conditions) => {
            conditions.iter().all(|c| evaluate_condition(c, tracked))
        }
        ConditionFlag::SatisfyAny(conditions) => {
            conditions.iter().any(|c| evaluate_condition(c, tracked))
        }
        ConditionFlag::ReachedMetric { metric, value } => {
            let current = tracked.get(metric).unwrap_or(&0.0);
            *current >= *value
        }
        ConditionFlag::UnderMetric { metric, value } => {
            let current = tracked.get(metric).unwrap_or(&0.0);
            *current < *value
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a tracked metrics map from a list of key/value pairs.
    fn get_tracked(metrics: Vec<(&str, f32)>) -> HashMap<String, f32> {
        let mut map = HashMap::new();
        for (k, v) in metrics {
            map.insert(k.to_string(), v);
        }
        map
    }

    #[test]
    fn test_simple_at_least() {
        let condition = ConditionFlag::ReachedMetric {
            metric: "total_assets".to_string(),
            value: 15000.0,
        };
        let tracked = get_tracked(vec![("total_assets", 16000.0)]);
        assert!(evaluate_condition(&condition, &tracked));
    }

    #[test]
    fn test_simple_less_than() {
        let condition = ConditionFlag::UnderMetric {
            metric: "waste_generated".to_string(),
            value: 10.0,
        };
        let tracked = get_tracked(vec![("waste_generated", 5.0)]);
        assert!(evaluate_condition(&condition, &tracked));
    }

    #[test]
    fn test_and_condition() {
        let condition = ConditionFlag::SatisfyAll(vec![
            ConditionFlag::ReachedMetric {
                metric: "total_assets".to_string(),
                value: 15000.0,
            },
            ConditionFlag::ReachedMetric {
                metric: "repeat_customers".to_string(),
                value: 100.0,
            },
        ]);
        let tracked = get_tracked(vec![("total_assets", 16000.0), ("repeat_customers", 120.0)]);
        assert!(evaluate_condition(&condition, &tracked));
    }

    #[test]
    fn test_or_condition() {
        let condition = ConditionFlag::SatisfyAny(vec![ConditionFlag::ReachedMetric {
            metric: "repeat_customers".to_string(),
            value: 100.0,
        }]);
        let tracked = get_tracked(vec![
            ("customer_satisfaction", 80.0),
            ("repeat_customers", 120.0),
        ]);
        assert!(evaluate_condition(&condition, &tracked));
    }

    #[test]
    fn test_nested_condition() {
        // Build the complex condition:
        // 1. total_assets >= 15000
        // 2. Either customer_satisfaction >= 85 OR repeat_customers >= 100
        // 3. And, recipe_popularity (basic_burger) >= 50 and waste_generated < 10.
        let condition = ConditionFlag::SatisfyAll(vec![
            ConditionFlag::ReachedMetric {
                metric: "total_assets".to_string(),
                value: 15000.0,
            },
            ConditionFlag::SatisfyAny(vec![ConditionFlag::ReachedMetric {
                metric: "repeat_customers".to_string(),
                value: 100.0,
            }]),
            ConditionFlag::SatisfyAll(vec![ConditionFlag::UnderMetric {
                metric: "waste_generated".to_string(),
                value: 10.0,
            }]),
        ]);
        let tracked = get_tracked(vec![
            ("total_assets", 16000.0),
            ("customer_satisfaction", 80.0),
            ("repeat_customers", 120.0),
            ("recipe_popularity:basic_burger", 55.0),
            ("waste_generated", 5.0),
        ]);
        assert!(evaluate_condition(&condition, &tracked));
    }
}
