use super::*;
use crate::loading;
use crate::ui::*;
use bevy::prelude::*;

pub(crate) fn setup(
    mut cmd: Commands,
    backdrop: NewGameBackdropQuery,
    ui: Res<loading::UiAssets>,
    fonts: Res<loading::FontAssets>,
    settings: Res<crate::SettingsConfig>,
) {
    let Ok(backdrop) = backdrop.get_single() else {
        return;
    };

    let enabled = settings
        .mods
        .enabled
        .iter()
        .any(|id| id.starts_with("tyconic"));

    cmd.entity(backdrop).with_children(|mut parent| {
        mods_checkbox(&mut parent, &ui, &fonts, enabled);
    });
}

pub(crate) fn load_mods() {}

pub(crate) fn mods_checkbox(
    cmd: &mut ChildBuilder,
    ui: &Res<loading::UiAssets>,
    fonts: &Res<loading::FontAssets>,
    enabled: bool,
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
                section_text("0.0.0-dev".into(), parent, &fonts);
            });

        parent
            .spawn(Node {
                height: Val::Px(UI_SCALE * 6.),
                ..default()
            })
            .with_children(|parent| {
                spawn_checkbox(
                    if enabled {
                        CheckboxState::Active
                    } else {
                        CheckboxState::Inactive
                    },
                    ModCheckbox(MetaShorthand("tyconic_0.0.0-dev".into())),
                    parent,
                    &fonts,
                    &ui,
                );
            });
    });
}

#[derive(Component)]
pub struct ModCheckbox(pub MetaShorthand);

pub fn interact_mods_checkbox(
    mut new_game: Query<&mut NewGame, Without<CheckboxState>>,
    mut pending: ResMut<PendingModProfile>,
    checkboxes: Query<(&CheckboxState, &ModCheckbox), Changed<CheckboxState>>,
) {
    let Ok(mut new_game) = new_game.get_single_mut() else {
        return;
    };

    for (checkbox, mod_) in checkboxes.iter() {
        if checkbox.is_active() {
            new_game.profile.0.insert(mod_.0.clone());
        } else {
            new_game.profile.0.remove(&mod_.0);
        }
        pending.0 = new_game.profile.clone();
    }
}
