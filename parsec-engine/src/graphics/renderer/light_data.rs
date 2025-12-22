use crate::graphics::{buffer::Buffer, pipeline::PipelineBinding};

pub struct LightData {
    direction_buffer: Buffer,
    direction_binding: PipelineBinding 
}
