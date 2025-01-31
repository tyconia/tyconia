use crate::GameState;
use bevy::prelude::*;

mod hotbar;

pub struct HUDPlugin;

impl Plugin for HUDPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), (spawn_hud_backdrop,));
        app.add_plugins(hotbar::HotbarPlugin);
    }
}

#[derive(Debug, Component)]
pub struct HUDBackdrop;

pub type HUDBackdropQuery<'a, 'b> = Query<'a, 'b, Entity, With<HUDBackdrop>>;

pub fn spawn_hud_backdrop(mut cmd: Commands) {
    cmd.spawn((
        StateScoped(GameState::Playing),
        Node {
            width: Val::Vw(100.),
            height: Val::Vh(100.),
            position_type: PositionType::Relative,
            ..default()
        },
        HUDBackdrop,
        //BackgroundColor(Color::srgba(255., 255., 255., 0.2)),
    ));
}
