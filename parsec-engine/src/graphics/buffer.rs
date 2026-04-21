use crate::{
    error::{ParsecError, StrError},
    graphics::ActiveGraphicsBackend,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferHandle {
    id: u32,
}

impl BufferHandle {
    pub fn new(id: u32) -> Self { Self { id } }
    pub fn id(&self) -> u32 { self.id }
}

#[derive(Debug)]
pub struct Buffer {
    handle: BufferHandle,
    usage: Vec<BufferUsage>,
}

impl Buffer {
    fn new(handle: BufferHandle, usage: Vec<BufferUsage>) -> Buffer {
        Buffer { handle, usage }
    }

    pub fn handle(&self) -> BufferHandle { self.handle }

    pub fn id(&self) -> u32 { self.handle.id }

    pub fn usage(&self) -> &[BufferUsage] { &self.usage }

    pub fn destroy(
        self,
        backend: &mut ActiveGraphicsBackend,
    ) -> Result<(), BufferError> {
        backend.delete_buffer(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BufferUsage {
    Uniform,
    Vertex,
    Index,
    Storage,
    TransferSrc,
    TransferDst,
}

pub struct BufferBuilder<'a> {
    data: Option<BufferContent<'a>>,
    usage: &'a [BufferUsage],
}

impl<'a> BufferBuilder<'a> {
    pub fn new() -> Self {
        Self {
            data: None,
            usage: &[],
        }
    }

    pub fn data(mut self, data: BufferContent<'a>) -> Self {
        self.data = Some(data);
        self
    }

    pub fn usage(mut self, usage: &'a [BufferUsage]) -> Self {
        self.usage = usage;
        self
    }

    pub fn build(
        self,
        backend: &mut ActiveGraphicsBackend,
    ) -> Result<Buffer, BufferError> {
        let data = self.data.ok_or(BufferError::BufferCreationError(
            StrError("no buffer data provided").into(),
        ))?;
        let handle = backend.create_buffer(data, self.usage)?;
        Ok(Buffer::new(handle, self.usage.to_vec()))
    }
}

pub struct BufferContent<'a> {
    pub data: &'a [u8],
    pub align: u32,
    pub len: u32,
}

impl<'a> BufferContent<'a> {
    pub fn from_slice<T: Copy>(data: &'a [T]) -> Self {
        let ptr = data.as_ptr();
        let len = size_of::<T>().next_multiple_of(align_of::<T>()) * data.len();
        let bytes = unsafe {
            std::slice::from_raw_parts(ptr as *const u8, len)
        };
        BufferContent {
            data: bytes,
            align: std::mem::align_of::<T>() as u32,
            len: data.len() as u32,
        }
    }
}

#[derive(Debug)]
pub enum BufferError {
    BufferCreationError(ParsecError),
    BufferUpdateError(ParsecError),
    BufferDeletionError(ParsecError),
    BufferBindError(ParsecError),
    BufferNotFound,
    PipelineBindingNotFound,
}
