use bevy_ecs::resource::Resource;

use crate::render_resource::WgpuWrapper;

#[derive(Resource, Clone)]
pub struct RenderDevice {
    device: WgpuWrapper<wgpu::Device>,
}
