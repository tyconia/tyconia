use super::*;
use crate::ui::*;

pub struct InventoryInteractionPlugin;

impl Plugin for InventoryInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ((
                interact_inventory_local,
                reskin_inventory_slot,
                interact_inventory_with_hotbar,
            )
                .run_if(in_state(EnableInventory::ENABLED)),),
        );
    }
}

// interactions include:
// 1. clicking on a slot to select it and be able to left/right click to another slot
//  a. left click will move the entire item entry to the new slot, if occupied, will swap the 2 item
//      entries
//  b. right click will transfer one item at a time to an empty or a slot that has the same item
// 2. dragging on a slot and be able to release the mouse on another slot
fn interact_inventory_local(
    mut cmd: Commands,
    mut inventory_source: Query<
        (&mut Inventory, &mut InventoryActive),
        (With<InventoryUISource>, Without<InventorySlot>),
    >,
    mut inventory_ui_slots: Query<
        (Entity, Ref<DepressButton>, &mut InventorySlot),
        (With<InventorySlot>, Without<InventoryUISource>),
    >,
    mut swap_entries: Local<Option<(Entity, Entity)>>,
    mut unset_inventory_slot: Local<Option<Entity>>,
) {
    let (mut inventory_source, mut inventory_active) = inventory_source.single_mut();

    let active = inventory_ui_slots
        .iter()
        .find(|(_, _, inventory_slot)| inventory_slot.selected)
        .map(|(entity, _, _)| entity);

    for (inventory_slot_entity, depress, mut inventory_slot) in inventory_ui_slots.iter_mut() {
        // if not already selected, slot is marked selected
        if depress.is_changed() && depress.invoked() {
            // if there is no recorded active slot, set current slot as active
            if active.is_none() && !inventory_slot.selected {
                info!("inventory slot is selected");

                inventory_slot.selected = true;
                inventory_active.0 = Some(inventory_slot.index);
            } else if let Some(active_inventory_slot) = active {
                info!("inventory slot is deselected");

                // unset if it's the same slot as the current active
                if active_inventory_slot == inventory_slot_entity {
                    inventory_slot.selected = false;
                    continue;
                }

                // set to selected as false
                *unset_inventory_slot = Some(active_inventory_slot);

                *swap_entries = Some((active_inventory_slot, inventory_slot_entity));
                inventory_active.0.take();
            }
        }
    }

    swap_entries.take().map(|(active, current)| {
        let [(active_slot_entity, _, mut active_slot), (current_slot_entity, _, mut current_slot)] =
            inventory_ui_slots.get_many_mut([active, current]).unwrap();

        // swap in inventory vector
        inventory_source
            .0
            .swap(active_slot.index, current_slot.index);

        // swap in the UI, this is probably synchronized with inventory actual
        std::mem::swap(&mut active_slot.entry, &mut current_slot.entry);

        for entity in [active_slot_entity, current_slot_entity] {
            cmd.entity(entity).insert(UpdateInventorySlot);
        }
    });

    unset_inventory_slot.take().map(|unset| {
        for (_, _, mut unset) in inventory_ui_slots
            .get_many_mut([unset])
            .expect("unset entity is missing")
        {
            unset.selected = false;
        }
    });
}

fn reskin_inventory_slot(
    mut inventory_slots: Query<
        (
            &Interaction,
            &ButtonSkins,
            &mut ImageNode,
            &InventorySlot,
            &DepressButton,
        ),
        Or<(
            Changed<InventorySlot>,
            Changed<Interaction>,
            Changed<DepressButton>,
        )>,
    >,
) {
    inventory_slots
        .iter_mut()
        .for_each(|(interaction, skins, mut image, slot, depress)| {
            match *interaction {
                Interaction::Hovered => {
                    // do not reskin a slot if selected
                    if !slot.selected && !depress.invoked() && !depress.pressed {
                        image.image = skins.hover.clone();
                    }
                }
                Interaction::None => {
                    // do not reskin slot if selected
                    if !slot.selected {
                        image.image = skins.normal.clone();
                    }
                }
                Interaction::Pressed => {
                    image.image = skins.active.clone();

                    if slot.selected {
                        image.image = skins.hover.clone();
                    }
                }
            };
        });
}

fn interact_inventory_with_hotbar() {}
