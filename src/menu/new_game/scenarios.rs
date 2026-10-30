use super::*;
use crate::loading;
use bevy::prelude::*;

pub(crate) fn setup(
    mut cmd: Commands,
    backdrop: NewGameBackdropQuery,
    ui: Res<loading::UiAssets>,
    fonts: Res<loading::FontAssets>,
) {
    let Ok(backdrop) = backdrop.get_single() else {
        return;
    };

    cmd.entity(backdrop).with_children(|mut parent| {
        scenario_dropdown(&mut parent, &ui, &fonts);
    });
}

fn scenario_dropdown(
    parent: &mut ChildBuilder,
    ui: &Res<loading::UiAssets>,
    fonts: &Res<loading::FontAssets>,
) {
    parent
        .spawn((
            StateScoped(NewGameTabs::Scenarios),
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.),
                ..default()
            },
        ))
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
        })
        .with_children(|mut parent| {
            separator(parent);
            separator(parent);
            line_separator(parent, &ui);
            separator(parent);
            separator(parent);

            section_text("Scenario:", &mut parent, &fonts);
            body_text("Campaign - Tyconic", &mut parent, &fonts);
            separator(parent);
            separator(parent);

            section_text("Mods:", &mut parent, &fonts);
            line_separator(parent, &ui);
            separator(parent);
            body_text("Base_v0.0.0-dev", &mut parent, &fonts);
            body_text("Tyconic_v0.0.0-dev", &mut parent, &fonts);

            separator(parent);
            separator(parent);
            section_text("Tweaks:", &mut parent, &fonts);
            line_separator(parent, &ui);
            body_text("Unmodified", &mut parent, &fonts);
        });
}
