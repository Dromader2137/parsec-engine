use std::sync::Arc;

use oxide_engine::{
    app::App,
    ecs::{
        system::{SystemTrigger, system},
        world::{World, component::Component, fetch::Mut, query::Query},
    },
    graphics::{
        GraphicsBundle,
        renderer::{
            assets::mesh::{Mesh, obj::load_obj},
            components::{camera::Camera, mesh_renderer::MeshRenderer, transform::Transform},
            material_data::{MaterialBase, MaterialData, MaterialDescriptorSets},
        },
        vulkan::{
            descriptor_set::{DescriptorSetBinding, DescriptorStage, DescriptorType},
            device::Device,
            framebuffer::Framebuffer,
            shader::{ShaderModule, ShaderType, read_shader_code},
        },
    },
    input::{Input, InputBundle, key::KeyCode},
    math::{quat::Quat, vec::Vec3f},
    resources::Resource,
    utils::id_vec::IdVec,
};

#[system]
fn test_system(
    device: Resource<Arc<Device>>,
    framebuffers: Resource<Vec<Arc<Framebuffer>>>,
    mut materials: Resource<IdVec<MaterialData>>,
    mut meshes: Resource<IdVec<Mesh>>,
) {
    let scale = 0.01;

    let vertex = ShaderModule::new(
        device.clone(),
        &read_shader_code("shaders/simple.spv").unwrap(),
        ShaderType::Vertex,
    )
    .unwrap();

    let fragment = ShaderModule::new(
        device.clone(),
        &read_shader_code("shaders/flat.spv").unwrap(),
        ShaderType::Fragment,
    )
    .unwrap();

    let material_base = MaterialBase::new(framebuffers.to_vec(), vertex, fragment, vec![
        vec![
            DescriptorSetBinding::new(0, DescriptorType::UNIFORM_BUFFER, DescriptorStage::VERTEX),
            DescriptorSetBinding::new(1, DescriptorType::UNIFORM_BUFFER, DescriptorStage::VERTEX),
            DescriptorSetBinding::new(2, DescriptorType::UNIFORM_BUFFER, DescriptorStage::VERTEX),
        ],
        vec![DescriptorSetBinding::new(
            0,
            DescriptorType::UNIFORM_BUFFER,
            DescriptorStage::VERTEX,
        )],
        vec![DescriptorSetBinding::new(
            0,
            DescriptorType::UNIFORM_BUFFER,
            DescriptorStage::VERTEX,
        )],
    ])
    .unwrap();

    let material = MaterialData::new(material_base, vec![
        MaterialDescriptorSets::ModelMatrixSet,
        MaterialDescriptorSets::ViewMatrixSet,
        MaterialDescriptorSets::ProjectionMatrixSet,
    ])
    .unwrap();

    let material_id = materials.push(material);

    let mesh = meshes.push(load_obj("sponza.obj").unwrap());

    World::spawn((
        Camera::new(40.0_f32.to_radians(), 0.1, 100.0),
        Transform::new(
            Vec3f::UP * 2.5,
            Vec3f::ZERO,
            Quat::from_euler(Vec3f::new(0.3, 0.0, 0.0)),
        ),
        CameraController {
            yaw: 0.0,
            pitch: 0.0,
        },
    ))
    .unwrap();

    World::spawn((
        Transform::new(
            Vec3f::ZERO,
            Vec3f::ONE * scale,
            Quat::from_euler(Vec3f::new(0.0, 3.14, 0.0)),
        ),
        MeshRenderer::new(mesh, material_id),
    ))
    .unwrap();
}

#[derive(Debug, Component)]
pub struct CameraController {
    yaw: f32,
    pitch: f32,
}

#[system]
fn camera_movement(
    mut cameras: Query<(Mut<Transform>, Camera, Mut<CameraController>)>,
    input: Resource<Input>,
) {
    for (_, (transform, _, camera_controller)) in cameras.iter() {
        if input.keys.is_down(KeyCode::KeyD) {
            transform.position += Vec3f::FORWARD * transform.rotation / 100.0;
        }
        if input.keys.is_down(KeyCode::KeyS) {
            transform.position += Vec3f::BACK * transform.rotation / 100.0;
        }
        if input.keys.is_down(KeyCode::KeyA) {
            transform.position += Vec3f::LEFT * transform.rotation / 100.0;
        }
        if input.keys.is_down(KeyCode::KeyH) {
            transform.position += Vec3f::RIGHT * transform.rotation / 100.0;
        }
        if input.keys.is_down(KeyCode::Space) {
            transform.position += Vec3f::UP * transform.rotation / 100.0;
        }
        if input.keys.is_down(KeyCode::ShiftLeft) {
            transform.position += Vec3f::DOWN * transform.rotation / 100.0;
        }
        let delta = input.mouse.delta();
        camera_controller.yaw += -delta.x / 100.0;
        camera_controller.pitch += delta.y / 100.0;
        transform.rotation = Quat::from_euler(Vec3f::new(
            camera_controller.pitch,
            camera_controller.yaw,
            0.0,
        ));
    }
}

fn main() {
    let mut app = App::new();
    app.systems.add_bundle(GraphicsBundle::default());
    app.systems.add_bundle(InputBundle::default());
    app.systems.add(SystemTrigger::LateStart, TestSystem::new());
    app.systems
        .add(SystemTrigger::Update, CameraMovement::new());
    app.run();
}
