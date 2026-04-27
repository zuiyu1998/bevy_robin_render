use bevy_camera::{CameraOutputMode, ClearColor, ClearColorConfig};
use bevy_ecs::{query::QueryItem, world::World};
use robin_render::{
    camera::ExtractedCamera,
    frame_graph::FrameGraph,
    render_graph::{NodeRunError, ViewNode},
    view::ViewTarget,
};

use crate::upscaling::ViewUpscalingPipeline;

pub struct UpscalingNode;

impl ViewNode for UpscalingNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static ViewUpscalingPipeline,
        Option<&'static ExtractedCamera>,
    );

    fn run<'w>(
        &self,
        frame_graph: &mut FrameGraph,
        (target, upscaling_target, camera): QueryItem<Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let clear_color_global = world.resource::<ClearColor>();

        let clear_color = if let Some(camera) = camera {
            match camera.output_mode {
                CameraOutputMode::Write { clear_color, .. } => clear_color,
                CameraOutputMode::Skip => {
                    return Ok(());
                }
            }
        } else {
            ClearColorConfig::Default
        };
        let clear_color = match clear_color {
            ClearColorConfig::Default => Some(clear_color_global.0),
            ClearColorConfig::Custom(color) => Some(color),
            ClearColorConfig::None => None,
        };
        let converted_clear_color = clear_color.map(Into::into);

        let out_attachment =
            target.create_out_texture_color_attachment(converted_clear_color, frame_graph);

        let mut pass_builder = frame_graph.create_pass_builder("upscaling");

        let mut render_pass_builder = pass_builder.create_render_pass_builder("upscaling_node");

        render_pass_builder.add_color_attachment(out_attachment);

        if let Some(camera) = camera
            && let Some(viewport) = &camera.viewport
        {
            let size = viewport.physical_size;
            let position = viewport.physical_position;
            render_pass_builder.set_scissor_rect(position.x, position.y, size.x, size.y);
        }

        render_pass_builder.set_render_pipeline(upscaling_target.0.id());
        // render_pass_builder.set_bind_group(0, bind_group, &[]);
        render_pass_builder.draw(0..3, 0..1);

        Ok(())
    }
}
