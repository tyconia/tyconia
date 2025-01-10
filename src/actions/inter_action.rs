use crate::actions::mappings::InputMappings;
use bevy::prelude::*;

#[derive(Event, Reflect, Eq, PartialEq, Hash, Clone, Debug, Copy)]
/// [`Event`] for player interactions with game world entities
pub enum InterAction {
    /// Grab copy of entity from inventory
    Pipette,
    /// Placing down entity
    Construct,
    /// Destroy entity
    Deconstruct,
    /// Provide item for entity
    Distribute,
    /// Copy entity attributes
    CopyConfiguration,
}

pub(crate) fn dispatch_inter_actions(
    movement_actions_event: EventWriter<InterAction>,
    input_mappings: Res<InputMappings>,
) {
}
