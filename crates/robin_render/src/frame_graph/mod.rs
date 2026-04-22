mod index;
mod pass;
mod pass_node;
mod pipeline_container;
mod resource_node;
mod resource_table;
mod texture_view;
mod transient_resource;

use bevy::render::renderer::RenderDevice;

pub use index::*;
pub use pass::*;
pub use pass_node::*;
pub use pipeline_container::*;
pub use resource_node::*;
pub use resource_table::*;
pub use texture_view::*;
pub use transient_resource::*;

pub trait TransientResourceCreator {
    fn create_resource(&self, desc: &AnyTransientResourceDescriptor) -> AnyTransientResource;
}

impl TransientResourceCreator for RenderDevice {
    fn create_resource(&self, desc: &AnyTransientResourceDescriptor) -> AnyTransientResource {
        match desc {
            AnyTransientResourceDescriptor::Buffer(desc) => {
                AnyTransientResource::OwnedBuffer(TransientBuffer {
                    resource: self.wgpu_device().create_buffer(&desc.get_desc()),
                    desc: desc.clone(),
                })
            }
            AnyTransientResourceDescriptor::Texture(desc) => {
                AnyTransientResource::OwnedTexture(TransientTexture {
                    resource: self.wgpu_device().create_texture(&desc.get_desc()),
                    desc: desc.clone(),
                })
            }
        }
    }
}
