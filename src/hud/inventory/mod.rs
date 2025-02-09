use crate::hud::*;
use crate::loading;
use crate::ui;
use crate::*;

mod interact;
mod interact_world;

pub use interact::*;
pub use interact_world::*;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<EnableInventory>()
            .enable_state_scoped_entities::<EnableInventory>()
            .add_state_scoped_event::<EnableInventory>(EnableHUD::ENABLED)
            .add_systems(
                Update,
                ((update_inventory_slots, handle_inventory_enable)
                    .run_if(in_state(EnableHUD::ENABLED)),),
            )
            .add_systems(
                OnEnter(EnableInventory::ENABLED),
                (spawn_inventory, populate_inventory_slots)
                    .chain()
                    .after(spawn_hud_backdrop),
            )
            .add_plugins((
                interact::InventoryInteractionPlugin,
                interact_world::InventoryInteractWorldPlugin,
            ));
    }
}

// indicates an inventory that must be displayed
#[derive(Component)]
pub struct InventoryUISource {
    pub display_width: usize,
    pub display_height: usize,
}

// widget that displays an inventory
#[derive(Component, Default)]
pub struct InventoryUI;

#[derive(Debug, Component, Default, Clone)]
#[require(ui::DepressButton)]
pub struct InventorySlot {
    pub entry: Option<ItemEntry>,
    pub index: usize,
    pub selected: bool,
}

pub fn update_inventory_slots(
    mut cmd: Commands,
    inventory_slot: Query<(Entity, &InventorySlot, &Children), Changed<InventorySlot>>,
    fonts: Res<loading::FontAssets>,
    textures: Res<loading::TextureAssets>,
) {
    inventory_slot
        .iter()
        .for_each(|(entity_slot, slot, children)| {
            children
                .iter()
                .for_each(|child| cmd.entity(*child).despawn_recursive());

            cmd.entity(entity_slot).with_children(|parent| {
                let mut spawn_child = parent.spawn((Node {
                    height: Val::Px(ui::UI_SCALE * 8.),
                    aspect_ratio: Some(1.),
                    position_type: PositionType::Relative,

                    overflow: Overflow::clip(),
                    ..default()
                },));

                if let Some(quantity) = slot
                    .entry
                    .as_ref()
                    .and_then(|item_entry| match item_entry.item.0.as_str() {
                        "auto_arm" => {
                            Some((item_entry.quantity, textures.isometric_inserters.clone()))
                        }
                        "mover_belt" => {
                            Some((item_entry.quantity, textures.isometric_belts.clone()))
                        }
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
                            top: Val::Px(ui::UI_SCALE),
                            left: Val::Px(ui::UI_SCALE * 2.),
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

pub fn populate_inventory_slots(
    mut cmd: Commands,
    hotbar_slot: Query<(Entity, &InventorySlot), Without<Children>>,
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
                .entry
                .as_ref()
                .and_then(|item_entry| match item_entry.item.0.as_str() {
                    "auto_arm" => Some((item_entry.quantity, textures.isometric_inserters.clone())),
                    "mover_belt" => Some((item_entry.quantity, textures.isometric_belts.clone())),
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

// TODO: support overflow for higher Inventory capacity for InventoryUISource(s)
// with less than display_width * display_height
pub fn spawn_inventory(
    mut cmd: Commands,
    backdrop: super::HUDBackdropQuery,
    inventory_source: Query<(&Inventory, &InventoryUISource, Option<&InventoryActive>)>,
    ui: Res<loading::UiAssets>,
    //textures: Res<loading::TextureAssets>,
    inventory_ui_active: Local<Option<Entity>>,
) {
    let (inventory_source, inventory_display, inventory_active) = inventory_source.single();

    cmd.entity(backdrop.single()).with_children(|parent| {
        parent
            .spawn((
                //InventoryUI::default(),
                StateScoped(EnableInventory::ENABLED),
                Node {
                    padding: UiRect::axes(Val::Px(ui::UI_SCALE * 2.), Val::Px(ui::UI_SCALE * 2.)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Row,
                    margin: UiRect::axes(Val::Auto, Val::Auto),
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
                parent
                    .spawn((Node {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Start,
                        flex_direction: FlexDirection::Column,

                        ..default()
                    },))
                    .with_children(|parent| {
                        for row in 0..inventory_display.display_height {
                            parent
                                .spawn((
                                    Node {
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        flex_direction: FlexDirection::Row,
                                        ..default()
                                    },
                                    //BackgroundColor(Color::BLACK)
                                ))
                                .with_children(|parent| {
                                    for col in 0..inventory_display.display_width {
                                        let index = row * inventory_display.display_width + col;
                                        if let Some(slot) = inventory_source.0.get(index) {
                                            let selected =
                                                inventory_active.map_or(false, |active| {
                                                    active.0.map_or(false, |active| active == index)
                                                });

                                            let entity_cmd = parent.spawn((
                                                ui::DepressButton::default(),
                                                ui::ButtonSkins {
                                                    hover: ui.inventory_slot_hover.clone(),
                                                    normal: ui.inventory_slot.clone(),
                                                    active: ui.inventory_slot_active.clone(),
                                                },
                                                Node {
                                                    height: Val::Px(ui::UI_SCALE * 8.),
                                                    aspect_ratio: Some(17. / 18.),
                                                    justify_content: JustifyContent::Center,
                                                    align_items: AlignItems::Center,
                                                    flex_direction: FlexDirection::RowReverse,
                                                    overflow: Overflow::visible(),
                                                    ..Default::default()
                                                },
                                                ImageNode {
                                                    image: if selected {
                                                        ui.inventory_slot_active.clone()
                                                    } else {
                                                        ui.inventory_slot.clone()
                                                    },
                                                    image_mode: ui::BUTTON_IMG_MODE_SLICED,

                                                    ..Default::default()
                                                },
                                                InventorySlot {
                                                    entry: slot.clone(),
                                                    index,
                                                    selected,
                                                },
                                                ui::CustomSkinBehavior,
                                            ));
                                        } else {
                                            break;
                                        }
                                    }
                                });
                        }
                    });
            });
    });
}

#[derive(SubStates, Clone, Eq, PartialEq, Debug, Hash, Copy, Event)]
#[source(EnableHUD = EnableHUD(true)) ]
pub struct EnableInventory(pub bool);

impl EnableInventory {
    pub const ENABLED: Self = Self(true);
    pub const DISABLED: Self = Self(false);
}

impl Default for EnableInventory {
    fn default() -> Self {
        //Self::ENABLED
        Self::DISABLED
    }
}

pub fn handle_inventory_enable(
    mut enable_inventory_channel: EventReader<EnableInventory>,
    enable_inventory: Res<State<EnableInventory>>,
    mut next_enable_inventory: ResMut<NextState<EnableInventory>>,
) {
    enable_inventory_channel.read().for_each(|_| {
        info!("Received event to toggle inventory");
        next_enable_inventory.set(EnableInventory(!enable_inventory.0));
    });
}
