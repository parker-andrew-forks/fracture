use super::{
    defaults::{ALPHA_PREFERENCES, PRESENT_PREFERENCES, TEXTURE_PREFERENCES},
    utility_vertex::Vertex,
};
use std::mem;
use wgpu::{Device, PipelineLayout, RenderPipeline, ShaderModule};

pub fn define_primary_sampler(
    device: &Device,
    mag: wgpu::FilterMode,
    min: wgpu::FilterMode,
) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: mag,
        min_filter: min,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    })
}

pub struct SelectedGpuCaps {
    pub present: wgpu::PresentMode,
    pub texture: wgpu::TextureFormat,
    pub alpha: wgpu::CompositeAlphaMode,
}

pub fn select_caps_with_preferences(
    gpu_capabilities: wgpu::SurfaceCapabilities,
    predicted_pipewire_format: wgpu::TextureFormat,
) -> SelectedGpuCaps {
    let alpha = ALPHA_PREFERENCES
        .iter()
        .find(|v| gpu_capabilities.alpha_modes.contains(&v))
        .map(|v| v.clone())
        .unwrap_or(gpu_capabilities.alpha_modes[0].clone());

    let texture_fmt = if gpu_capabilities
        .formats
        .contains(&predicted_pipewire_format)
    {
        predicted_pipewire_format.clone()
    } else {
        TEXTURE_PREFERENCES
            .iter()
            .find(|v| gpu_capabilities.formats.contains(&v))
            .map(|v| v.clone())
            .unwrap_or(gpu_capabilities.formats[0].clone())
    };

    let present = PRESENT_PREFERENCES
        .iter()
        .find(|v| gpu_capabilities.present_modes.contains(&v))
        .map(|v| v.clone())
        .unwrap_or(gpu_capabilities.present_modes[0].clone());

    SelectedGpuCaps {
        present,
        texture: texture_fmt,
        alpha,
    }
}

pub fn define_primary_pipeline(
    device: &Device,
    module: &ShaderModule,
    layout: &PipelineLayout,
    config_texture_format: &wgpu::TextureFormat,
) -> RenderPipeline {
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs_main"), // 1.
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x3,
                    },
                    wgpu::VertexAttribute {
                        offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                        shader_location: 1,
                        format: wgpu::VertexFormat::Float32x2,
                    },
                ],
            }],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config_texture_format.clone(),
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),

        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },

        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview_mask: None,
        cache: None,
    });

    render_pipeline
}
