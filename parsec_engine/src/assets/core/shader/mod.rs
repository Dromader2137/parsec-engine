use crate::{
    assets::Asset,
    ecs::resources::Resources,
    graphics::{
        ActiveGraphicsBackend,
        shader_module::{
            ShaderError, ShaderModule, ShaderModuleBuilder, ShaderType, reinterpret_shader_code,
        },
    },
};

pub mod file_handler;

#[derive(Debug)]
pub struct InputShader {
    pub shader_extension: String,
    pub shader_code: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CookedShader {
    shader_type: ShaderType,
    spirv_compiled_code: Vec<u32>,
}

#[derive(Debug)]
pub struct Shader {
    pub shader_type: ShaderType,
    pub module: ShaderModule,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ShaderCookError {
    #[error("invalid shader extension '{0}'")]
    InvalidShaderExtension(String),
    #[error("failed to compile shader: {0}")]
    Compilation(shaderc::Error),
    #[error("failed to create compiler: {0}")]
    CompilerInitialization(shaderc::Error),
    #[error(transparent)]
    Reinterpret(#[from] ShaderError),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ShaderLoadError {
    #[error("graphics backend not found in resources")]
    GraphicsBackendNotFound,
    #[error(transparent)]
    Build(#[from] ShaderError),
}

impl Asset for Shader {
    type Input = InputShader;
    type Cooked = CookedShader;
    type CookError = ShaderCookError;
    type LoadError = ShaderLoadError;

    const ASSET_TYPE: &'static str = "shader";

    fn cook(input_data: Self::Input) -> Result<Self::Cooked, Self::CookError> {
        let compiler =
            shaderc::Compiler::new().map_err(ShaderCookError::CompilerInitialization)?;
        let spv = compiler
            .compile_into_spirv(
                &input_data.shader_code,
                match input_data.shader_extension.as_str() {
                    "vert" => shaderc::ShaderKind::DefaultVertex,
                    "frag" => shaderc::ShaderKind::DefaultFragment,
                    _ => panic!("invalid shader extension"),
                },
                "ast",
                "main",
                None,
            )
            .map_err(ShaderCookError::Compilation)?;
        let shader_type = match input_data.shader_extension.as_str() {
            "vert" => Ok(ShaderType::Vertex),
            "frag" => Ok(ShaderType::Fragment),
            val => Err(ShaderCookError::InvalidShaderExtension(val.to_string())),
        }?;
        let code = reinterpret_shader_code(spv.as_binary_u8())?;
        Ok(CookedShader { shader_type, spirv_compiled_code: code })
    }

    fn load(cooked: Self::Cooked, resources: &mut Resources) -> Result<Self, Self::LoadError> {
        let mut backend = resources
            .get_mut::<ActiveGraphicsBackend>()
            .ok_or(ShaderLoadError::GraphicsBackendNotFound)?;
        let shader_module = ShaderModuleBuilder::default()
            .shader_type(cooked.shader_type)
            .code(&cooked.spirv_compiled_code)
            .build(&mut backend)?;
        Ok(Shader { shader_type: cooked.shader_type, module: shader_module })
    }
}
