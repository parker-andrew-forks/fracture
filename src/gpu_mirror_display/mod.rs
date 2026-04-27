pub mod binary_images;
pub mod defaults;
pub mod event_loop;
pub mod input;
pub mod overlay_ui;
pub mod pipeline_definitions;
pub mod postprocessing_shaders;
pub mod render;
pub mod shutdown;
pub mod state;
pub mod utility_texture;
pub mod utility_vertex;
pub mod window_cropping;
pub mod window_resizing;

static START_TIME: std::sync::LazyLock<std::time::Instant> =
    std::sync::LazyLock::new(|| std::time::Instant::now());
