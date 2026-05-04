use image::EncodableLayout;
use parsec_engine::{
    app::App,
    ecs::{
        resources::Resource,
        system::{SystemTrigger, requests::Requests, system},
        world::{component::Component, fetch::Mut, query::Query},
    },
    error::ParsecError,
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
        shader::{ShaderBuilder, ShaderType},
        window::Window,
    },
    graphics_bundle::GraphicsBundle,
    input::{Input, InputBundle},
    math::{quat::Quat, uvec::Vec2u, vec::Vec3f},
    renderer::{
        RendererMainRenderpass,
        assets::mesh::{Mesh, obj::load_obj},
        components::{
            camera::Camera, light::Light, mesh_renderer::MeshRenderer,
            transform::Transform,
        },
        material_data::{MaterialBase, MaterialData, MaterialPipelineBinding},
    },
    time::{Time, TimeBundle},
    utils::identifiable::IdStore,
    vulkan::shader::read_shader_code,
};

#[system]
fn test_system(
    mut requests: Resource<Requests>,
    mut backend: Resource<ActiveGraphicsBackend>,
    mut materials: Resource<IdStore<MaterialData>>,
    mut material_bases: Resource<IdStore<MaterialBase>>,
    mut meshes: Resource<IdStore<Mesh>>,
    renderpass: Resource<RendererMainRenderpass>,
) -> Result<(), ParsecError> {
    let vertex = ShaderBuilder::new()
        .code(&read_shader_code("shaders/simple.spv")?)
        .shader_type(ShaderType::Vertex)
        .build(&mut backend)
        .unwrap();
    let fragment = ShaderBuilder::new()
        .code(&read_shader_code("shaders/multilight.spv")?)
        .shader_type(ShaderType::Fragment)
        .build(&mut backend)
        .unwrap();

    let material_base = MaterialBase::new(
        &mut backend,
        vertex.handle(),
        fragment.handle(),
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

    let mesh = meshes.push(load_obj("test.obj").unwrap());

    requests.spawn_entity((
        Camera::new(40.0_f32.to_radians(), 0.1, 100.0),
        Transform::new(Vec3f::BACK, Vec3f::ZERO, Quat::IDENTITY),
        CameraController {
            yaw: 0.0,
            target_yaw: 0.0,
            pitch: 0.0,
            target_pitch: 0.0,
            fov: 40.0,
        },
    ));

    for _ in 0..20 {
        requests.spawn_entity((
            Transform::new(
                Vec3f::new(
                    rand::random_range(-10.0..10.0),
                    rand::random_range(-10.0..10.0),
                    rand::random_range(-10.0..10.0),
                ),
                Vec3f::ONE,
                Quat::IDENTITY,
            ),
            MeshRenderer::new(mesh, material_id),
        ));
    }
    requests.spawn_entity((
        Transform::new(Vec3f::ONE * 20.0, Vec3f::ONE, Quat::IDENTITY),
        Light::new(
            (Vec3f::ONE * -1.0).normalize(),
            Vec3f::new(-1.0, 1.0, -1.0).normalize(),
            Vec3f::ONE * 0.8,
        ),
        // OrbitingLight {
        //     orbit_radius: 20.0,
        //     speed: 0.1,
        //     offset: std::f32::consts::PI / 0.5,
        // },
    ));
    requests.spawn_entity((
        Transform::new(
            Vec3f::new(1.0, 1.0, -1.0) * 20.0,
            Vec3f::ONE,
            Quat::IDENTITY,
        ),
        Light::new(
            Vec3f::new(-1.0, -1.0, 1.0).normalize(),
            Vec3f::new(-1.0, 1.0, 1.0).normalize(),
            Vec3f::ONE * 0.2,
        ),
        // OrbitingLight {
        //     orbit_radius: 20.0,
        //     speed: 0.1,
        //     offset: 0.0,
        // },
    ));

    Ok(())
}

#[derive(Debug, Component)]
pub struct OrbitingLight {
    orbit_radius: f32,
    speed: f32,
    offset: f32,
}

#[derive(Debug, Component)]
pub struct CameraController {
    yaw: f32,
    target_yaw: f32,
    pitch: f32,
    target_pitch: f32,
    fov: f32,
}

#[system]
fn camera_movement(
    mut cameras: Query<(Mut<Transform>, Mut<Camera>, Mut<CameraController>)>,
    mut window: Resource<Window>,
    input: Resource<Input>,
    time: Resource<Time>,
) {
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
}

#[system]
fn light_mover(
    mut movable_lights: Query<(Mut<Transform>, Mut<Light>, OrbitingLight)>,
    time: Resource<Time>,
) {
    for (_, (tra, lig, mov)) in movable_lights.iter() {
        tra.position.x = (time.elapsed_time().unwrap() * mov.speed as f64
            + mov.offset as f64)
            .sin() as f32
            * mov.orbit_radius;
        tra.position.z = (time.elapsed_time().unwrap() * mov.speed as f64
            + mov.offset as f64)
            .cos() as f32
            * mov.orbit_radius;
        tra.position.y = mov.orbit_radius;
        lig.direction = tra.position.normalize() * -1.0;
        lig.up = (tra.position * Vec3f::new(1.0, -1.0, 1.0)).normalize();
    }
}

fn main() {
    let mut app = App::new();
    app.systems.add_bundle(GraphicsBundle::default());
    app.systems.add_bundle(InputBundle::default());
    app.systems.add_bundle(TimeBundle);
    app.systems.add(SystemTrigger::LateStart, TestSystem::new());
    app.systems
        .add(SystemTrigger::Update, CameraMovement::new());
    app.systems.add(SystemTrigger::Update, LightMover::new());
    app.run();
}
