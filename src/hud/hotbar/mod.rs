use crate::GameState;
use crate::*;
use bevy::prelude::*;

pub struct HotbarPlugin;

impl Plugin for HotbarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing),
            spawn_hotbar.after(super::spawn_hud_backdrop),
        )
        .add_systems(
            Update,
            (interact_hotbar_slot,).run_if(any_with_component::<Hotbarslot>),
        );
    }
}

#[derive(Debug, Component, Default)]
#[require(ui::DepressButton)]
pub struct Hotbarslot {
    pub item: Option<ItemEntry>,
}

#[derive(Debug, Component, Default)]
pub struct HotbarslotLocked;

pub fn spawn_hotbar(
    mut cmd: Commands,
    backdrop: super::HUDBackdropQuery,
    ui: Res<loading::UiAssets>,
    textures: Res<loading::TextureAssets>,
) {
    cmd.entity(backdrop.single()).with_children(|parent| {
        parent
            .spawn((
                Node {
                    padding: UiRect::all(Val::Px(ui::UI_SCALE * 1.2)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    bottom: Val::Px(ui::UI_SCALE * 2.5),
                    position_type: PositionType::Absolute,
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(ui::UI_SCALE),
                    margin: UiRect::axes(Val::Auto, Val::Px(0.)),
                    ..Default::default()
                },
                ImageNode {
                    // load default state
                    image: ui.inventory_slot.clone(),
                    image_mode: bevy::ui::widget::NodeImageMode::Sliced(TextureSlicer {
                        border: BorderRect::from([6., 6., 5., 5.]),
                        center_scale_mode: SliceScaleMode::Tile { stretch_value: 2.5 },
                        sides_scale_mode: SliceScaleMode::Tile { stretch_value: 2.5 },
                        max_corner_scale: 2.5,
                        ..default()
                    }),
                    ..Default::default()
                },
            ))
            .with_children(|parent| {
                for i in 0..9 {
                    parent
                        .spawn((
                            Node {
                                height: Val::Px(ui::UI_SCALE * 6.),
                                aspect_ratio: Some(1.),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                flex_direction: FlexDirection::RowReverse,
                                overflow: Overflow::clip(),
                                ..Default::default()
                            },
                            ImageNode {
                                image: ui.inventory_slot.clone(),
                                image_mode: bevy::ui::widget::NodeImageMode::Sliced(
                                    TextureSlicer {
                                        border: BorderRect::from([5., 5., 4., 4.]),
                                        center_scale_mode: SliceScaleMode::Tile {
                                            stretch_value: 1.5,
                                        },
                                        sides_scale_mode: SliceScaleMode::Tile {
                                            stretch_value: 1.5,
                                        },
                                        max_corner_scale: 1.5,
                                        ..default()
                                    },
                                ),
                                ..Default::default()
                            },
                            Hotbarslot::default(),
                        ))
                        .with_children(|parent| {
                            let mut spawn_child = parent.spawn((Node {
                                height: Val::Px(ui::UI_SCALE * 8.),
                                aspect_ratio: Some(1.),

                                overflow: Overflow::clip(),
                                ..default()
                            },));

                            if let Some(texture) = match i {
                                0 => Some(textures.isometric_table.clone()),
                                1 => Some(textures.isometric_inserters.clone()),
                                2 => Some(textures.isometric_belts.clone()),
                                3 => Some(textures.infinite_io.clone()),
                                _ => None,
                            } {
                                spawn_child.insert(ImageNode {
                                    image: texture,
                                    ..default()
                                });
                            }
                        });
                }
            });
    });
}

pub fn interact_hotbar_slot(
    mut cmd: Commands,
    hotbar_slot: Query<(&Children, &Hotbarslot, &ui::DepressButton), Changed<ui::DepressButton>>,
) {
    for (children, slot, depress) in hotbar_slot.iter() {
        if depress.invoked() {
            children.first().map(|child| {
                cmd.entity(*child)
                    .entry::<ImageNode>()
                    .and_modify(|mut img_node| {
                        img_node.color = Color::srgba_u8(255, 255, 255, 100);
                    });
            });
        }
    }
}
