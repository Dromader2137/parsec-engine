use std::{
    fs::File,
    io::{Read, Seek},
};

use crate::{error::ParsecError, graphics::ActiveGraphicsBackend};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShaderHandle {
    id: u32,
}

impl ShaderHandle {
    pub fn new(id: u32) -> Self { Self { id } }
    pub fn id(&self) -> u32 { self.id }
}

#[derive(Debug)]
pub struct Shader {
    handle: ShaderHandle,
    shader_type: ShaderType,
}

impl Shader {
    fn new(handle: ShaderHandle, shader_type: ShaderType) -> Self {
        Self {
            handle,
            shader_type,
        }
    }

    pub fn handle(&self) -> ShaderHandle { self.handle }
    pub fn id(&self) -> u32 { self.handle.id }
    pub fn shader_type(&self) -> ShaderType { self.shader_type }

    pub fn destroy(
        self,
        backend: &mut ActiveGraphicsBackend,
    ) -> Result<(), ShaderError> {
        backend.delete_shader(self)
    }
}

pub struct ShaderBuilder<'a> {
    code: Option<&'a [u32]>,
    shader_type: ShaderType,
}

impl<'a> Default for ShaderBuilder<'a> {
    fn default() -> Self { Self::new() }
}

impl<'a> ShaderBuilder<'a> {
    pub fn new() -> Self {
        Self {
            code: None,
            shader_type: ShaderType::Vertex,
        }
    }

    pub fn code(mut self, code: &'a [u32]) -> Self {
        self.code = Some(code);
        self
    }

    pub fn shader_type(mut self, shader_type: ShaderType) -> Self {
        self.shader_type = shader_type;
        self
    }

    pub fn build(
        self,
        backend: &mut ActiveGraphicsBackend,
    ) -> Result<Shader, ShaderError> {
        let code = self.code.ok_or(ShaderError::MissingCode)?;
        let handle = backend.create_shader(code, self.shader_type)?;
        Ok(Shader::new(handle, self.shader_type))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ShaderError {
    #[error("failed to create shader: {0}")]
    ShaderCreationError(ParsecError),
    #[error("failed to delete shader: {0}")]
    ShaderDeletionError(ParsecError),
    #[error("shader does not exist")]
    ShaderNotFound,
    #[error("shader code not provided")]
    MissingCode,
    #[error("shader file parsing error")]
    ShaderFileError(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderType {
    // TODO maybe cleanup shadow material
    Vertex,
    Fragment,
}

pub fn read_shader_code(path: &str) -> Result<Vec<u32>, ShaderError> {
    let mut file = match File::open(path) {
        Ok(val) => val,
        Err(err) => return Err(ShaderError::ShaderFileError(err)),
    };

    let size = file.seek(std::io::SeekFrom::End(0))?;
    file.rewind()?;
    let words = size / 4;
    let mut result = vec![0u32; words as usize];
    file.read_exact(unsafe {
        std::slice::from_raw_parts_mut(
            result.as_mut_ptr().cast::<u8>(),
            words as usize * 4,
        )
    })?;
    const MAGIC_NUMBER: u32 = 0x0723_0203;
    if !result.is_empty() && result[0] == MAGIC_NUMBER.swap_bytes() {
        for word in &mut result {
            *word = word.swap_bytes();
        }
    }
    Ok(result)
}
