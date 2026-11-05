use crate::hud::{EnableHUD, InventoryUISource};
use crate::*;
use bevy::prelude::*;

mod context_menu;

pub struct HotbarPlugin;

impl Plugin for HotbarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(EnableHUD::ENABLED),
            spawn_hotbar.after(super::spawn_hud_backdrop),
        )
        .add_systems(
            Update,
            (
                sync_hotbar_slots,
                repopulate_hotbar_slots,
                interact_hotbar_slot,
            )
                .chain()
                .run_if(any_with_component::<HotbarSlot>),
        );
    }
}

#[derive(Debug, Component, Default, Clone)]
#[require(ui::DepressButton)]
pub struct HotbarSlot {
    pub index: usize,
    pub entry: Option<ItemEntry>,
    pub selected: bool,
}

#[derive(Debug, Component, Default, Clone)]
pub struct UpdateHotbarSlot;

#[derive(Debug, Component, Default, Clone)]
#[require(ui::DepressButton)]
pub struct Hotbarslot {
    pub entry: Option<ItemId>,
}

#[derive(Debug, Default, Clone)]
pub enum HotbarSlotStatus {
    Disabled,
    Locked,
    #[default]
    Working,
}

fn sync_hotbar_slots(
    mut cmd: Commands,
    inventory: Query<(&Inventory, &InventoryActive), With<InventoryUISource>>,
    mut slots: Query<(Entity, &mut HotbarSlot)>,
) {
    let Ok((inventory, active)) = inventory.get_single() else {
        return;
    };

    for (entity, mut slot) in slots.iter_mut() {
        let entry = inventory.0.get(slot.index).cloned().flatten();
        let selected = active.0 == Some(slot.index);

        if slot.entry != entry || slot.selected != selected {
            slot.entry = entry;
            slot.selected = selected;
            cmd.entity(entity).insert(UpdateHotbarSlot);
        }
    }
}

fn repopulate_hotbar_slots(
    mut cmd: Commands,
    slots: Query<(Entity, &HotbarSlot, Option<&Children>), With<UpdateHotbarSlot>>,
    fonts: Res<loading::FontAssets>,
) {
    for (entity, slot, children) in slots.iter() {
        if let Some(children) = children {
            for child in children.iter() {
                cmd.entity(*child).despawn_recursive();
            }
        }

        cmd.entity(entity)
            .remove::<UpdateHotbarSlot>()
            .with_children(|parent| {
                let Some(entry) = &slot.entry else {
                    return;
                };

                parent
                    .spawn((
                        Node {
                            width: Val::Percent(100.),
                            height: Val::Percent(100.),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        PickingBehavior::IGNORE,
                    ))
                    .with_children(|label| {
                        let color = if slot.selected {
                            Color::srgb(1.0, 0.85, 0.3)
                        } else {
                            Color::WHITE
                        };

                        label.spawn((
                            Text::new(entry.label()),
                            TextColor(color),
                            TextFont {
                                font: fonts.jersey.clone(),
                                font_size: ui::UI_SCALE * 1.8,
                                font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                            },
                            PickingBehavior::IGNORE,
                        ));
                        label.spawn((
                            Text::new(entry.quantity_label()),
                            TextColor(color),
                            TextFont {
                                font: fonts.jersey.clone(),
                                font_size: ui::UI_SCALE * 1.8,
                                font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                            },
                            PickingBehavior::IGNORE,
                        ));
                    });
            });
    }
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
    let Ok(backdrop) = backdrop.get_single() else {
        return;
    };

    cmd.entity(backdrop).with_children(|parent| {
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
                    let entity_cmd = parent.spawn((
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
                        HotbarSlot {
                            index: i,
                            entry: None,
                            selected: false,
                        },
                        UpdateHotbarSlot,
                    ));
                }
            });
    });
}

pub fn item_entry_selection() {}

pub fn interact_hotbar_slot(
    mut inventory: Query<&mut InventoryActive, With<InventoryUISource>>,
    hotbar_slot: Query<(&HotbarSlot, &ui::DepressButton), Changed<ui::DepressButton>>,
) {
    let Ok(mut active) = inventory.get_single_mut() else {
        return;
    };

    for (slot, depress) in hotbar_slot.iter() {
        if depress.invoked() {
            active.0 = Some(slot.index);
        }
    }
}
