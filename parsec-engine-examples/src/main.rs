use image::EncodableLayout;
use parsec_engine::{
    app::App,
    ecs::{
        system::{SystemTrigger, requests::Requests, system},
        world::{component::Component, fetch::Mut, query::Query},
    },
    error::ParsecError,
    graphics::{
        ActiveGraphicsBackend, GraphicsBundle,
        buffer::{BufferContent, BufferUsage},
        image::{ImageAspect, ImageFormat, ImageUsage},
        pipeline::{
            DefaultVertex, PipelineBindingType, PipelineCullingMode,
            PipelineOptions, PipelineResourceBindingLayout,
            PipelineShaderStage,
        },
        renderer::{
            RendererMainRenderpass,
            assets::mesh::{Mesh, obj::load_obj},
            components::{
                camera::Camera, mesh_renderer::MeshRenderer,
                transform::Transform,
            },
            material_data::{
                MaterialBase, MaterialData, MaterialPipelineBinding,
            },
        },
        shader::ShaderType,
        vulkan::shader::read_shader_code,
        window::Window,
    },
    input::{Input, InputBundle},
    math::{mat::Matrix4f, quat::Quat, uvec::Vec2u, vec::Vec3f},
    resources::Resource,
    time::{Time, TimeBundle},
    utils::identifiable::IdStore,
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
    let vertex = backend
        .create_shader(
            &read_shader_code("shaders/simple.spv")?,
            ShaderType::Vertex,
        )
        .unwrap();
    let fragment = backend
        .create_shader(
            &read_shader_code("shaders/flat.spv")?,
            ShaderType::Fragment,
        )
        .unwrap();

    let material_base = MaterialBase::new(
        &mut *backend,
        vertex,
        fragment,
        renderpass.0,
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
                PipelineBindingType::UniformBuffer,
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
    let texture_buffer = backend
        .create_buffer(BufferContent::from_slice(image_data), &[
            BufferUsage::TransferSrc,
        ])
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
            texture_buffer,
            texture_image,
            Vec2u::new(width, height),
            Vec2u::ZERO,
        )
        .unwrap();
    let texture_binding_layout = backend
        .create_pipeline_resource_layout(&[PipelineResourceBindingLayout::new(
            PipelineBindingType::TextureSampler,
            &[PipelineShaderStage::Fragment],
        )])
        .unwrap();
    let texture_binding = backend
        .create_pipeline_resource(texture_binding_layout)
        .unwrap();
    let texture_sampler = backend.create_image_sampler().unwrap();
    let texture_image_view = backend.create_image_view(texture_image).unwrap();
    backend
        .bind_sampler(texture_binding, texture_sampler, texture_image_view, 0)
        .unwrap();

    let material = MaterialData::new(&material_base, vec![
        MaterialPipelineBinding::Model,
        MaterialPipelineBinding::View,
        MaterialPipelineBinding::Projection,
        MaterialPipelineBinding::Light,
        MaterialPipelineBinding::Generic(texture_binding),
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

    for _ in 0..50 {
        requests.spawn_entity((
            Transform::new(
                Vec3f::new(
                    rand::random_range(-10.0..10.0),
                    rand::random_range(-10.0..10.0),
                    rand::random_range(-10.0..10.0),
                ),
                Vec3f::ONE * rand::random::<f32>(),
                Quat::from_euler(Vec3f::new(
                    rand::random_range(0.0..std::f32::consts::PI),
                    rand::random_range(0.0..std::f32::consts::PI),
                    rand::random_range(0.0..std::f32::consts::PI),
                )),
            ),
            MeshRenderer::new(mesh, material_id),
        ));
    }

    Ok(())
}

#[derive(Debug, Component)]
pub struct Movable {
    base_pos: Vec3f,
    offset: f32,
    speed: f32,
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
fn box_mover(
    mut movable_boxes: Query<(Mut<Transform>, Movable)>,
    time: Resource<Time>,
) {
    for (_, (tra, mov)) in movable_boxes.iter() {
        tra.position.y = mov.base_pos.y
            + (time
                .current_time()
                .duration_since(time.start_time())
                .unwrap()
                .as_millis() as f32
                / 1000.0
                * mov.speed
                + mov.offset)
                .sin();
    }
}

fn main() {
    let mut app = App::new();
    app.systems.add_bundle(GraphicsBundle::default());
    app.systems.add_bundle(InputBundle::default());
    app.systems.add_bundle(TimeBundle::default());
    app.systems.add(SystemTrigger::LateStart, TestSystem::new());
    app.systems
        .add(SystemTrigger::Update, CameraMovement::new());
    app.systems.add(SystemTrigger::Update, BoxMover::new());
    app.run();
}
