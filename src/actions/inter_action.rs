use crate::actions::*;

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
    mut inter_actions_event: EventWriter<InterAction>,
    input_mappings: Res<InputMappings>,
    mouse_scroll: MouseScrollEvent,
    mouse_button: MouseButtonResource,
    key_button: KeyButtonResource,
) {
    let ctrl_incoming = (&mouse_button, &key_button, &mouse_scroll);

    input_mappings
        .inter_actions
        .iter()
        .for_each(|(actions, entry)| match actions {
            InterAction::Pipette => {
                if entry
                    .just_pressed(InterAction::Pipette, ctrl_incoming)
                    .is_some()
                {
                    inter_actions_event.send(InterAction::Pipette);
                }
            }
            InterAction::Construct => {
                if entry
                    .just_pressed(InterAction::Construct, ctrl_incoming)
                    .is_some()
                {
                    inter_actions_event.send(InterAction::Construct);
                }
            }
            InterAction::Deconstruct => {
                if entry
                    .just_pressed(InterAction::Deconstruct, ctrl_incoming)
                    .is_some()
                {
                    inter_actions_event.send(InterAction::Deconstruct);
                }
            }
            InterAction::Distribute => {
                if entry
                    .just_pressed(InterAction::Distribute, ctrl_incoming)
                    .is_some()
                {
                    inter_actions_event.send(InterAction::Distribute);
                }
            }
            InterAction::CopyConfiguration => {
                if entry
                    .just_pressed(InterAction::CopyConfiguration, ctrl_incoming)
                    .is_some()
                {
                    inter_actions_event.send(InterAction::CopyConfiguration);
                }
            }
            InterAction::PasteConfiguration => {
                if entry
                    .just_pressed(InterAction::PasteConfiguration, ctrl_incoming)
                    .is_some()
                {
                    inter_actions_event.send(InterAction::PasteConfiguration);
                }
            }
        });
}
