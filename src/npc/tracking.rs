//! Tracking converts perception into short-term memory and decays it over time.

use super::*;
use bevy::prelude::*;

/// Update memory from the latest perception result.
pub(crate) fn update_memory(
    perception: &NpcPerception,
    memory: &mut NpcMemory,
    delta_seconds: f32,
) {
    if perception.sees_target {
        if let Some(target) = perception.target {
            memory.last_known = Some(target);
        }
        memory.time_since_seen = 0.0;
    } else {
        memory.time_since_seen += delta_seconds.max(0.0);
        if memory.time_since_seen >= memory.remember_for {
            memory.last_known = None;
        }
    }
}

pub(super) fn npc_tracking(time: Res<Time>, mut npcs: Query<(&NpcPerception, &mut NpcMemory)>) {
    let delta = time.delta_secs();
    for (perception, mut memory) in &mut npcs {
        update_memory(perception, &mut memory, delta);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_tracks_visible_targets_and_forgets_after_timeout() {
        let mut memory = NpcMemory::default();
        let seen = NpcPerception {
            sees_target: true,
            target: Some(Vec2::new(4.0, 7.0)),
        };

        update_memory(&seen, &mut memory, 0.1);
        assert_eq!(memory.last_known, Some(Vec2::new(4.0, 7.0)));
        assert_eq!(memory.time_since_seen, 0.0);

        let unseen = NpcPerception::default();
        update_memory(&unseen, &mut memory, 1.0);
        assert_eq!(memory.last_known, Some(Vec2::new(4.0, 7.0)));
        assert_eq!(memory.time_since_seen, 1.0);

        let remember_for = memory.remember_for;
        update_memory(&unseen, &mut memory, remember_for);
        assert_eq!(memory.last_known, None);
    }
}
