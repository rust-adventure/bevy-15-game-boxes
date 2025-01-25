use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

// This struct defines the data that will be
// passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GoalMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
    #[texture(1)]
    #[sampler(2)]
    pub color_texture: Option<Handle<Image>>,
    pub alpha_mode: AlphaMode,
}

/// The Material trait is very configurable, but
/// comes with sensible defaults for all methods.
/// You only need to implement functions for
/// features that need non-default behavior. See
/// the Material api docs for details!
impl Material for GoalMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/goal.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    fn specialize(
            _pipeline: &bevy::pbr::MaterialPipeline<Self>,
            descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
            _layout: &bevy::render::mesh::MeshVertexBufferLayoutRef,
            _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError>{
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}
