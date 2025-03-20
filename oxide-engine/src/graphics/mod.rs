use graphics_data::GraphicsData;

pub mod graphics_data;
pub mod renderer;
pub mod vulkan;
pub mod window;

#[derive(Debug)]
pub struct Graphics {
    pub data: Option<GraphicsData>,
}

impl Graphics {
    pub fn new() -> Graphics {
        Graphics { data: None }
    }
}
