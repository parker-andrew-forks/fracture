use crate::{
    global_application_state::LastReported,
    ui_state::{
        ScaleDecision, TitleBarDisplay, UiState, VideoAspect, VideoLocation, WindowBackground,
        WindowBehaviour,
    },
};
use std::sync::Arc;
use winit::dpi::PhysicalSize;

use super::{
    defaults::{CROP_COLOR, SELECTION_WINDOW_OFFSETS},
    state::{AdditionalRenderingState, State},
};

pub fn start_crop_selection(additional_state: &mut AdditionalRenderingState, state: &mut State) {
    state.window.set_maximized(false);
    state.window.set_minimized(false);

    additional_state.crop_button_pressed = true;

    additional_state.shutdown_settings_ui();

    additional_state.settings_state = UiState {
        display_title: TitleBarDisplay::TitleBarVisible,
        aspect_ratio: VideoAspect::MaintainAspectRatio(
            ScaleDecision::DontScale,
            WindowBehaviour::SizeSetByUser(VideoLocation::Center),
        ),
        frame_transparency: 100.0,
        need_rebuild: true,
        updated: true,

        green_screen: crate::ui_state::GreenScreen::None,
        postprocessor: Default::default(),
        background: WindowBackground::Color(CROP_COLOR.0, CROP_COLOR.1, CROP_COLOR.2, CROP_COLOR.3),
        ..Default::default()
    };

    additional_state
        .channels
        .gpu_sender_request
        .send(additional_state.settings_state.clone())
        .unwrap();

    additional_state.cropped = Some(CroppedArea {
        relative_to_window_position: InitialAbsoluteWindowPosition { x: 0, y: 0 },
        size: Size {
            width: additional_state.last_frame_size.0,
            height: additional_state.last_frame_size.1,
        },
        relative_to_frame_position: InitialAbsoluteFramePosition {
            x: state.last_reported_offsets.0,
            y: state.last_reported_offsets.1,
        },
    });

    state
        .window
        .request_inner_size(PhysicalSize {
            width: additional_state.cropped.as_ref().unwrap().size.width
                + SELECTION_WINDOW_OFFSETS.0,
            height: additional_state.cropped.as_ref().unwrap().size.height
                + SELECTION_WINDOW_OFFSETS.1,
        })
        .unwrap();
}

pub fn if_crop_button_is_active(
    state: &State,
    crop_button_press: &bool,
    frame: &Arc<LastReported>,
    additional_state: &mut AdditionalRenderingState,
) {
    let cropped_button_press: bool = *crop_button_press;

    if cropped_button_press {
        if state.window.is_maximized() {
            state.window.set_maximized(false);
        }

        let PhysicalSize { width, height } = state.window.inner_size();
        let (f_w, f_h) = frame.window_dimensions;
        let (off_w, off_h) = SELECTION_WINDOW_OFFSETS;

        if f_w + off_w != width || f_h + off_h != height {
            additional_state.cropped = Some(CroppedArea {
                relative_to_window_position: InitialAbsoluteWindowPosition { x: 0, y: 0 },
                size: Size {
                    width: frame.window_dimensions.0,
                    height: frame.window_dimensions.1,
                },
                relative_to_frame_position: InitialAbsoluteFramePosition {
                    x: state.last_reported_offsets.0,
                    y: state.last_reported_offsets.1,
                },
            });

            state
                .window
                .request_inner_size(PhysicalSize {
                    width: f_w + off_w,
                    height: f_h + off_h,
                })
                .unwrap();
        }
    }
}
#[derive(Debug, PartialEq)]
pub enum CropEndTriggeredFrom {
    MouseUp,
    EnterPress,
}

#[derive(Clone, Debug)]
pub struct InitialAbsoluteWindowPosition {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug)]
pub struct InitialAbsoluteFramePosition {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug)]
pub struct CroppedArea {
    /// The x and y values are from mouse clicks on the window that is created
    /// by the application. This window size changes over time.
    pub relative_to_window_position: InitialAbsoluteWindowPosition,
    pub relative_to_frame_position: InitialAbsoluteFramePosition,
    /// A cro
    pub size: Size,
}

