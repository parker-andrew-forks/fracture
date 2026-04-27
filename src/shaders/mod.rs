pub const SHADER_INVERT_COLORS: &'static str =
    include_str!("./examples/invert_colors.wgsl.snippet");

pub const SHADER_FLIP_HORIZONTAL: &'static str =
    include_str!("./examples/flip_horizontal.wgsl.snippet");

pub const SHADER_FLIP_VERTICAL: &'static str =
    include_str!("./examples/flip_vertical.wgsl.snippet");

pub const SHADER_COLOR_GRADIENT: &'static str =
    include_str!("./examples/shifting_color_gradient.wgsl.snippet");

pub const SHADER_ROTATE_LEFT: &'static str = include_str!("./examples/rotate_left.wgsl.snippet");

pub const SHADER_SHOW_ALL_INPUTS: &'static str = concat!(
    include_str!("./examples/show_inputs.wgsl.snippet"),
    "\r\n\r\n",
    "/*",
    "\r\n",
    include_str!("./mirror.wgsl"),
    "\r\n",
    "*/"
);
