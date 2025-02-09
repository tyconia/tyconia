use crate::hud::EnableHUD;
use crate::GameState;
use crate::*;
use bevy::prelude::*;

mod context_menu;
mod selection;

pub struct HotbarPlugin;

impl Plugin for HotbarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(EnableHUD::ENABLED),
            (spawn_hotbar, populate_hotbar_slots)
                .chain()
                .after(super::spawn_hud_backdrop),
        )
        .add_systems(
            Update,
            (interact_hotbar_slot,).run_if(any_with_component::<HotbarSlot>),
        );
        //.add_plugins(selection::CursorSelectPlugin);
    }
}

#[derive(Debug, Component, Default, Clone)]
#[require(ui::DepressButton)]
pub struct HotbarSlot(pub Option<ItemEntry>);

#[derive(Debug, Component, Default)]
pub struct HotbarslotLocked;

pub fn populate_hotbar_slots(
    mut cmd: Commands,
    hotbar_slot: Query<(Entity, &HotbarSlot), Without<Children>>,
    textures: Res<loading::TextureAssets>,
    fonts: Res<loading::FontAssets>,
) {
    hotbar_slot.iter().for_each(|(entity, slot)| {
        cmd.entity(entity).with_children(|parent| {
            let mut spawn_child = parent.spawn((Node {
                height: Val::Px(ui::UI_SCALE * 8.),
                aspect_ratio: Some(1.),
                position_type: PositionType::Relative,

                overflow: Overflow::clip(),
                ..default()
            },));

            if let Some(quantity) = slot
                .0
                .as_ref()
                .and_then(|item_entry| match item_entry.item.0.as_str() {
                    "inserter" => Some((item_entry.quantity, textures.isometric_inserters.clone())),
                    "belt" => Some((item_entry.quantity, textures.isometric_belts.clone())),
                    "infinite_io" => Some((item_entry.quantity, textures.infinite_io.clone())),
                    "table" => Some((item_entry.quantity, textures.isometric_table.clone())),
                    _ => None,
                })
                .and_then(|(quantity, image)| {
                    spawn_child.insert(ImageNode {
                        image,
                        color: if quantity < 1 {
                            Color::srgba_u8(255, 255, 255, 100)
                        } else {
                            Default::default()
                        },
                        ..default()
                    });

                    Some(quantity)
                })
            {
                parent.spawn((
                    if quantity > 0 {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(0.),
                        left: Val::Px(ui::UI_SCALE * 0.75),
                        ..default()
                    },
                    Text::new(format!("{}", quantity)),
                    TextColor::BLACK,
                    TextFont {
                        font: fonts.jersey.clone(),
                        font_size: ui::UI_SCALE * 2.,
                        font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                    },
                ));
            }
        });
    });
}

#[derive(Component)]
pub struct HotBar {
    pub selection: Vec<Entity>,
}

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
                    //column_gap: Val::Px(ui::UI_SCALE),
                    margin: UiRect::axes(Val::Auto, Val::Px(0.)),
                    ..Default::default()
                },
                ImageNode {
                    // load default state
                    image: ui.inventory_slot.clone(),
                    image_mode: ui::BUTTON_IMG_MODE_SLICED,
                    ..Default::default()
                },
            ))
            .with_children(|parent| {
                for i in 0..9 {
                    let mut entity_cmd = parent.spawn((
                        ui::DepressButton::default(),
                        ui::ButtonSkins {
                            active: ui.inventory_slot_active.clone(),
                            hover: ui.inventory_slot_hover.clone(),
                            normal: ui.inventory_slot.clone(),
                        },
                        Node {
                            height: Val::Px(ui::UI_SCALE * 8.),
                            aspect_ratio: Some(17. / 18.),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::RowReverse,
                            overflow: Overflow::visible(),
                            position_type: PositionType::Relative,
                            ..Default::default()
                        },
                        ImageNode {
                            image: ui.inventory_slot.clone(),
                            image_mode: ui::BUTTON_IMG_MODE_SLICED,

                            ..Default::default()
                        },
                        HotbarSlot(Some(ItemEntry {
                            item: ItemId(
                                match i {
                                    0 => "inserter",
                                    1 => "belt",
                                    2 => "infinite_io",
                                    3 => "table",
                                    _ => "",
                                }
                                .into(),
                            ),
                            quantity: i,
                        })),
                    ));
                }
            });
    });
}

pub fn item_entry_selection() {}

pub fn interact_hotbar_slot(
    mut cmd: Commands,
    hotbar_slot: Query<(&Children, &HotbarSlot, &ui::DepressButton), Changed<ui::DepressButton>>,
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
