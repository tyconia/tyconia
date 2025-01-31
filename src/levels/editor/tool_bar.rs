//! tool bar for summoning windowed editors

use crate::*;
use bevy::prelude::*;

pub struct ToolBarPlugin;

impl Plugin for ToolBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing),
            spawn_tool_bar
                .run_if(in_state(DeveloperMode(true)))
                .after(hud::spawn_hud_backdrop),
        );
    }
}

fn spawn_tool_bar(mut cmd: Commands, backdrop: crate::hud::HUDBackdropQuery) {
    cmd.entity(backdrop.single()).with_children(|parent| {
        parent.spawn((
            StateScoped(DeveloperMode(true)),
            StateScoped(GameState::Playing),
            Node {
                width: Val::Vw(20.),
                left: Val::Px(ui::UI_SCALE * 1.2),
                bottom: Val::Px(ui::UI_SCALE * 2.5),
                position_type: PositionType::Absolute,
                margin: UiRect::all(Val::Auto),
                ..default()
            },
            //ZIndex::from(crate::ui::ZIndices::Window),
            BackgroundColor(Color::WHITE),
        ));
    });
}
