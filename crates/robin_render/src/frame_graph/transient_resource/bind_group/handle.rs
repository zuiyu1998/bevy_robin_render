use core::num::NonZero;

use wgpu::{BindGroupLayout, Sampler};
use variadics_please::all_tuples_with_size;

use crate::frame_graph::{
    PassNodeBuilderExt, ResourceHandle, TransientBindGroup, TransientBindGroupBuffer,
    TransientBindGroupEntry, TransientBindGroupResource, TransientBindGroupTextureView,
    TransientBuffer, TransientTexture, TransientTextureViewDescriptor,
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TransientBindGroupTextureViewHandle {
    pub texture: ResourceHandle<TransientTexture>,
    pub texture_view_desc: TransientTextureViewDescriptor,
}

impl TransientBindGroupTextureViewHandle {
    pub fn create<T: PassNodeBuilderExt>(
        &self,
        pass_node_builder: &mut T,
    ) -> TransientBindGroupTextureView {
        let texture = pass_node_builder.read(self.texture.clone());

        TransientBindGroupTextureView {
            texture,
            texture_view_desc: self.texture_view_desc.clone(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TransientBindGroupBufferHandle {
    pub buffer: ResourceHandle<TransientBuffer>,
    pub size: Option<NonZero<u64>>,
    pub offset: u64,
}

impl TransientBindGroupBufferHandle {
    pub fn create<T: PassNodeBuilderExt>(
        &self,
        pass_node_builder: &mut T,
    ) -> TransientBindGroupBuffer {
        let buffer = pass_node_builder.read(self.buffer.clone());

        TransientBindGroupBuffer {
            buffer,
            size: self.size,
            offset: self.offset,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TransientBindGroupResourceHandle {
    Buffer(TransientBindGroupBufferHandle),
    Sampler(Sampler),
    TextureView(TransientBindGroupTextureViewHandle),
    TextureViewArray(Vec<TransientBindGroupTextureViewHandle>),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TransientBindGroupEntryHandle {
    pub binding: u32,
    pub resource: TransientBindGroupResourceHandle,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TransientBindGroupHandle {
    pub label: Option<String>,
    pub layout: BindGroupLayout,
    pub entries: Vec<TransientBindGroupEntryHandle>,
}

impl TransientBindGroupHandle {
    pub fn create<T: PassNodeBuilderExt>(&self, pass_builder: &mut T) -> TransientBindGroup {
        let entries = self
            .entries
            .iter()
            .map(|handle| {
                let resource = match &handle.resource {
                    TransientBindGroupResourceHandle::Buffer(buffer) => {
                        TransientBindGroupResource::Buffer(buffer.create(pass_builder))
                    }
                    TransientBindGroupResourceHandle::Sampler(sampler) => {
                        TransientBindGroupResource::Sampler(sampler.clone())
                    }
                    TransientBindGroupResourceHandle::TextureView(texture_view) => {
                        TransientBindGroupResource::TextureView(texture_view.create(pass_builder))
                    }
                    TransientBindGroupResourceHandle::TextureViewArray(texture_view_array) => {
                        let texture_view_array = texture_view_array
                            .iter()
                            .map(|texture_view| texture_view.create(pass_builder))
                            .collect();

                        TransientBindGroupResource::TextureViewArray(texture_view_array)
                    }
                };

                TransientBindGroupEntry {
                    binding: handle.binding,
                    resource,
                }
            })
            .collect();

        TransientBindGroup {
            label: self.label.clone(),
            layout: self.layout.clone(),
            entries,
        }
    }

    pub fn build(layout: &BindGroupLayout) -> TransientBindGroupHandleBuilder {
        TransientBindGroupHandleBuilder::new(layout)
    }
}

pub trait IntoTransientBindGroupResourceHandle {
    fn into_handle(self) -> TransientBindGroupResourceHandle;
}

impl IntoTransientBindGroupResourceHandle for TransientBindGroupBufferHandle {
    fn into_handle(self) -> TransientBindGroupResourceHandle {
        TransientBindGroupResourceHandle::Buffer(self)
    }
}

impl IntoTransientBindGroupResourceHandle for Sampler {
    fn into_handle(self) -> TransientBindGroupResourceHandle {
        TransientBindGroupResourceHandle::Sampler(self)
    }
}

impl IntoTransientBindGroupResourceHandle for TransientBindGroupTextureViewHandle {
    fn into_handle(self) -> TransientBindGroupResourceHandle {
        TransientBindGroupResourceHandle::TextureView(self)
    }
}

impl IntoTransientBindGroupResourceHandle for Vec<TransientBindGroupTextureViewHandle> {
    fn into_handle(self) -> TransientBindGroupResourceHandle {
        TransientBindGroupResourceHandle::TextureViewArray(self)
    }
}

impl<T: Clone + IntoTransientBindGroupResourceHandle> IntoTransientBindGroupResourceHandle for &T {
    fn into_handle(self) -> TransientBindGroupResourceHandle {
        self.clone().into_handle()
    }
}

pub struct TransientBindGroupHandleBuilder {
    label: Option<String>,
    layout: BindGroupLayout,
    entries: Vec<TransientBindGroupEntryHandle>,
}

impl TransientBindGroupHandleBuilder {
    pub fn new(layout: &BindGroupLayout) -> Self {
        Self {
            label: None,
            layout: layout.clone(),
            entries: vec![],
        }
    }

    pub fn set_entries(mut self, entries: &[TransientBindGroupEntryHandle]) -> Self {
        self.entries = entries.to_vec();

        self
    }

    pub fn push<T: IntoTransientBindGroupResourceHandle>(mut self, value: T) -> Self {
        let handle = value.into_handle();

        self.entries.push(TransientBindGroupEntryHandle {
            resource: handle,
            binding: self.entries.len() as u32,
        });

        self
    }

    pub fn set_label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());

        self
    }

    pub fn finished(self) -> TransientBindGroupHandle {
        TransientBindGroupHandle {
            label: self.label,
            layout: self.layout,
            entries: self.entries,
        }
    }
}

pub trait IntoTransientBindGroupEntryHandleArray<const N: usize> {
    fn into_array(self) -> [TransientBindGroupResourceHandle; N];
}

macro_rules! impl_to_transient_bind_group_entry_handle_slice {
    ($N: expr, $(#[$meta:meta])* $(($T: ident, $I: ident)),*) => {
        $(#[$meta])*
        impl<$($T: IntoTransientBindGroupResourceHandle),*> IntoTransientBindGroupEntryHandleArray<$N> for ($($T,)*) {
            #[inline]
            fn into_array(self) -> [TransientBindGroupResourceHandle; $N] {
                let ($($I,)*) = self;
                [$($I.into_handle(), )*]
            }
        }
    }
}

all_tuples_with_size!(
    #[doc(fake_variadic)]
    impl_to_transient_bind_group_entry_handle_slice,
    1,
    32,
    T,
    s
);

pub trait IntoIndexedTransientBindGroupResourceHandleArray<const N: usize> {
    fn into_array(self) -> [(u32, TransientBindGroupResourceHandle); N];
}

pub struct BindGroupEntryHandles<const N: usize = 1> {
    entries: [TransientBindGroupEntryHandle; N],
}

impl<const N: usize> BindGroupEntryHandles<N> {
    #[inline]
    pub fn sequential(resources: impl IntoTransientBindGroupEntryHandleArray<N>) -> Self {
        let mut i = 0;
        Self {
            entries: resources.into_array().map(|resource| {
                let binding = i;
                i += 1;
                TransientBindGroupEntryHandle { binding, resource }
            }),
        }
    }

    #[inline]
    pub fn with_indices(
        indexed_resources: impl IntoIndexedTransientBindGroupResourceHandleArray<N>,
    ) -> Self {
        Self {
            entries: indexed_resources
                .into_array()
                .map(|(binding, resource)| TransientBindGroupEntryHandle { binding, resource }),
        }
    }
}

impl BindGroupEntryHandles< 1> {
    pub fn single(resource: impl IntoTransientBindGroupResourceHandle) -> [TransientBindGroupEntryHandle; 1] {
        [TransientBindGroupEntryHandle {
            binding: 0,
            resource: resource.into_handle(),
        }]
    }
}

impl<const N: usize> core::ops::Deref for BindGroupEntryHandles<N> {
    type Target = [TransientBindGroupEntryHandle];

    fn deref(&self) -> &[TransientBindGroupEntryHandle] {
        &self.entries
    }
}
