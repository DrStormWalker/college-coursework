use specs::{Join, ReadExpect, ReadStorage, System};

use super::{components::RenderModel, model::DrawModel, state::RenderPassContainer};

/*pub fn render_entites<'a>(
    models: ReadStorage<'a, RenderModel>,
    pipeline: &'a wgpu::RenderPipeline,
    render_pass: &'a mut wgpu::RenderPass<'a>,
    camera_bind_group: &'a wgpu::BindGroup,
    light_bind_group: &'a wgpu::BindGroup,
) {
    render_pass.set_pipeline(pipeline);

    (&models).join().for_each(|model: &RenderModel| {
        render_pass.draw_model(&model.model, camera_bind_group, light_bind_group)
    })
}*/
