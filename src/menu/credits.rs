use super::MenuNavState;
use crate::loading::{FontAssets, UiAssets};
use crate::ui::*;
use crate::ChangeStates;
use bevy::prelude::*;

pub struct CreditsPlugin;

impl Plugin for CreditsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuNavState::Credits), (setup,));
    }
}

pub fn setup(
    mut cmd: Commands,
    backdrop: super::MenuBackdropQuery,
    fonts: Res<FontAssets>,
    ui: Res<UiAssets>,
) {
    let Ok(backdrop) = backdrop.get_single() else {
        return;
    };

    cmd.entity(backdrop).with_children(|mut parent| {
        spawn_window(
            &mut parent,
            StateScoped(MenuNavState::Credits),
            ChangeStates(MenuNavState::Root),
            &ui,
            &fonts,
            WindowMeta::new("Credits".into(), 400., 9. / 16.),
            |parent| {
                parent
                    .spawn((
                        Node {
                            width: Val::Percent(100.),
                            height: Val::Percent(100.),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(UI_SCALE * 2.)),
                            row_gap: Val::Px(UI_SCALE),
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                        Scrollable,
                    ))
                    .with_children(|parent| {
                        section_text("Tyconic", parent, &fonts);
                        section_text("A factory automation game built with Bevy", parent, &fonts);
                        section_text("", parent, &fonts);
                        section_text("Developed by:", parent, &fonts);
                        body_text("The Tyconic Contributors", parent, &fonts);
                        section_text("", parent, &fonts);
                        section_text("Engine: Bevy 0.15", parent, &fonts);
                        section_text("Language: Rust", parent, &fonts);
                        section_text("", parent, &fonts);
                        section_text("Special thanks to:", parent, &fonts);
                        body_text("The Bevy Community", parent, &fonts);
                        body_text("Open source library authors", parent, &fonts);
                        section_text("", parent, &fonts);
                        section_text("License: MIT / Apache-2.0", parent, &fonts);
                    });
            },
        )
    });
}
