use bevy::prelude::*;

pub fn setup(mut cmd: Commands, backdrop: super::SettingsBackdropQuery) {
    cmd.entity(backdrop.single()).with_children(|parent| {
        parent.spawn((StateScoped(super::SettingsTabsState::Audio),));
    });
}
