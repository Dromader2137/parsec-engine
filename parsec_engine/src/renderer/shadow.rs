use parsec_engine_macros::embed_shader;

use crate::{
    graphics::{
        ActiveGraphicsBackend,
        framebuffer::{Framebuffer, FramebufferBuilder},
        image::{ImageAspect, ImageFormat, ImageSize, ImageUsage},
        pipeline::{
            PipelineBindingType, PipelineOptions, PipelineResource, PipelineResourceBindingLayout,
            PipelineResourceLayout, PipelineResourceLayoutBuilder, PipelineShaderStage,
        },
        renderpass::{
            Renderpass, RenderpassAttachment, RenderpassAttachmentLoadOp,
            RenderpassAttachmentStoreOp, RenderpassAttachmentType, RenderpassBuilder,
            RenderpassClearValue,
        },
        shader_module::{ShaderModule, ShaderModuleBuilder, ShaderType},
    },
    math::uvec::Vec2u,
    renderer::{
        integrated_image::{IntegratedImage, IntegratedImageBuilder},
        material_data::{MaterialBase, MaterialData, MaterialPipelineBinding},
    },
};

pub struct RendererShadows {
    vertex_shader: ShaderModule,
    fragment_shader: ShaderModule,
    pub material_base: MaterialBase,
    pub material: MaterialData,
    pub renderpass: Renderpass,
    pub framebuffer: Framebuffer,
    shadow_texture: IntegratedImage,
    texture_resource_layout: PipelineResourceLayout,
    pub texture_resource: PipelineResource,
}

const SHADOW_VERT: &'static [u32] = embed_shader!(
    "vert",
    "
#version 450

struct DirLightData {
    mat4 world_to_light;
	vec4 atlas_clip;
    vec3 direction;
    vec3 color;
};

layout(set = 0, binding = 0) uniform Translation {
    mat4 matrix;
} translation;
layout(set = 0, binding = 1) uniform Scale {
    mat4 matrix;
} scale;
layout(set = 0, binding = 2) uniform Rotation {
    mat4 matrix;
} rotation;
layout(std140, set = 1, binding = 0) readonly buffer LightData {
    uint light_count;
    DirLightData data[32];
} lightData;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec3 inTangent;
layout(location = 3) in vec2 inUV;

out gl_PerVertex {
	vec4 gl_Position;
	float gl_ClipDistance[4];
};

void main() {
    vec4 pos =
        lightData.data[gl_InstanceIndex].world_to_light *
            translation.matrix *
            scale.matrix *
            rotation.matrix *
            vec4(inPosition, 1.0);
	vec2 lo = lightData.data[gl_InstanceIndex].atlas_clip.xy * 2.0 - 1.0;
	vec2 hi = lightData.data[gl_InstanceIndex].atlas_clip.zw * 2.0 - 1.0;
	pos.xy = lo + (pos.xy + 1.0) * (hi - lo) * 0.5;
    gl_ClipDistance[0] = pos.x - lo.x * pos.w;
    gl_ClipDistance[1] = hi.x * pos.w - pos.x;
    gl_ClipDistance[2] = pos.y - lo.y * pos.w;
    gl_ClipDistance[3] = hi.y * pos.w - pos.y;
	gl_Position = pos;
}
"
);

const SHADOW_FRAG: &'static [u32] = embed_shader!(
    "frag",
    "
#version 450

void main() {}
"
);

impl RendererShadows {
    pub fn new(backend: &mut ActiveGraphicsBackend) -> RendererShadows {
        let renderpass = RenderpassBuilder::new()
            .attachment(RenderpassAttachment {
                attachment_type: RenderpassAttachmentType::Depth,
                image_format: ImageFormat::D32,
                clear_value: RenderpassClearValue::Depth(1.0),
                load_op: RenderpassAttachmentLoadOp::Clear,
                store_op: RenderpassAttachmentStoreOp::Store,
            })
            .build(backend)
            .unwrap();
        let vertex_shader = ShaderModuleBuilder::new()
            .code(SHADOW_VERT)
            .shader_type(ShaderType::Vertex)
            .build(backend)
            .unwrap();
        let fragment_shader = ShaderModuleBuilder::new()
            .code(SHADOW_FRAG)
            .shader_type(ShaderType::Fragment)
            .build(backend)
            .unwrap();
        let material_base = MaterialBase::new(
            &mut *backend,
            vertex_shader.handle(),
            fragment_shader.handle(),
            renderpass.handle(),
            vec![
                vec![
                    PipelineResourceBindingLayout::new(PipelineBindingType::UniformBuffer, &[
                        PipelineShaderStage::Vertex,
                    ]),
                    PipelineResourceBindingLayout::new(PipelineBindingType::UniformBuffer, &[
                        PipelineShaderStage::Vertex,
                    ]),
                    PipelineResourceBindingLayout::new(PipelineBindingType::UniformBuffer, &[
                        PipelineShaderStage::Vertex,
                    ]),
                ],
                vec![PipelineResourceBindingLayout::new(PipelineBindingType::StorageBuffer, &[
                    PipelineShaderStage::Vertex,
                    PipelineShaderStage::Fragment,
                ])],
            ],
            PipelineOptions::default(),
        );

        let texture_size = 1 << 12;

        let material = MaterialData::new(&material_base, vec![
            MaterialPipelineBinding::Model,
            MaterialPipelineBinding::Light,
        ]);

        let shadow_texture = IntegratedImageBuilder::default()
            .size(ImageSize::new(Vec2u::new(texture_size << 1, texture_size)).unwrap())
            .format(ImageFormat::D32)
            .aspect(ImageAspect::Depth)
            .usage(&[ImageUsage::DepthAttachment, ImageUsage::Sampled])
            .build(backend)
            .unwrap();

        let texture_resource_layout = PipelineResourceLayoutBuilder::new()
            .bindings(&[PipelineResourceBindingLayout {
                binding_type: PipelineBindingType::TextureSampler,
                shader_stages: vec![PipelineShaderStage::Fragment],
            }])
            .build(backend)
            .unwrap();

        let texture_resource = texture_resource_layout.create_resource(backend).unwrap();

        texture_resource
            .bind_sampler(
                backend,
                shadow_texture.sampler().handle(),
                shadow_texture.view().handle(),
                0,
            )
            .unwrap();

        let framebuffer = FramebufferBuilder::new()
            .attachment(shadow_texture.view().handle())
            .size(Vec2u::new(texture_size << 1, texture_size))
            .renderpass(renderpass.handle())
            .build(backend)
            .unwrap();

        RendererShadows {
            shadow_texture,
            renderpass,
            material,
            material_base,
            vertex_shader,
            fragment_shader,
            texture_resource_layout,
            texture_resource,
            framebuffer,
        }
    }

    pub fn destroy(self, backend: &mut ActiveGraphicsBackend) {
        self.texture_resource.destroy(backend).unwrap();
        self.texture_resource_layout.destroy(backend).unwrap();
        self.shadow_texture.destroy(backend).unwrap();
        self.framebuffer.destroy(backend).unwrap();
        self.renderpass.destroy(backend).unwrap();
        // TODO maybe cleanup shadow material
        self.material_base.destroy(backend).unwrap();
        self.fragment_shader.destroy(backend).unwrap();
        self.vertex_shader.destroy(backend).unwrap();
    }
}
