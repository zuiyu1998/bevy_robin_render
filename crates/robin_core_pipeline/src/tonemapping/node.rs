use std::ops::Deref;

use crate::tonemapping::{TonemappingLuts, TonemappingPipeline, ViewTonemappingPipeline};

use bevy_ecs::prelude::*;
use robin_render::{
    diagnostic::RecordDiagnostics,
    frame_graph::{
        BindGroupEntryHandles, TransientBindGroupHandle, TransientRenderPassColorAttachment,
    },
    render_asset::RenderAssets,
    render_phase::TrackedRenderPass,
    render_resource::{LoadOp, Operations, PipelineCache, StoreOp},
    renderer::{FrameGraphs, RenderContext, ViewQuery},
    texture::{FallbackImage, GpuImage},
    view::{ViewTarget, ViewUniformOffset, ViewUniforms},
};

use super::{Tonemapping, get_lut_bindings};

/// Cached bind group state for tonemapping.
#[derive(Default)]
pub struct TonemappingBindGroupCache {
    last_tonemapping: Option<Tonemapping>,
}

pub fn tonemapping(
    view: ViewQuery<(
        &ViewUniformOffset,
        &ViewTarget,
        &ViewTonemappingPipeline,
        &Tonemapping,
    )>,
    pipeline_cache: Res<PipelineCache>,
    tonemapping_pipeline: Res<TonemappingPipeline>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    fallback_image: Res<FallbackImage>,
    view_uniforms: Res<ViewUniforms>,
    tonemapping_luts: Res<TonemappingLuts>,
    mut cache: Local<TonemappingBindGroupCache>,
    ctx: RenderContext,
    mut frame_graphs: ResMut<FrameGraphs>,
) {
    let entity = view.entity();
    let (view_uniform_offset, target, view_tonemapping_pipeline, tonemapping) = view.into_inner();

    if *tonemapping == Tonemapping::None {
        return;
    }

    if !target.is_hdr() {
        return;
    }

    let Some(pipeline) = pipeline_cache.get_render_pipeline(view_tonemapping_pipeline.0) else {
        return;
    };

    let diagnostics = ctx.diagnostic_recorder();
    let diagnostics = diagnostics.as_deref();

    let mut frame_graph = frame_graphs.get_or_insert(entity);

    let view_uniforms_buffer_handle = view_uniforms
        .uniforms
        .get_buffer_handle(frame_graph)
        .unwrap();

    let post_process = target.post_process_write(&mut frame_graph);
    let source = post_process.source_texture_view_handle();

    let tonemapping_changed = cache.last_tonemapping != Some(*tonemapping);
    if tonemapping_changed {
        cache.last_tonemapping = Some(*tonemapping);
    }

    let lut_bindings = get_lut_bindings(
        &gpu_images,
        &tonemapping_luts,
        tonemapping,
        &fallback_image,
        frame_graph,
    );

    let bind_group = TransientBindGroupHandle::build(
        &pipeline_cache.get_bind_group_layout(&tonemapping_pipeline.texture_bind_group),
    )
    .set_entries(&BindGroupEntryHandles::sequential((
        view_uniforms_buffer_handle,
        source,
        tonemapping_pipeline.sampler.deref(),
        lut_bindings.0,
        lut_bindings.1.deref(),
    )))
    .finished();

    let mut pass_builder = frame_graph.create_pass_builder("tonemapping_node");

    let mut render_pass_builder = pass_builder.create_render_pass_builder("tonemapping");
    let destination = post_process.destination(&mut render_pass_builder);

    render_pass_builder.add_color_attachment(TransientRenderPassColorAttachment {
        view: destination,
        depth_slice: None,
        resolve_target: None,
        ops: Operations {
            load: LoadOp::Clear(Default::default()), // TODO shouldn't need to be cleared
            store: StoreOp::Store,
        },
    });
    let mut render_pass = TrackedRenderPass::new(ctx.render_device(), render_pass_builder);

    let time_span = diagnostics.time_span(&mut render_pass, "tonemapping");

    {
        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group_handle(0, &bind_group, &[view_uniform_offset.offset]);
        render_pass.draw(0..3, 0..1);
    }

    time_span.end(&mut render_pass);
}
