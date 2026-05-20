use image::EncodableLayout;
use parsec_engine::{
    app::App,
    assets::core::{mesh::Mesh, shader::Shader},
    ctx::Ctx,
    ecs::{
        system::SystemTrigger,
        world::{component::Component, fetch::Mut},
    },
    error::{OptionNoneErr, ParsecError},
    graphics::{
        ActiveGraphicsBackend,
        buffer::{BufferBuilder, BufferContent, BufferUsage},
        image::{ImageAspect, ImageFormat, ImageUsage},
        pipeline::{
            DefaultVertex, PipelineBindingType, PipelineCullingMode,
            PipelineOptions, PipelineResourceBindingLayout,
            PipelineResourceLayoutBuilder, PipelineShaderStage,
        },
        sampler::SamplerBuilder,
        window::Window,
    },
    input::{Input, InputBundle},
    math::{quat::Quat, uvec::Vec2u, vec::Vec3f},
    renderer::{
        RendererMainRenderpass,
        components::{
            camera::Camera, light::Light, mesh_renderer::MeshRenderer,
            transform::Transform,
        },
        graphics_bundle::GraphicsBundle,
        material_data::{MaterialBase, MaterialData, MaterialPipelineBinding},
    },
    time::{Time, TimeBundle},
    utils::identifiable::IdStore,
};
use parsec_engine_vulkan::VulkanBackend;

fn test_system(ctx: Ctx) -> Result<(), ParsecError> {
    let vertex_shader_handle =
        ctx.assets.load::<Shader>("shaderv", ctx.resources)?;
    let fragment_shader_handle =
        ctx.assets.load::<Shader>("shaderf", ctx.resources)?;
    let mesh_handle =
        ctx.assets.load::<Mesh>("testmesh", ctx.resources).unwrap();
    
    let mut backend =
        ctx.resources.get_mut::<ActiveGraphicsBackend>().none_err()?;
    let mut materials =
        ctx.resources.get_mut::<IdStore<MaterialData>>().none_err()?;
    let mut material_bases =
        ctx.resources.get_mut::<IdStore<MaterialBase>>().none_err()?;
    let renderpass =
        ctx.resources.get::<RendererMainRenderpass>().none_err()?;

    let material_base = MaterialBase::new(
        &mut backend,
        ctx.assets.get(vertex_shader_handle).none_err()?.module.handle(),
        ctx.assets.get(fragment_shader_handle).none_err()?.module.handle(),
        renderpass.0.handle(),
        vec![
            vec![
                PipelineResourceBindingLayout::new(
                    PipelineBindingType::UniformBuffer,
                    &[PipelineShaderStage::Vertex],
                ),
                PipelineResourceBindingLayout::new(
                    PipelineBindingType::UniformBuffer,
                    &[PipelineShaderStage::Vertex],
                ),
                PipelineResourceBindingLayout::new(
                    PipelineBindingType::UniformBuffer,
                    &[PipelineShaderStage::Vertex],
                ),
            ],
            vec![PipelineResourceBindingLayout::new(
                PipelineBindingType::UniformBuffer,
                &[PipelineShaderStage::Vertex],
            )],
            vec![PipelineResourceBindingLayout::new(
                PipelineBindingType::UniformBuffer,
                &[PipelineShaderStage::Vertex],
            )],
            vec![PipelineResourceBindingLayout::new(
                PipelineBindingType::StorageBuffer,
                &[PipelineShaderStage::Fragment, PipelineShaderStage::Vertex],
            )],
            vec![PipelineResourceBindingLayout::new(
                PipelineBindingType::TextureSampler,
                &[PipelineShaderStage::Fragment],
            )],
            vec![PipelineResourceBindingLayout::new(
                PipelineBindingType::TextureSampler,
                &[PipelineShaderStage::Fragment],
            )],
        ],
        PipelineOptions::new::<DefaultVertex>(PipelineCullingMode::CullBack),
    );

    let image =
        image::load_from_memory(include_bytes!("../../test.png"))?.to_rgba8();
    let (width, height) = image.dimensions();
    let image_data = image.as_raw().as_bytes();
    let texture_buffer = BufferBuilder::new()
        .usage(&[BufferUsage::TransferSrc])
        .data(BufferContent::from_slice(image_data))
        .build(&mut backend)
        .unwrap();
    let texture_image = backend
        .create_image(
            Vec2u::new(width, height),
            ImageFormat::RGBA8SRGB,
            ImageAspect::Color,
            &[ImageUsage::Sampled, ImageUsage::TransferDst],
        )
        .unwrap();
    backend
        .load_image_from_buffer(
            texture_buffer.handle(),
            texture_image,
            Vec2u::new(width, height),
            Vec2u::ZERO,
        )
        .unwrap();
    let texture_binding = PipelineResourceLayoutBuilder::new()
        .bindings(&[PipelineResourceBindingLayout::new(
            PipelineBindingType::TextureSampler,
            &[PipelineShaderStage::Fragment],
        )])
        .build(&mut backend)
        .unwrap()
        .create_resource(&mut backend)
        .unwrap();
    let texture_sampler = SamplerBuilder::new().build(&mut backend).unwrap();
    let texture_image_view = backend.create_image_view(texture_image).unwrap();
    backend
        .bind_sampler(
            texture_binding.handle(),
            texture_sampler.handle(),
            texture_image_view,
            0,
        )
        .unwrap();

    let material = MaterialData::new(&material_base, vec![
        MaterialPipelineBinding::Model,
        MaterialPipelineBinding::View,
        MaterialPipelineBinding::Projection,
        MaterialPipelineBinding::Light,
        MaterialPipelineBinding::Generic(texture_binding.handle()),
        MaterialPipelineBinding::ShadowMap,
    ]);

    material_bases.push(material_base);
    let material_id = materials.push(material);

    ctx.world.spawn((
        Camera::new(40.0_f32.to_radians(), 0.1, 100.0),
        Transform::new(Vec3f::BACK, Vec3f::ZERO, Quat::IDENTITY),
        CameraController {
            yaw: 0.0,
            target_yaw: 0.0,
            pitch: 0.0,
            target_pitch: 0.0,
            fov: 40.0,
        },
    ))?;
    ctx.world.spawn((
        Transform::new(Vec3f::ZERO, Vec3f::ONE, Quat::IDENTITY),
        MeshRenderer::new(mesh_handle, material_id),
    ))?;
    ctx.world.spawn((
        Transform::new(Vec3f::ONE * 20.0, Vec3f::ONE, Quat::IDENTITY),
        Light::new(
            (Vec3f::ONE * -1.0).normalize(),
            Vec3f::new(-1.0, 1.0, -1.0).normalize(),
            Vec3f::ONE * 0.9,
        ),
    ))?;

    Ok(())
}

