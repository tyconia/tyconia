use crate::actions::*;
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
    /// Paste entity attributes
    PasteConfiguration,
}

impl InputAction for InterAction {
    fn display(&self) -> String {
        match *self {
            Self::Pipette => "Grab entity",
            Self::Construct => "Build entity",
            Self::Deconstruct => "Destroy entity",
            Self::Distribute => "Give item to entity",
            Self::CopyConfiguration => "Copy configuration from entity",
            Self::PasteConfiguration => "Paste configuration from entity",
        }
        .into()
    }

    fn desktop_mapping(&self, input_mapping: &Res<InputMappings>) -> Option<InputMappingEntry> {
        input_mapping.inter_actions.get(&*self).cloned()
    }
}

pub(crate) fn dispatch_inter_actions(
    movement_actions_event: EventWriter<InterAction>,
    input_mappings: Res<InputMappings>,
) {
}
