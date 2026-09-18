use crate::{
    assets::Asset,
    ecs::resources::Resources,
    error::OptionNoneErr,
    graphics::{
        ActiveGraphicsBackend,
        shader_module::{
            ShaderModule, ShaderModuleBuilder, ShaderType,
            reinterpret_shader_code,
        },
    },
};

#[derive(Debug)]
pub struct InputShader {
    pub shader_extension: String,
    pub shader_code: String
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

impl Asset for Shader {
    type Input = InputShader;
    type Cooked = CookedShader;

    const ASSET_TYPE: &'static str = "shader";

    fn cook(input_data: Self::Input) -> Self::Cooked {
        let compiler = shaderc::Compiler::new().unwrap();
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
            .unwrap();
        let shader_type = match input_data.shader_extension.as_str() {
            "vert" => ShaderType::Vertex,
            "frag" => ShaderType::Fragment,
            _ => panic!("invalid shader extension"),
        };
        let code = reinterpret_shader_code(spv.as_binary_u8()).unwrap();
        return CookedShader { shader_type, spirv_compiled_code: code };
    }

    fn load(cooked: Self::Cooked, resources: &mut Resources) -> Self {
        let mut backend = resources
            .get_mut::<ActiveGraphicsBackend>()
            .none_err()
            .unwrap();
        let shader_module = ShaderModuleBuilder::default()
            .shader_type(cooked.shader_type)
            .code(&cooked.spirv_compiled_code)
            .build(&mut backend)
            .unwrap();
        Shader {
            shader_type: cooked.shader_type,
            module: shader_module,
        }
    }
}