#[derive(Debug, Component)]
struct CameraController {
    yaw: f32,
    target_yaw: f32,
    pitch: f32,
    target_pitch: f32,
    fov: f32,
}

fn controller(ctx: Ctx) -> Result<(), ParsecError> {
    let mut cameras =
        ctx.world
            .query::<(Mut<Transform>, Mut<Camera>, Mut<CameraController>)>();
    let mut window = ctx.resources.get_mut::<Window>().none_err()?;
    let input = ctx.resources.get::<Input>().none_err()?;
    let time = ctx.resources.get::<Time>().none_err()?;

    for (_, (transform, camera, camera_controller)) in cameras.iter() {
        let delta = input.mouse.positon_delta();
        camera_controller.target_yaw += -delta.x / window.width() as f32 * 10.0;
        camera_controller.target_pitch +=
            delta.y / window.width() as f32 * 10.0;
        camera_controller.target_pitch =
            camera_controller.target_pitch.clamp(-1.57, 1.57);
        camera_controller.pitch = camera_controller.target_pitch * 0.3
            + camera_controller.pitch * 0.7;
        camera_controller.yaw =
            camera_controller.target_yaw * 0.3 + camera_controller.yaw * 0.7;
        transform.rotation = Quat::from_euler(Vec3f::new(
            camera_controller.pitch,
            camera_controller.yaw,
            0.0,
        ));
        camera_controller.fov -= input.mouse.wheel_delta().y;
        camera_controller.fov = camera_controller.fov.clamp(30.0, 60.0);
        camera.vertical_fov = camera_controller.fov.to_radians();
        let rotation =
            Quat::from_euler(Vec3f::new(0.0, camera_controller.yaw, 0.0));
        let movement_speed = 5.0;
        if input.keys.is_down("r") {
            transform.position +=
                Vec3f::FORWARD * rotation * time.delta_time() * movement_speed;
        }
        if input.keys.is_down("h") {
            transform.position +=
                Vec3f::BACK * rotation * time.delta_time() * movement_speed;
        }
        if input.keys.is_down("s") {
            transform.position +=
                Vec3f::LEFT * rotation * time.delta_time() * movement_speed;
        }
        if input.keys.is_down("t") {
            transform.position +=
                Vec3f::RIGHT * rotation * time.delta_time() * movement_speed;
        }
        if input.keys.is_down("a") {
            transform.position +=
                Vec3f::UP * rotation * time.delta_time() * movement_speed;
        }
        if input.keys.is_down("z") {
            transform.position +=
                Vec3f::DOWN * rotation * time.delta_time() * movement_speed;
        }
        if input.keys.is_pressed("c") {
            window.toggle_cursor_lock().unwrap();
            window.toggle_cursor_visibility();
        }
    }
    Ok(())
}

fn main() {
    let mut app = App::new();
    app.systems
        .add_bundle(GraphicsBundle::<VulkanBackend>::default());
    app.systems.add_bundle(InputBundle);
    app.systems.add_bundle(TimeBundle);
    app.systems.add(SystemTrigger::LateStart, test_system);
    app.systems.add(SystemTrigger::Update, controller);
    app.run();
}
