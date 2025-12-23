use image::EncodableLayout;
use parsec_engine::{
    app::App,
    ecs::{
        system::{SystemTrigger, system},
        world::{World, component::Component, fetch::Mut, query::Query},
    },
    graphics::{
        GraphicsBundle,
        backend::GraphicsBackend,
        buffer::BufferUsage,
        image::{ImageFormat, ImageUsage},
        pipeline::{
            PipelineBindingType, PipelineShaderStage, PipelineSubbindingLayout,
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
        vulkan::{VulkanBackend, shader::read_shader_code},
        window::Window,
    },
    input::{Input, InputBundle},
    math::{quat::Quat, vec::Vec3f},
    resources::Resource,
    time::{Time, TimeBundle},
    utils::id_vec::IdVec,
};

#[system]
fn test_system(
    mut backend: Resource<VulkanBackend>,
    mut materials: Resource<IdVec<MaterialData>>,
    mut material_bases: Resource<IdVec<MaterialBase>>,
    mut meshes: Resource<IdVec<Mesh>>,
    renderpass: Resource<RendererMainRenderpass>,
) {
    let vertex = backend
        .create_shader(
            &read_shader_code("shaders/simple.spv").unwrap(),
            ShaderType::Vertex,
        )
        .unwrap();
    let fragment = backend
        .create_shader(
            &read_shader_code("shaders/flat.spv").unwrap(),
            ShaderType::Fragment,
        )
        .unwrap();

    let material_base =
        MaterialBase::new(&mut *backend, vertex, fragment, renderpass.0, vec![
            vec![
                PipelineSubbindingLayout::new(
                    PipelineBindingType::UniformBuffer,
                    PipelineShaderStage::Vertex,
                ),
                PipelineSubbindingLayout::new(
                    PipelineBindingType::UniformBuffer,
                    PipelineShaderStage::Vertex,
                ),
                PipelineSubbindingLayout::new(
                    PipelineBindingType::UniformBuffer,
                    PipelineShaderStage::Vertex,
                ),
            ],
            vec![PipelineSubbindingLayout::new(
                PipelineBindingType::UniformBuffer,
                PipelineShaderStage::Vertex,
            )],
            vec![PipelineSubbindingLayout::new(
                PipelineBindingType::UniformBuffer,
                PipelineShaderStage::Vertex,
            )],
            vec![PipelineSubbindingLayout::new(
                PipelineBindingType::UniformBuffer,
                PipelineShaderStage::Fragment,
            )],
            vec![PipelineSubbindingLayout::new(
                PipelineBindingType::TextureSampler,
                PipelineShaderStage::Fragment,
            )],
            vec![PipelineSubbindingLayout::new(
                PipelineBindingType::TextureSampler,
                PipelineShaderStage::Fragment,
            )],
        ]);

    let image = image::load_from_memory(include_bytes!("../../test.png"))
        .unwrap()
        .to_rgba8();
    let (width, height) = image.dimensions();
    let image_data = image.as_raw().as_bytes();
    let texture_buffer = backend
        .create_buffer(image_data, &[BufferUsage::TransferSrc])
        .unwrap();
    let texture_image = backend
        .create_image((width, height), ImageFormat::RGBA8SRGB, &[
            ImageUsage::Sampled,
            ImageUsage::ColorBuffer,
            ImageUsage::Dst,
        ])
        .unwrap();
    backend
        .load_image_from_buffer(texture_buffer, texture_image)
        .unwrap();
    let texture_binding_layout = backend
        .create_pipeline_binding_layout(&[PipelineSubbindingLayout::new(
            PipelineBindingType::TextureSampler,
            PipelineShaderStage::Fragment,
        )])
        .unwrap();
    let texture_binding = backend
        .create_pipeline_binding(texture_binding_layout)
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

    World::spawn((
        Camera::new(40.0_f32.to_radians(), 0.1, 100.0),
        Transform::new(Vec3f::UP, Vec3f::ZERO, Quat::IDENTITY),
        CameraController {
            yaw: 0.0,
            target_yaw: 0.0,
            pitch: 0.0,
            target_pitch: 0.0,
            fov: 40.0,
        },
    ))
    .unwrap();

    World::spawn((
        Transform::new(Vec3f::ZERO, Vec3f::ONE, Quat::IDENTITY),
        MeshRenderer::new(mesh, material_id),
        Movable  { base_pos: Vec3f::ZERO, offset: 0.0, speed: 1.0 },
    ))
    .unwrap();
    
    World::spawn((
        Transform::new(Vec3f::new(-2.0, 2.0, -2.0), Vec3f::ONE * 0.4, Quat::IDENTITY),
        MeshRenderer::new(mesh, material_id),
        Movable  { base_pos: Vec3f::new(-2.0, 2.0, -2.0), offset: 1.0, speed: 2.5 },
    ))
    .unwrap();

    World::spawn((
        Transform::new(
            Vec3f::new(3.0, -3.0, 3.0),
            Vec3f::ONE * 1.5,
            Quat::IDENTITY,
        ),
        MeshRenderer::new(mesh, material_id),
    ))
    .unwrap();
}

#[derive(Debug, Component)]
pub struct Movable { base_pos: Vec3f, offset: f32, speed: f32 }

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
        tra.position.y = mov.base_pos.y + (time
            .current_time()
            .duration_since(time.start_time())
            .unwrap()
            .as_millis() as f32 / 1000.0 * mov.speed + mov.offset).sin();
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
    app.systems
        .add(SystemTrigger::Update, BoxMover::new());
    app.run();
}
