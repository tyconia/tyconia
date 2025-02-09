use crate::levels::*;
use crate::InGameState;
use bevy::prelude::*;

pub struct TransportPlugin;

impl Plugin for TransportPlugin {
    fn build(&self, app: &mut App) {
        //app.add_systems(
        //    Update,
        //    (initiate_transfer,).run_if(in_state(InGameState::Normal)),
        //);
    }
}

/// component to be attached for receiver
#[derive(Component, Debug)]
#[require(Inventory)]
pub struct TransportInbound(pub Vec<ItemEntry>);

/// Items requested for tranfer, acts like filter.
/// this is created when the entity is created or is refreshed
/// if an entity is created pointing to this
#[derive(Component, Debug, Default)]
pub struct TransportDemand(pub Vec<ItemEntry>);

/// storage buffer of entity
//#[derive(Component, Debug, Default)]
//pub struct Inventory(pub Vec<ItemEntry>);

// at this part of the lifecycle, a source entity is given a demand which is a combination of
// different entities requesting different items
// after checking availability, item is subtracted from inventory and committed to the receiver entity as a TransportInbound component
//pub fn initiate_transfer(
//    mut cmd: Commands,
//    transport: Query<(Entity, &Inventory, &TransportDemand)>,
//) {
//    for (source, inventory, demand) in transport.iter() {
//        for (receiver, item) in demand.0.iter() {
//            if let Some(index) = inventory
//                .0
//                .iter()
//                .position(|(item_inventory, quantity)| item == item_inventory && *quantity > 0)
//            {
//                // finally subtract one item from inventory
//                cmd.entity(source)
//                    .entry::<Inventory>()
//                    .and_modify(move |mut inventory| {
//                        inventory.0.get_mut(index).unwrap().1 -= 1;
//                    });
//
//                // confront receiver entity
//                let item = item.clone();
//                cmd.entity(*receiver)
//                    .entry::<TransportInbound>()
//                    .and_modify(|mut inbound| inbound.0.push(item));
//            }
//        }
//    }
//}

#[derive(Event)]
pub enum Transport {
    Provider,
    Requester,
}

pub fn handle_transport(trigger: Trigger<Transport>) {}

pub fn observers(mut cmd: Commands) {
    let entity_1 = cmd.spawn_empty().id();

    let entity_2 = cmd.spawn(Observer::new(handle_transport).with_entity(entity_1));
}