pub fn if_in_crop_complete_crop(
    additional_state: &mut AdditionalRenderingState,
    state: &State,
    from: CropEndTriggeredFrom,
) {
    let PhysicalSize { width, height } = state.window.inner_size();

    if additional_state.in_crop_selection
        || additional_state.crop_button_pressed && (from == CropEndTriggeredFrom::EnterPress)
    {
        let (min_x, max_x) = {
            match from {
                CropEndTriggeredFrom::MouseUp => (
                    additional_state
                        .last_known_mouse_position
                        .0
                        .min(additional_state.mouse_select_start.0),
                    additional_state
                        .last_known_mouse_position
                        .0
                        .max(additional_state.mouse_select_start.0),
                ),
                CropEndTriggeredFrom::EnterPress => (0, width),
            }
        };

        let (min_y, max_y) = {
            match from {
                CropEndTriggeredFrom::MouseUp => (
                    additional_state
                        .last_known_mouse_position
                        .1
                        .min(additional_state.mouse_select_start.1),
                    additional_state
                        .last_known_mouse_position
                        .1
                        .max(additional_state.mouse_select_start.1),
                ),
                CropEndTriggeredFrom::EnterPress => (0, height),
            }
        };

        let min_x: i32 = (min_x as i32 - ((SELECTION_WINDOW_OFFSETS.0 / 2) as i32)).max(0);
        let min_y: i32 = (min_y as i32 - ((SELECTION_WINDOW_OFFSETS.1 / 2) as i32)).max(0);

        let max_x: i32 = (max_x as i32 - ((SELECTION_WINDOW_OFFSETS.0 / 2) as i32)).max(0);
        let max_y: i32 = (max_y as i32 - ((SELECTION_WINDOW_OFFSETS.1 / 2) as i32)).max(0);

        let pos = (min_x as u32, min_y as u32);

        let window_position = InitialAbsoluteWindowPosition { x: pos.0, y: pos.1 };

        additional_state.cropped = Some(CroppedArea {
            relative_to_frame_position: InitialAbsoluteFramePosition {
                x: window_position.x + state.last_reported_offsets.0,
                y: window_position.y + state.last_reported_offsets.1,
            },
            relative_to_window_position: window_position,
            size: Size {
                width: ((max_x as i32 - min_x as i32) + 1 as i32)
                    .min(additional_state.last_frame_size.0 as i32 - pos.0 as i32)
                    .max(0) as u32,
                height: ((max_y as i32 - min_y as i32) + 1 as i32)
                    .min(additional_state.last_frame_size.1 as i32 - pos.1 as i32)
                    .max(0) as u32,
            },
        });

        if additional_state.cropped.as_ref().unwrap().size.width <= 5
            || additional_state.cropped.as_ref().unwrap().size.height <= 5
        {
            // This is kinda a hack to force small selections and entirely offscreen selections to fullscreen. I'm doubtful it will always work,
            // and it seems very likely to crash. I think the section of code that crops and repositions the frame
            // has an off by 1 error that will core dump on very small selections (like 1-5 pixels). There's likely still
            // other unfound crashes with resizing because of it, but the desired behaviour is for very small
            // crops (and entirely offscreen crops) to snap to the entire window size.
            //
            // I'm not fixing it at this time because I want to focus on more important issues before spending time trying to determine
            // what is wrong with it. If this is commented out, (The if statement is removed, and the else is left to be called always)
            // then a 1 pixel selection is made, the crash will happen.
            if_in_crop_complete_crop(additional_state, state, CropEndTriggeredFrom::EnterPress);
        } else {
            additional_state.settings_state = UiState {
                display_title: TitleBarDisplay::HiddenTitleBar,
                aspect_ratio: VideoAspect::MaintainAspectRatio(
                    ScaleDecision::Scale,
                    WindowBehaviour::SizeMatchesMirrorAspect,
                ),
                frame_transparency: 100.0,
                need_rebuild: true,
                updated: true,
                green_screen: crate::ui_state::GreenScreen::None,
                postprocessor: Default::default(),
                ..Default::default()
            };

            additional_state.new_settings = true;

            state
                .window
                .request_inner_size(PhysicalSize {
                    width: additional_state.cropped.as_ref().unwrap().size.width,
                    height: additional_state.cropped.as_ref().unwrap().size.height,
                })
                .unwrap();

            additional_state
                .channels
                .gpu_sender_request
                .send(additional_state.settings_state.clone())
                .unwrap();
        }

        additional_state.in_crop_selection = false;
        additional_state.crop_button_pressed = false;
    }
}
