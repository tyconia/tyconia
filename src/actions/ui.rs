use crate::{
    actions::{mappings::InputMappings, *},
    GameState,
};
use bevy::prelude::*;

#[derive(Event, Reflect, Eq, PartialEq, Hash, Clone, Debug, Copy)]
/// [`Event`] for changing UI elements
pub enum UiAction {
    /// Change camera zoom
    Zoom(isize),
    /// Select hotbar slot
    HotbarSlot(usize),
    /// Next hotbar slot
    HotbarSlotNext,
    /// Previous hotbar slot
    HotbarSlotPrevious,
    /// Switch hotbar
    Hotbar(usize),
    /// Summon pause menu
    Menu,
}

impl UiAction {
    /// gives zoom factor if it is a zoom, useful for filters
    pub fn zoom(&self) -> Option<isize> {
        if let Self::Zoom(factor) = *self {
            Some(factor)
        } else {
            None
        }
    }
}

pub(crate) fn dispatch_ui_actions(
    mut ui_actions_events: EventWriter<UiAction>,
    input_mappings: Res<InputMappings>,

    mut next_state: ResMut<NextState<GameState>>,
    mouse_scroll: MouseScrollEvent,
    mouse_button: MouseButtonResource,
    key_button: KeyButtonResource,
) {
    let ctrl_incoming = (&mouse_button, &key_button, &mouse_scroll);
    input_mappings
        .ui_actions
        .iter()
        .for_each(|(actions, entry)| match actions {
            UiAction::Menu => {
                if entry.just_pressed(UiAction::Menu, ctrl_incoming).is_some() {
                    next_state.set(GameState::Menu);
                }
            }
            _ => {}
        });
}
