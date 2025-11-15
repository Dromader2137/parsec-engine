use std::sync::Arc;

use crate::graphics::vulkan::{
    VulkanError, descriptor_set::DescriptorSetLayout, framebuffer::Framebuffer,
    shader::ShaderModule,
};

pub struct GraphicsPipeline {
    pub framebuffer: Arc<Framebuffer>,
    pub vertex_shader: Arc<ShaderModule>,
    pub fragment_shader: Arc<ShaderModule>,
    pub descriptor_set_layouts: Vec<Arc<DescriptorSetLayout>>,
    graphics_pipeline: ash::vk::Pipeline,
    graphics_pipeline_layout: ash::vk::PipelineLayout,
}

#[derive(Debug)]
pub enum GraphicsPipelineError {
    LayoutError(ash::vk::Result),
    CreationError(ash::vk::Result),
}

impl From<GraphicsPipelineError> for VulkanError {
    fn from(value: GraphicsPipelineError) -> Self { VulkanError::GrphicsPipelineError(value) }
}

pub type VertexFieldFormat = ash::vk::Format;

pub struct VertexField {
    pub format: VertexFieldFormat,
    pub offset: u32,
}

pub trait Vertex: Clone + Copy {
    fn description() -> Vec<VertexField>;
    fn size() -> u32;
}

impl GraphicsPipeline {
    pub fn new<V: Vertex>(
        framebuffer: Arc<Framebuffer>,
        vertex_shader: Arc<ShaderModule>,
        fragment_shader: Arc<ShaderModule>,
        descriptor_set_layouts: Vec<Arc<DescriptorSetLayout>>,
    ) -> Result<Arc<GraphicsPipeline>, GraphicsPipelineError> {
        let set_layouts: Vec<_> = descriptor_set_layouts
            .iter()
            .map(|x| *x.get_layout_raw())
            .collect();

        let layout_create_info =
            ash::vk::PipelineLayoutCreateInfo::default().set_layouts(&set_layouts);

        let pipeline_layout = match unsafe {
            framebuffer
                .renderpass
                .device
                .get_device_raw()
                .create_pipeline_layout(&layout_create_info, None)
        } {
            Ok(val) => val,
            Err(err) => return Err(GraphicsPipelineError::LayoutError(err)),
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
        let viewport_state_info = ash::vk::PipelineViewportStateCreateInfo::default()
            .scissors(&scissors)
            .viewports(&viewports);

        let rasterization_info = ash::vk::PipelineRasterizationStateCreateInfo {
            front_face: ash::vk::FrontFace::COUNTER_CLOCKWISE,
            line_width: 1.0,
            polygon_mode: ash::vk::PolygonMode::FILL,
            ..Default::default()
        };
        let multisample_state_info = ash::vk::PipelineMultisampleStateCreateInfo {
            rasterization_samples: ash::vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };
        let color_blend_attachment_states = [ash::vk::PipelineColorBlendAttachmentState {
            blend_enable: 0,
            src_color_blend_factor: ash::vk::BlendFactor::SRC_COLOR,
            dst_color_blend_factor: ash::vk::BlendFactor::ONE_MINUS_DST_COLOR,
            color_blend_op: ash::vk::BlendOp::ADD,
            src_alpha_blend_factor: ash::vk::BlendFactor::ZERO,
            dst_alpha_blend_factor: ash::vk::BlendFactor::ZERO,
            alpha_blend_op: ash::vk::BlendOp::ADD,
            color_write_mask: ash::vk::ColorComponentFlags::RGBA,
        }];
        let color_blend_state = ash::vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op(ash::vk::LogicOp::CLEAR)
            .attachments(&color_blend_attachment_states);

        let dynamic_state = [
            ash::vk::DynamicState::VIEWPORT,
            ash::vk::DynamicState::SCISSOR,
        ];
        let dynamic_state_info =
            ash::vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_state);

        let vertex_input_binding_descriptions = [ash::vk::VertexInputBindingDescription {
            binding: 0,
            stride: V::size(),
            input_rate: ash::vk::VertexInputRate::VERTEX,
        }];
        let vertex_input_attribute_descriptions = V::description()
            .iter()
            .enumerate()
            .map(|(i, x)| ash::vk::VertexInputAttributeDescription {
                binding: 0,
                location: i as u32,
                format: x.format,
                offset: x.offset,
            })
            .collect::<Vec<_>>();

        let vertex_input_state_info = ash::vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_attribute_descriptions(&vertex_input_attribute_descriptions)
            .vertex_binding_descriptions(&vertex_input_binding_descriptions);

        let vertex_input_assembly_state_info = ash::vk::PipelineInputAssemblyStateCreateInfo {
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

        let graphic_pipeline_info = ash::vk::GraphicsPipelineCreateInfo::default()
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
            .render_pass(*framebuffer.renderpass.get_renderpass_raw());

        let pipeline = match unsafe {
            framebuffer
                .renderpass
                .device
                .get_device_raw()
                .create_graphics_pipelines(
                    ash::vk::PipelineCache::null(),
                    &[graphic_pipeline_info],
                    None,
                )
        } {
            Ok(val) => val,
            Err(err) => return Err(GraphicsPipelineError::CreationError(err.1)),
        }[0];

        Ok(Arc::new(GraphicsPipeline {
            framebuffer,
            vertex_shader,
            fragment_shader,
            descriptor_set_layouts,
            graphics_pipeline: pipeline,
            graphics_pipeline_layout: pipeline_layout,
        }))
    }

    pub fn get_pipeline_raw(&self) -> &ash::vk::Pipeline { &self.graphics_pipeline }

    pub fn get_layout_raw(&self) -> &ash::vk::PipelineLayout { &self.graphics_pipeline_layout }
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        unsafe {
            self.framebuffer
                .renderpass
                .device
                .get_device_raw()
                .destroy_pipeline_layout(self.graphics_pipeline_layout, None);
            self.framebuffer
                .renderpass
                .device
                .get_device_raw()
                .destroy_pipeline(self.graphics_pipeline, None);
        };
    }
}
