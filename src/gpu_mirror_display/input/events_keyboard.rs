use crate::gpu_mirror_display::{
    state::{AdditionalRenderingState, State},
    window_cropping::{CropEndTriggeredFrom, if_in_crop_complete_crop},
};
use winit::{event::WindowEvent, keyboard::KeyCode};

pub(crate) fn on_keyboard_events(
    event: &WindowEvent,
    state: &State,
    additional_state: &mut AdditionalRenderingState,
) {
    match event {
        WindowEvent::KeyboardInput {
            device_id: _,
            event,
            is_synthetic: _,
        } => match event.physical_key {
            winit::keyboard::PhysicalKey::Code(key_code) => match key_code {
                KeyCode::NumpadEnter | KeyCode::Enter => {
                    if_in_crop_complete_crop(
                        additional_state,
                        &state,
                        CropEndTriggeredFrom::EnterPress,
                    );
                }
                KeyCode::Escape => {
                    // todo: Should something happen on escape?
                }
                _ => {}
            },
            _ => {}
        },
        _ => {}
    }
}
