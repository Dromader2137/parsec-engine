use crate::graphics::{buffer::Buffer, pipeline::PipelineBinding};

pub struct LightData {
    texture_pos: (u32, u32),
    texture_size: u32,
    transformation_direction_buffer: Buffer,
    transformation_direction_binding: PipelineBinding,
}
