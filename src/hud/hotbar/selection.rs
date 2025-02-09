use super::super::EnableHUD;
use crate::loading;
use crate::menu;
use crate::ui::*;
use bevy::prelude::*;

pub struct CursorSelectPlugin;

impl Plugin for CursorSelectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(EnableHUD(true)),
            spawn_cursor_select.after(menu::settings::setup),
        )
        .add_systems(Update, interact_button.run_if(in_state(EnableHUD(true))));
    }
}

#[derive(Component)]
pub struct CursorSelect;

pub fn spawn_cursor_select(mut cmd: Commands, ui: Res<loading::UiAssets>) {
    cmd.spawn((
        StateScoped(crate::GameState::Playing),
        CursorSelect,
        Node {
            top: Val::Px(-6.),
            left: Val::Px(-6.),
            right: Val::Px(-6.),
            bottom: Val::Px(-6.),
            position_type: PositionType::Absolute,
            ..default()
        },
        //Visibility::Hidden,
        ZIndex::from(ZIndices::Selection),
        ImageNode {
            image: ui.selection.clone(),
            image_mode: bevy::ui::widget::NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect::from([6., 6., 6., 6.]),
                max_corner_scale: 3.,
                sides_scale_mode: SliceScaleMode::Stretch,
                center_scale_mode: SliceScaleMode::Stretch,
            }),
            ..default()
        },
    ));
}

pub fn interact_button(
    mut cmd: Commands,
    buttons: Query<(Entity, &Interaction), (With<DepressButton>, Changed<Interaction>)>,
    selection: Query<(Entity, Option<&Parent>), (With<CursorSelect>, Without<DepressButton>)>,
    mut hovered: Local<Option<Entity>>,
) {
    let (selection_entity, selection_parent) = selection.single();

    buttons.iter().for_each(|(entity, button)| match *button {
        Interaction::Hovered => {
            match selection_parent {
                Some(parent) => {
                    cmd.entity(**parent).remove_children(&[selection_entity]);
                }
                None => {
                    cmd.entity(entity).add_child(selection_entity);
                }
            };
            //*hovered = Some(entity);
        }
        Interaction::None => {
            //*hovered = None;
            match selection_parent {
                Some(parent) => {
                    cmd.entity(**parent).remove_children(&[selection_entity]);
                }
                None => {}
            };
        }
        Interaction::Pressed => {}
    });
}
