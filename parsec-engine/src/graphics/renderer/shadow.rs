use crate::{
    graphics::{
        ActiveGraphicsBackend,
        buffer::{BufferBuilder, BufferContent, BufferUsage},
        framebuffer::{Framebuffer, FramebufferBuilder},
        image::{ImageAspect, ImageFormat, ImageSize, ImageUsage},
        pipeline::{
            PipelineBindingType, PipelineOptions, PipelineResource,
            PipelineResourceBindingLayout, PipelineResourceLayout,
            PipelineResourceLayoutBuilder, PipelineShaderStage,
        },
        renderer::{
            light_data::RendererLights,
            material_data::{
                MaterialBase, MaterialData, MaterialPipelineBinding,
            },
            texture::{Texture, TextureBuilder},
        },
        renderpass::{
            Renderpass, RenderpassAttachment, RenderpassAttachmentLoadOp,
            RenderpassAttachmentStoreOp, RenderpassAttachmentType,
            RenderpassBuilder, RenderpassClearValue,
        },
        shader::{Shader, ShaderBuilder, ShaderType},
        vulkan::shader::read_shader_code,
    },
    math::{mat::Matrix4f, uvec::Vec2u, vec::Vec3f},
};

pub struct RendererShadows {
    vertex_shader: Shader,
    fragment_shader: Shader,
    pub material_base: MaterialBase,
    pub material: MaterialData,
    pub renderpass: Renderpass,
    pub framebuffer: Framebuffer,
    shadow_texture: Texture,
    texture_resource_layout: PipelineResourceLayout,
    pub texture_resource: PipelineResource,
}

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
        let vertex_shader = ShaderBuilder::new()
            .code(&read_shader_code("shaders/shadow_vert.spv").unwrap())
            .shader_type(ShaderType::Vertex)
            .build(backend)
            .unwrap();
        let fragment_shader = ShaderBuilder::new()
            .code(&read_shader_code("shaders/shadow_frag.spv").unwrap())
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
                    PipelineResourceBindingLayout::new(
                        PipelineBindingType::UniformBuffer,
                        &[PipelineShaderStage::Vertex],
                    ),
                    PipelineResourceBindingLayout::new(
                        PipelineBindingType::UniformBuffer,
                        &[PipelineShaderStage::Vertex],
                    ),
                    PipelineResourceBindingLayout::new(
                        PipelineBindingType::UniformBuffer,
                        &[PipelineShaderStage::Vertex],
                    ),
                ],
                vec![PipelineResourceBindingLayout::new(
                    PipelineBindingType::StorageBuffer,
                    &[PipelineShaderStage::Vertex],
                )],
            ],
            PipelineOptions::default(),
        );

        let texture_size = 1 << 10;

        let material = MaterialData::new(&material_base, vec![
            MaterialPipelineBinding::Model,
            MaterialPipelineBinding::Light,
        ]);

        let shadow_texture = TextureBuilder::default()
            .size(
                ImageSize::new(Vec2u::new(texture_size, texture_size)).unwrap(),
            )
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

        let texture_resource =
            texture_resource_layout.create_resource(backend).unwrap();

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
            .size(Vec2u::new(texture_size, texture_size))
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
        // self.material.destroy(backend).unwrap();
        // self.material_base.destroy(backend).unwrap();
        self.fragment_shader.destroy(backend).unwrap();
        self.vertex_shader.destroy(backend).unwrap();
    }
}
