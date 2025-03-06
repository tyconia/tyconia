use super::*;
use crate::loading;
use crate::ui::*;
use bevy::prelude::*;

pub(crate) fn setup(
    mut cmd: Commands,
    backdrop: NewGameBackdropQuery,
    ui: Res<loading::UiAssets>,
    fonts: Res<loading::FontAssets>,
) {
    cmd.entity(backdrop.single()).with_children(|mut parent| {
        mods_checkbox(&mut parent, &ui, &fonts);
    });
}

pub(crate) fn mods_checkbox(
    cmd: &mut ChildBuilder,
    ui: &Res<loading::UiAssets>,
    fonts: &Res<loading::FontAssets>,
) {
    cmd.spawn((
        StateScoped(NewGameTabs::Mods),
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            width: Val::Percent(100.),
            ..default()
        },
    ))
    .with_children(|parent| {
        parent
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .with_children(|parent| {
                body_text("tyconic".into(), parent, &fonts);
                section_text("0.1.0-dev".into(), parent, &fonts);
            });

        parent
            .spawn(Node {
                height: Val::Px(UI_SCALE * 6.),
                ..default()
            })
            .with_children(|parent| {
                spawn_checkbox(CheckboxState::Inactive, (), parent, &fonts, &ui);
            });
    });
}
