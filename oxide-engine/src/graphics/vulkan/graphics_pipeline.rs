use crate::{
    graphics::{
        renderer::mesh_data::{Vertex, VertexFieldFormat},
        vulkan::{
            descriptor_set::VulkanDescriptorSetLayout, device::VulkanDevice,
            format_size::format_size, framebuffer::VulkanFramebuffer,
            renderpass::VulkanRenderpass, shader::VulkanShaderModule,
        },
    },
    utils::id_counter::IdCounter,
};

pub struct VulkanGraphicsPipeline {
    id: u32,
    device_id: u32,
    _framebuffer_id: u32,
    _vertex_shader_id: u32,
    _fragment_shader_id: u32,
    _descriptor_set_layout_ids: Vec<u32>,
    graphics_pipeline: ash::vk::Pipeline,
    graphics_pipeline_layout: ash::vk::PipelineLayout,
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanGraphicsPipelineError {
    #[error("Failed to create Pipeline layout: {0}")]
    LayoutError(ash::vk::Result),
    #[error("Failed to create Pipeline: {0}")]
    CreationError(ash::vk::Result),
    #[error("Pipeline created on a different device")]
    DeviceMismatch,
    #[error("Framebuffer created for a different renderpass")]
    RenderpassMismatch,
}

pub type VulkanVertexFieldFormat = ash::vk::Format;

impl From<VertexFieldFormat> for VulkanVertexFieldFormat {
    fn from(value: VertexFieldFormat) -> Self {
        match value {
            VertexFieldFormat::Float => VulkanVertexFieldFormat::R32_SFLOAT,
            VertexFieldFormat::Vec2 => VulkanVertexFieldFormat::R32G32_SFLOAT,
            VertexFieldFormat::Vec3 => {
                VulkanVertexFieldFormat::R32G32B32_SFLOAT
            },
            VertexFieldFormat::Vec4 => {
                VulkanVertexFieldFormat::R32G32B32A32_SFLOAT
            },
        }
    }
}

static ID_COUNTER: once_cell::sync::Lazy<IdCounter> =
    once_cell::sync::Lazy::new(|| IdCounter::new(0));
impl VulkanGraphicsPipeline {
    pub fn new<V: Vertex>(
        device: &VulkanDevice,
        renderpass: &VulkanRenderpass,
        framebuffer: &VulkanFramebuffer,
        vertex_shader: &VulkanShaderModule,
        fragment_shader: &VulkanShaderModule,
        descriptor_set_layouts: &Vec<VulkanDescriptorSetLayout>,
    ) -> Result<VulkanGraphicsPipeline, VulkanGraphicsPipelineError> {
        if device.id() != renderpass.device_id()
            || device.id() != vertex_shader.device_id()
            || device.id() != fragment_shader.device_id()
        {
            return Err(VulkanGraphicsPipelineError::DeviceMismatch);
        }

        for layout in descriptor_set_layouts.iter() {
            if layout.device_id() != device.id() {
                return Err(VulkanGraphicsPipelineError::DeviceMismatch);
            }
        }

        if renderpass.id() != framebuffer.renderpass_id() {
            return Err(VulkanGraphicsPipelineError::RenderpassMismatch);
        }

        let set_layouts: Vec<_> = descriptor_set_layouts
            .iter()
            .map(|x| *x.get_layout_raw())
            .collect();

        let layout_create_info = ash::vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&set_layouts);

        let pipeline_layout = match unsafe {
            device
                .get_device_raw()
                .create_pipeline_layout(&layout_create_info, None)
        } {
            Ok(val) => val,
            Err(err) => {
                return Err(VulkanGraphicsPipelineError::LayoutError(err));
            },
        };

