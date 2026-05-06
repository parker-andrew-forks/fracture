use crate::{
    gpu_mirror_display::{
        state::{AdditionalRenderingState, State},
        window_cropping::{CropEndTriggeredFrom, if_in_crop_complete_crop},
    },
    ui_state::{TitleBarDisplay, VideoAspect, WindowBehaviour},
};
use std::time::SystemTime;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::WindowEvent,
};

#[derive(PartialEq, Eq)]
pub enum ResizeInteractionsState {
    None,
    NwResize,
    SwResize,
    NeResize,
    SeResize,
    North,
    South,
    East,
    West,
}

pub(crate) fn on_mouse_events(
    event: &WindowEvent,
    state: &State,
    additional_state: &mut AdditionalRenderingState,
) {
    match event {
        WindowEvent::MouseInput {
            device_id: _,
            state: state2,
            button: btn,
        } => match state2 {
            winit::event::ElementState::Pressed => match btn {
                winit::event::MouseButton::Left => {
                    additional_state.keep_borders = true;

                    match additional_state.mouse_resize_state {
                        ResizeInteractionsState::None => {
                            additional_state.keep_borders = false;
                            additional_state.mouse_downs.push((
                                (additional_state.last_known_mouse_position),
                                SystemTime::now(),
                            ));

                            if additional_state.crop_button_pressed {
                                additional_state.in_crop_selection = true;
                            }

                            additional_state.mouse_is_down = true;
                            additional_state.mouse_select_start =
                                additional_state.last_known_mouse_position;
                        }
                        ResizeInteractionsState::NwResize => {
                            let _ = state
                                .window
                                .drag_resize_window(winit::window::ResizeDirection::NorthWest);
                        }
                        ResizeInteractionsState::SwResize => {
                            let _ = state
                                .window
                                .drag_resize_window(winit::window::ResizeDirection::SouthWest);
                        }
                        ResizeInteractionsState::NeResize => {
                            let _ = state
                                .window
                                .drag_resize_window(winit::window::ResizeDirection::NorthEast);
                        }
                        ResizeInteractionsState::SeResize => {
                            let _ = state
                                .window
                                .drag_resize_window(winit::window::ResizeDirection::SouthEast);
                        }
                        ResizeInteractionsState::North => {
                            let _ = state
                                .window
                                .drag_resize_window(winit::window::ResizeDirection::North);
                        }
                        ResizeInteractionsState::South => {
                            let _ = state
                                .window
                                .drag_resize_window(winit::window::ResizeDirection::South);
                        }
                        ResizeInteractionsState::East => {
                            let _ = state
                                .window
                                .drag_resize_window(winit::window::ResizeDirection::East);
                        }
                        ResizeInteractionsState::West => {
                            let _ = state
                                .window
                                .drag_resize_window(winit::window::ResizeDirection::West);
                        }
                    }
                }
                winit::event::MouseButton::Right => {
                    state.window.show_window_menu(PhysicalPosition {
                        x: additional_state.last_known_mouse_position.0,
                        y: additional_state.last_known_mouse_position.1,
                    });
                }

                _ => {}
            },
            winit::event::ElementState::Released => match btn {
                winit::event::MouseButton::Left => {
                    additional_state.mouse_clicks.push((
                        (additional_state.last_known_mouse_position),
                        SystemTime::now(),
                    ));

                    additional_state.mouse_is_down = false;

                    if_in_crop_complete_crop(
                        additional_state,
                        &state,
                        CropEndTriggeredFrom::MouseUp,
                    );
                }
                _ => {}
            },
        },

        WindowEvent::CursorMoved {
            device_id: _,
            position,
        } => {
            on_cursor_movements(&state, additional_state, position);
        }

        WindowEvent::CursorEntered { device_id: _ } => {
            additional_state.mouse_over_screen = true;
        }
        WindowEvent::CursorLeft { device_id: _ } => {
            additional_state.mouse_over_screen = false;
            {
                additional_state.mouse_is_down = false;
            }
        }
        _ => {}
    }
}

fn on_cursor_movements(
    state: &State,
    additional_state: &mut AdditionalRenderingState,
    position: &PhysicalPosition<f64>,
) {
    additional_state.keep_borders = false;

    additional_state.last_known_mouse_position =
        (position.x.round() as u32, position.y.round() as u32);

    let PhysicalSize { width, height } = state.window.inner_size();

    let (x, y) = *&additional_state.last_known_mouse_position;

    let resize = 10;

    if additional_state.settings_state.display_title == TitleBarDisplay::HiddenTitleBar {
        if x < resize && y < resize {
            state.window.set_cursor(winit::window::CursorIcon::NwResize);
            additional_state.mouse_resize_state = ResizeInteractionsState::NwResize;
        } else if x < resize && y > height - resize {
            state.window.set_cursor(winit::window::CursorIcon::SwResize);
            additional_state.mouse_resize_state = ResizeInteractionsState::SwResize;
        } else if y < resize && x > width - resize {
            state.window.set_cursor(winit::window::CursorIcon::NeResize);
            additional_state.mouse_resize_state = ResizeInteractionsState::NeResize;
        } else if y > height - resize && x > width - resize {
            state.window.set_cursor(winit::window::CursorIcon::SeResize);
            additional_state.mouse_resize_state = ResizeInteractionsState::SeResize;
        } else {
            if let VideoAspect::MaintainAspectRatio(_, WindowBehaviour::SizeMatchesMirrorAspect) =
                additional_state.settings_state.aspect_ratio
            {
                state.window.set_cursor(winit::window::CursorIcon::Default);
                additional_state.mouse_resize_state = ResizeInteractionsState::None;
            } else {
                if x < resize || x > width - resize || y < resize || y > height - resize {
                    if x < resize || x > width - resize {
                        state
                            .window
                            .set_cursor(winit::window::CursorIcon::ColResize);

                        if x < resize {
                            additional_state.mouse_resize_state = ResizeInteractionsState::West;
                        } else {
                            additional_state.mouse_resize_state = ResizeInteractionsState::East;
                        }
                    } else {
                        state
                            .window
                            .set_cursor(winit::window::CursorIcon::RowResize);

                        if y < resize {
                            additional_state.mouse_resize_state = ResizeInteractionsState::North;
                        } else {
                            additional_state.mouse_resize_state = ResizeInteractionsState::South;
                        }
                    }
                } else {
                    state.window.set_cursor(winit::window::CursorIcon::Default);
                    additional_state.mouse_resize_state = ResizeInteractionsState::None;
                }
            }
        }
    } else {
        state.window.set_cursor(winit::window::CursorIcon::Default);
        additional_state.mouse_resize_state = ResizeInteractionsState::None;
    }
}
