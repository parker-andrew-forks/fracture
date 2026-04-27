struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    // out.full_frame = model.full_frame;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

struct UiRenderData {
    flagged: u32,
    transparency: f32,
    mouse_y: u32,
    mouse_x: u32,
    surface_height: u32,
    surface_width: u32,
    select_y: u32,
    select_x: u32,
    surface_start_x: u32,
    surface_start_y: u32,
    surface_end_x: u32,
    surface_end_y: u32,
    gs_r: f32,
    gs_g: f32,
    gs_b: f32,
    gs_sensitivity: f32,
    time: f32,
 }

const DisplayOverlays: u32 = 1;
const MouseOverWindow: u32 = 2;
const MouseDown: u32 = 4;
const WaitingForCrop: u32 = 8;
const OnlyAngles: u32 = 16;
const KeepBorders: u32 = 32;
const UseGreenScreen: u32 = 64;

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
@group(0) @binding(2)
var overlay: texture_2d<f32>;

@group(0) @binding(3)
var<uniform> flags: UiRenderData;

fn use_set_transparency(in_color: vec4<f32>) -> vec4<f32> {
    var color = in_color;

    color.a = color.a - (1.0 - flags.transparency);

    return color;
}


fn use_remove_green_screen_colors(in_color: vec4<f32>) -> vec4<f32> {
    var color = in_color;

    if (flags.flagged & UseGreenScreen) != 0 {
        var sense: f32 = (flags.gs_sensitivity / 100.0) + 0.01;

        let compare_col = vec3(flags.gs_r, flags.gs_g, flags.gs_b);

        if (max(compare_col.r, color.r) - min(compare_col.r, color.r)) < sense {
            if (max(compare_col.g, color.g) - min(compare_col.g, color.g)) < sense {
                if (max(compare_col.b, color.b) - min(compare_col.b, color.b)) < sense {
                    color.a = 0.0;
                    color.r = 0.0;
                    color.g = 0.0;
                    color.b = 0.0;
                }
            }
        }
    }

    return color;
}

fn preprocessing(in_color: vec4<f32>) -> vec4<f32> {
    var color = in_color;

    color = use_set_transparency(color);
    color = use_remove_green_screen_colors(color);

    return color;
}

/// When rendering from rgb instead of rgba the alpha is dropped. This adds
/// the alpha back if it is missing.
fn alpha_sample(position: vec2<f32>) -> vec4<f32> {
     var result = textureSample(t_diffuse, s_diffuse, position);
    
        // todo: Add a flag that states rgb vs. rgba, etc. so these checks
        // don't have to be completed
        if result.a > 0.0 {
            return result;
        }

        if u32(position.x * f32(flags.surface_width)) > flags.surface_start_x  
                &&  u32(position.y * f32(flags.surface_height)) > flags.surface_start_y 
                &&  u32(position.x * f32(flags.surface_width)) < flags.surface_end_x 
                &&  u32(position.y * f32(flags.surface_height)) < flags.surface_end_y  {
            
            if (result.r != 0.0 || result.g != 0.0 || result.b !=0) && result.a == 0.0 {
                result.a = 1.0;
            } 
        }
    

     return result;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = alpha_sample(in.tex_coords);
    var current_position = in.tex_coords;

    color = preprocessing(color);

    // INJECT HERE 1

    return color;
}


// POSTPROCESSOR BELOW THIS LINE
// INJECT HERE 2