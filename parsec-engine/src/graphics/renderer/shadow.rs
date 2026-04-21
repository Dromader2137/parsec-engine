use crate::{
    graphics::{
        ActiveGraphicsBackend,
        buffer::{Buffer, BufferBuilder, BufferContent, BufferUsage},
        framebuffer::{Framebuffer, FramebufferBuilder},
        image::{
            ImageAspect, ImageBuilder, ImageFormat, ImageSize, ImageUsage,
            ImageViewBuilder,
        },
        pipeline::{
            PipelineBindingType, PipelineOptions, PipelineResource,
            PipelineResourceBindingLayout, PipelineResourceLayout,
            PipelineResourceLayoutBuilder, PipelineShaderStage,
        },
        renderer::{
            LD,
            material_data::{
                MaterialBase, MaterialData, MaterialPipelineBinding,
            },
            texture_atlas::{TextureAtlas, TextureAtlasBuilder},
        },
        renderpass::{
            Renderpass, RenderpassAttachment, RenderpassAttachmentLoadOp,
            RenderpassAttachmentStoreOp, RenderpassAttachmentType,
            RenderpassBuilder, RenderpassClearValue,
        },
        sampler::SamplerBuilder,
        shader::{Shader, ShaderBuilder, ShaderType},
        vulkan::shader::read_shader_code,
    },
    math::{mat::Matrix4f, uvec::Vec2u, vec::Vec3f},
    utils::identifiable::IdStore,
};

pub struct RendererShadow {
    vertex_shader: Shader,
    fragment_shader: Shader,
    pub material_base: MaterialBase,
    pub material: MaterialData,
    pub renderpass: Renderpass,
    pub framebuffer: Framebuffer,
    atlas: TextureAtlas,
    pub image_resource: PipelineResource,
    light_buffer: Buffer,
    light_layout: PipelineResourceLayout,
    pub light_resource: PipelineResource,
    proj_buffer: Buffer,
    proj_resource: PipelineResource,
    look_buffer: Buffer,
    look_resource: PipelineResource,
}

impl RendererShadow {
    pub fn new(
        backend: &mut ActiveGraphicsBackend,
    ) -> RendererShadow {
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
                    PipelineBindingType::UniformBuffer,
                    &[PipelineShaderStage::Vertex],
                )],
                vec![PipelineResourceBindingLayout::new(
                    PipelineBindingType::UniformBuffer,
                    &[PipelineShaderStage::Vertex],
                )],
            ],
            PipelineOptions::default(),
        );

        let image_size = 1 << 10;
        let proj_buffer = BufferBuilder::new()
            .usage(&[BufferUsage::Uniform])
            .data(BufferContent::from_slice(&[Matrix4f::orthographic(
                0.0, 100.0, 25.0, 25.0,
            )]))
            .build(backend)
            .unwrap();
        let look_buffer = BufferBuilder::new()
            .usage(&[BufferUsage::Uniform])
            .data(BufferContent::from_slice(&[Matrix4f::look_at(
                Vec3f::new(-40.0, 40.0, -40.0),
                Vec3f::new(1.0, -1.0, 1.0),
                Vec3f::new(1.0, 1.0, 1.0),
            )]))
            .build(backend)
            .unwrap();
        let proj_resource = material_base.resource_layouts()[2]
            .create_resource(backend)
            .unwrap();
        let look_resource = material_base.resource_layouts()[1]
            .create_resource(backend)
            .unwrap();
        proj_resource
            .bind_buffer(backend, proj_buffer.handle(), 0)
            .unwrap();
        look_resource
            .bind_buffer(backend, look_buffer.handle(), 0)
            .unwrap();
        let material = MaterialData::new(&material_base, vec![
            MaterialPipelineBinding::Model,
            MaterialPipelineBinding::Generic(look_resource.handle()),
            MaterialPipelineBinding::Generic(proj_resource.handle()),
        ]);

        let atlas = TextureAtlasBuilder::default()
            .size(
                ImageSize::new(Vec2u::new(
                    image_size,
                    image_size,
                ))
                .unwrap(),
            )
            .format(ImageFormat::D32)
            .aspect(ImageAspect::Depth)
            .usage(&[ImageUsage::DepthAttachment, ImageUsage::Sampled])
            .build(backend)
            .unwrap();

        let tex_resource = PipelineResourceLayoutBuilder::new()
            .bindings(&[PipelineResourceBindingLayout {
                binding_type: PipelineBindingType::TextureSampler,
                shader_stages: vec![PipelineShaderStage::Fragment],
            }])
            .build(backend)
            .unwrap()
            .create_resource(backend)
            .unwrap();
        tex_resource
            .bind_sampler(
                backend,
                atlas.texture().sampler().handle(),
                atlas.texture().view().handle(),
                0,
            )
            .unwrap();

        let framebuffer = FramebufferBuilder::new()
            .attachment(atlas.texture().view().handle())
            .size(Vec2u::new(image_size, image_size))
            .renderpass(renderpass.handle())
            .build(backend)
            .unwrap();

        let light_buffer = BufferBuilder::new()
            .usage(&[BufferUsage::Uniform])
            .data(BufferContent::from_slice(&[LD {
                dir: Vec3f::new(1.0, -1.0, 1.0),
                mat: Matrix4f::orthographic(0.0, 100.0, 25.0, 25.0)
                    * Matrix4f::look_at(
                        Vec3f::new(-40.0, 40.0, -40.0),
                        Vec3f::new(1.0, -1.0, 1.0),
                        Vec3f::new(1.0, 1.0, 1.0),
                    ),
                _pad: 0,
            }]))
            .build(backend)
            .unwrap();
        let light_layout = PipelineResourceLayoutBuilder::new()
            .bindings(&[PipelineResourceBindingLayout::new(
                PipelineBindingType::UniformBuffer,
                &[PipelineShaderStage::Fragment, PipelineShaderStage::Vertex],
            )])
            .build(backend)
            .unwrap();
        let light_resource = light_layout.create_resource(backend).unwrap();
        light_resource
            .bind_buffer(backend, light_buffer.handle(), 0)
            .unwrap();

        RendererShadow {
            atlas,
            light_buffer,
            light_layout,
            light_resource,
            renderpass,
            material,
            material_base,
            vertex_shader,
            fragment_shader,
            image_resource: tex_resource,
            framebuffer: framebuffer,
            proj_buffer: proj_buffer,
            proj_resource: proj_resource,
            look_buffer: look_buffer,
            look_resource,
        }
    }

    pub fn destroy(self, backend: &mut ActiveGraphicsBackend) {
        self.look_resource.destroy(backend).unwrap();
        self.look_buffer.destroy(backend).unwrap();
        self.proj_resource.destroy(backend).unwrap();
        self.proj_buffer.destroy(backend).unwrap();
        self.light_resource.destroy(backend).unwrap();
        self.light_layout.destroy(backend).unwrap();
        self.light_buffer.destroy(backend).unwrap();
        self.image_resource.destroy(backend).unwrap();
        self.atlas.destroy(backend).unwrap();
        self.framebuffer.destroy(backend).unwrap();
        self.renderpass.destroy(backend).unwrap();
        // TODO maybe cleanup shadow material
        // self.material.destroy(backend).unwrap();
        // self.material_base.destroy(backend).unwrap();
        self.fragment_shader.destroy(backend).unwrap();
        self.vertex_shader.destroy(backend).unwrap();
    }
}
