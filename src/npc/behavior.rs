//! Mode switching is the decision stage of the NPC lifecycle.

use super::*;
use bevy::prelude::*;

/// Decide which lifecycle mode should be active this frame.
pub(crate) fn mode_for(
    perception: &NpcPerception,
    memory: &NpcMemory,
    patrol: &NpcPatrol,
) -> NpcMode {
    if perception.sees_target {
        NpcMode::Chase
    } else if memory.last_known.is_some() && memory.time_since_seen < memory.remember_for {
        NpcMode::Investigate
    } else if patrol.points.is_empty() {
        NpcMode::Idle
    } else {
        NpcMode::Patrol
    }
}

pub(super) fn npc_mode_switching(
    mut npcs: Query<(&NpcPerception, &NpcMemory, &NpcPatrol, &mut NpcMode)>,
) {
    for (perception, memory, patrol, mut mode) in &mut npcs {
        let next = mode_for(perception, memory, patrol);
        if *mode != next {
            *mode = next;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn perception(sees_target: bool) -> NpcPerception {
        NpcPerception {
            sees_target,
            target: sees_target.then_some(Vec2::new(10.0, 0.0)),
        }
    }

    #[test]
    fn mode_follows_vision_then_memory_then_patrol() {
        let patrol = NpcPatrol::new(vec![Vec2::new(20.0, 0.0)]);
        let mut memory = NpcMemory::default();

        assert_eq!(
            mode_for(&perception(true), &memory, &patrol),
            NpcMode::Chase
        );

        memory.last_known = Some(Vec2::new(10.0, 0.0));
        memory.time_since_seen = 1.0;
        assert_eq!(
            mode_for(&perception(false), &memory, &patrol),
            NpcMode::Investigate
        );

        memory.time_since_seen = memory.remember_for + 0.1;
        assert_eq!(
            mode_for(&perception(false), &memory, &patrol),
            NpcMode::Patrol
        );

        let idle = NpcPatrol::default();
        assert_eq!(mode_for(&perception(false), &memory, &idle), NpcMode::Idle);
    }
}
