use super::*;
use crate::loading;
use bevy::prelude::*;

pub(crate) fn setup(
    mut cmd: Commands,
    backdrop: NewGameBackdropQuery,
    ui: Res<loading::UiAssets>,
    fonts: Res<loading::FontAssets>,
) {
    cmd.entity(backdrop.single()).with_children(|mut parent| {
        scenario_dropdown(&mut parent, &ui, &fonts);
    });
}

fn scenario_dropdown(
    parent: &mut ChildBuilder,
    ui: &Res<loading::UiAssets>,
    fonts: &Res<loading::FontAssets>,
) {
    parent
        .spawn((StateScoped(NewGameTabs::Scenarios), Node { ..default() }))
        .with_children(|parent| {
            //body_text("Scenarios", parent, fonts);
            spawn_dropdown(
                parent,
                &ui,
                &fonts,
                &[
                    "Campaign - Tyconic".to_string(),
                    "Freeplay - Tyconic".to_string(),
                    "Editor".to_string(),
                ],
                "Select Scenario".into(),
                None,
            );
        });
}
