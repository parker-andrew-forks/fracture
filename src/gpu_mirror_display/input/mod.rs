use super::state::{AdditionalRenderingState, State};
use events_keyboard::on_keyboard_events;
use events_mouse::on_mouse_events;
use winit::event::WindowEvent;

pub(crate) mod events_keyboard;
pub(crate) mod events_mouse;
pub(crate) mod utility_mouse;

pub(crate) fn on_input_events(
    event: &WindowEvent,
    state: &State,
    additional_state: &mut AdditionalRenderingState,
) {
    on_mouse_events(event, state, additional_state);
    on_keyboard_events(event, state, additional_state);
}
