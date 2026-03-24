#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Buffer {
    id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BufferUsage {
    Uniform,
    Vertex,
    Index,
    TransferSrc,
    TransferDst,
}

#[derive(Debug)]
pub enum BufferError {
    BufferCreationError(anyhow::Error),
    BufferUpdateError(anyhow::Error),
    BufferDeletionError(anyhow::Error),
    BufferBindError(anyhow::Error),
    BufferNotFound,
    PipelineBindingNotFound,
}

impl Buffer {
    pub fn new(id: u32) -> Buffer { Buffer { id } }

    pub fn id(&self) -> u32 { self.id }
}

pub struct BufferContent<'a> {
    pub data: &'a [u8],
    pub align: u32,
    pub len: u32,
}

impl<'a> BufferContent<'a> {
    pub fn from_slice<T: bytemuck::NoUninit>(data: &'a [T]) -> Self {
        BufferContent {
            data: bytemuck::cast_slice(data),
            align: std::mem::align_of::<T>() as u32,
            len: data.len() as u32,
        }
    }
}
