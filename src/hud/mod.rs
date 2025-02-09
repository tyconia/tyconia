use crate::GameState;
use bevy::prelude::*;

mod hotbar;
mod inventory;

pub use hotbar::*;
pub use inventory::*;

pub struct HUDPlugin;

impl Plugin for HUDPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<EnableHUD>()
            .enable_state_scoped_entities::<EnableHUD>()
            .add_state_scoped_event::<EnableHUD>(GameState::Playing)
            .add_systems(
                Update,
                handle_hud_enable.run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnEnter(EnableHUD::ENABLED), (spawn_hud_backdrop,));

        app.add_plugins((HotbarPlugin, InventoryPlugin));
    }
}

#[derive(Debug, Component)]
pub struct HUDBackdrop;

pub type HUDBackdropQuery<'a, 'b> = Query<'a, 'b, Entity, With<HUDBackdrop>>;

pub fn spawn_hud_backdrop(mut cmd: Commands) {
    cmd.spawn((
        StateScoped(EnableHUD::ENABLED),
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

#[derive(SubStates, Clone, Eq, PartialEq, Debug, Hash, Copy, Event)]
#[source(GameState = GameState::Playing) ]
pub struct EnableHUD(pub bool);

impl EnableHUD {
    pub const ENABLED: Self = Self(true);
    pub const DISABLED: Self = Self(false);
}

impl Default for EnableHUD {
    fn default() -> Self {
        Self::ENABLED
    }
}

pub fn handle_hud_enable(
    mut enable_hud_channel: EventReader<EnableHUD>,
    enable_hud: Res<State<EnableHUD>>,
    mut next_enable_hud: ResMut<NextState<EnableHUD>>,
) {
    enable_hud_channel.read().for_each(|_| {
        info!("HUD toggled");
        next_enable_hud.set(EnableHUD(!enable_hud.0));
    });
}
