use bevy::render::render_resource::{Buffer, BufferId, Texture, TextureId};

use crate::frame_graph::{
    FrameGraph, ResourceHandle, TransientBuffer, TransientBufferDescriptor, TransientResource,
    TransientTexture, TransientTextureDescriptor,
};

pub trait ResourceMaterial {
    type ResourceType: TransientResource;

    fn imported(&self, frame_graph: &mut FrameGraph) -> ResourceHandle<Self::ResourceType>;
}

pub fn create_buffer_key(id: BufferId) -> String {
    format!("buffer_{:?}", id)
}

impl ResourceMaterial for Buffer {
    type ResourceType = TransientBuffer;

    fn imported(&self, frame_graph: &mut FrameGraph) -> ResourceHandle<Self::ResourceType> {
        let key = create_buffer_key(self.id());

        frame_graph.get_or_create(&key, TransientBufferDescriptor::External)
    }
}

pub fn create_texture_key(id: TextureId) -> String {
    format!("texture_{:?}", id)
}

impl ResourceMaterial for Texture {
    type ResourceType = TransientTexture;

    fn imported(&self, frame_graph: &mut FrameGraph) -> ResourceHandle<Self::ResourceType> {
        let key = create_texture_key(self.id());
        frame_graph.get_or_create(&key, TransientTextureDescriptor::External)
    }
}
