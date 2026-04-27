pub const APPLICATION_NAME: &'static str = "FRACTURE";

#[cfg(feature = "flatpak")]
pub const FLATPAK_ID: &'static str = "systems.fracture.launcher";

pub const CROP_COLOR: (f32, f32, f32, f32) = (0.2, 0.5, 0.8, 0.9);

pub const SELECTION_WINDOW_OFFSETS: (u32, u32) = (60, 60);

pub const ALPHA_PREFERENCES: [wgpu::CompositeAlphaMode; 5] = [
    wgpu::CompositeAlphaMode::PreMultiplied,
    wgpu::CompositeAlphaMode::Inherit,
    wgpu::CompositeAlphaMode::PostMultiplied,
    wgpu::CompositeAlphaMode::Opaque,
    wgpu::CompositeAlphaMode::Auto,
];

pub const TEXTURE_PREFERENCES: [wgpu::TextureFormat; 3] = [
    wgpu::TextureFormat::Rgba8Unorm,
    wgpu::TextureFormat::Bgra8Unorm,
    wgpu::TextureFormat::NV12,
];

pub const PRESENT_PREFERENCES: [wgpu::PresentMode; 6] = [
    wgpu::PresentMode::AutoVsync,
    wgpu::PresentMode::Fifo,
    wgpu::PresentMode::FifoRelaxed,
    wgpu::PresentMode::Mailbox,
    wgpu::PresentMode::Immediate,
    wgpu::PresentMode::AutoNoVsync,
];
