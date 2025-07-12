// use std::sync::Arc;
//
// use crate::graphics::vulkan::{graphics_pipeline::GraphicsPipeline, shader::ShaderModule};
//
// pub struct MaterialData {
//     name: &'static str,
//     vertex_shader: Arc<ShaderModule>,
//     fragment_shader: Arc<ShaderModule>,
//     pipeline: Arc<GraphicsPipeline>,
// }

// impl MaterialData {
//     pub fn new(name: &str) -> Result<MaterialData, VulkanError> {
//         let vertex_shader = ShaderModule::new(device, code);
//         let fragment_shader = ShaderModule::new(device, code);
//
//         Some(MaterialData {
//             name,
//             vertex_shader,
//             fragment_shader: ,
//             pipeline
//         })
//
//     }
// }
