use bevy::prelude::*;

pub fn setup(mut cmd: Commands, backdrop: super::SettingsBackdropQuery) {
    let Ok(backdrop) = backdrop.get_single() else {
        return;
    };

    cmd.entity(backdrop).with_children(|parent| {
        parent.spawn((StateScoped(super::SettingsTabsState::Audio),));
    });
}