        let shader_entry_name = c"main";
        let shader_stage_create_infos = [
            ash::vk::PipelineShaderStageCreateInfo {
                module: *vertex_shader.get_shader_module_raw(),
                p_name: shader_entry_name.as_ptr(),
                stage: ash::vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            ash::vk::PipelineShaderStageCreateInfo {
                module: *fragment_shader.get_shader_module_raw(),
                p_name: shader_entry_name.as_ptr(),
                stage: ash::vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
        ];

        let viewports = [ash::vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: framebuffer.get_extent_raw().width as f32,
            height: framebuffer.get_extent_raw().height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        let scissors = [framebuffer.get_extent_raw().into()];
        let viewport_state_info =
            ash::vk::PipelineViewportStateCreateInfo::default()
                .scissors(&scissors)
                .viewports(&viewports);

        let rasterization_info =
            ash::vk::PipelineRasterizationStateCreateInfo {
                front_face: ash::vk::FrontFace::COUNTER_CLOCKWISE,
                line_width: 1.0,
                polygon_mode: ash::vk::PolygonMode::FILL,
                ..Default::default()
            };
        let multisample_state_info =
            ash::vk::PipelineMultisampleStateCreateInfo {
                rasterization_samples: ash::vk::SampleCountFlags::TYPE_1,
                ..Default::default()
            };
        let color_blend_attachment_states =
            [ash::vk::PipelineColorBlendAttachmentState {
                blend_enable: 0,
                src_color_blend_factor: ash::vk::BlendFactor::SRC_COLOR,
                dst_color_blend_factor:
                    ash::vk::BlendFactor::ONE_MINUS_DST_COLOR,
                color_blend_op: ash::vk::BlendOp::ADD,
                src_alpha_blend_factor: ash::vk::BlendFactor::ZERO,
                dst_alpha_blend_factor: ash::vk::BlendFactor::ZERO,
                alpha_blend_op: ash::vk::BlendOp::ADD,
                color_write_mask: ash::vk::ColorComponentFlags::RGBA,
            }];
        let color_blend_state =
            ash::vk::PipelineColorBlendStateCreateInfo::default()
                .logic_op(ash::vk::LogicOp::CLEAR)
                .attachments(&color_blend_attachment_states);

        let dynamic_state = [
            ash::vk::DynamicState::VIEWPORT,
            ash::vk::DynamicState::SCISSOR,
        ];
        let dynamic_state_info =
            ash::vk::PipelineDynamicStateCreateInfo::default()
                .dynamic_states(&dynamic_state);

        let mut vertex_input_attribute_descriptions = Vec::new();
        let mut current_offset = 0;
        for (idx, field) in V::fields().iter().enumerate() {
            vertex_input_attribute_descriptions.push(
                ash::vk::VertexInputAttributeDescription {
                    binding: 0,
                    location: idx as u32,
                    format: field.format.into(),
                    offset: current_offset,
                },
            );
            current_offset += format_size(field.format.into()).unwrap();
        }

        let vertex_input_binding_descriptions =
            [ash::vk::VertexInputBindingDescription {
                binding: 0,
                stride: current_offset,
                input_rate: ash::vk::VertexInputRate::VERTEX,
            }];

        let vertex_input_state_info =
            ash::vk::PipelineVertexInputStateCreateInfo::default()
                .vertex_attribute_descriptions(
                    &vertex_input_attribute_descriptions,
                )
                .vertex_binding_descriptions(
                    &vertex_input_binding_descriptions,
                );

        let vertex_input_assembly_state_info =
            ash::vk::PipelineInputAssemblyStateCreateInfo {
                topology: ash::vk::PrimitiveTopology::TRIANGLE_LIST,
                ..Default::default()
            };

        let noop_stencil_state = ash::vk::StencilOpState {
            fail_op: ash::vk::StencilOp::KEEP,
            pass_op: ash::vk::StencilOp::KEEP,
            depth_fail_op: ash::vk::StencilOp::KEEP,
            compare_op: ash::vk::CompareOp::ALWAYS,
            ..Default::default()
        };

        let depth_state_info = ash::vk::PipelineDepthStencilStateCreateInfo {
            depth_test_enable: 1,
            depth_write_enable: 1,
            depth_compare_op: ash::vk::CompareOp::LESS_OR_EQUAL,
            front: noop_stencil_state,
            back: noop_stencil_state,
            max_depth_bounds: 1.0,
            ..Default::default()
        };

        let graphic_pipeline_info =
            ash::vk::GraphicsPipelineCreateInfo::default()
                .stages(&shader_stage_create_infos)
                .vertex_input_state(&vertex_input_state_info)
                .input_assembly_state(&vertex_input_assembly_state_info)
                .viewport_state(&viewport_state_info)
                .rasterization_state(&rasterization_info)
                .multisample_state(&multisample_state_info)
                .color_blend_state(&color_blend_state)
                .dynamic_state(&dynamic_state_info)
                .layout(pipeline_layout)
                .depth_stencil_state(&depth_state_info)
                .render_pass(*renderpass.get_renderpass_raw());

        let pipeline = match unsafe {
            device.get_device_raw().create_graphics_pipelines(
                ash::vk::PipelineCache::null(),
                &[graphic_pipeline_info],
                None,
            )
        } {
            Ok(val) => val,
            Err(err) => {
                return Err(VulkanGraphicsPipelineError::CreationError(err.1));
            },
        }[0];

        Ok(VulkanGraphicsPipeline {
            id: ID_COUNTER.next(),
            device_id: device.id(),
            _framebuffer_id: framebuffer.id(),
            _vertex_shader_id: vertex_shader.id(),
            _fragment_shader_id: fragment_shader.id(),
            _descriptor_set_layout_ids: descriptor_set_layouts
                .iter()
                .map(|x| x.id())
                .collect(),
            graphics_pipeline: pipeline,
            graphics_pipeline_layout: pipeline_layout,
        })
    }

    pub fn get_pipeline_raw(&self) -> &ash::vk::Pipeline {
        &self.graphics_pipeline
    }

    pub fn get_layout_raw(&self) -> &ash::vk::PipelineLayout {
        &self.graphics_pipeline_layout
    }

    pub fn id(&self) -> u32 { self.id }

    pub fn _framebuffer_id(&self) -> u32 { self._framebuffer_id }

    pub fn _vertex_shader_id(&self) -> u32 { self._vertex_shader_id }

    pub fn _fragment_shader_id(&self) -> u32 { self._fragment_shader_id }

    pub fn _descriptor_set_layout_ids(&self) -> &[u32] {
        &self._descriptor_set_layout_ids
    }

    pub fn device_id(&self) -> u32 { self.device_id }
}
