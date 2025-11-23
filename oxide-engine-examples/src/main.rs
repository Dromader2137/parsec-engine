use std::sync::{Arc, Mutex};

use oxide_engine::{
    app::App,
    ecs::{system::SystemTrigger, world::{WORLD, World}},
    graphics::{
        GraphicsBundle,
        renderer::{
            DefaultVertex,
            assets::mesh::Mesh,
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
    math::vec::Vec3f,
    resources::Resource,
    system,
    utils::id_vec::IdVec,
};

#[system]
fn test_system(
    device: Resource<Arc<Device>>,
    framebuffers: Resource<Vec<Arc<Framebuffer>>>,
    mut materials: Resource<IdVec<MaterialData>>,
    mut meshes: Resource<IdVec<Mesh>>,
) {
    let scale = 1.0;

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

    let vertices = vec![
        DefaultVertex::new(Vec3f::new(0.0, 0.0, 0.0) * scale, Vec3f::new(0.0, 0.0, 0.0)),
        DefaultVertex::new(Vec3f::new(1.0, 1.0, 0.0) * scale, Vec3f::new(1.0, 1.0, 0.0)),
        DefaultVertex::new(Vec3f::new(0.0, 1.0, 0.0) * scale, Vec3f::new(0.0, 1.0, 0.0)),
        DefaultVertex::new(Vec3f::new(1.0, 0.0, 0.0) * scale, Vec3f::new(1.0, 0.0, 0.0)),
    ];

    let indices = vec![0, 2, 1, 0, 1, 3];

    let mesh = meshes.push(Mesh::new(vertices, indices));

    world
        .spawn((
            Camera::new(40.0_f32.to_radians(), 1.0, 100.0),
            Transform::new(Vec3f::ZERO, Vec3f::ZERO, Vec3f::ZERO),
        ))
        .unwrap();

    world
        .spawn((
            Transform::new(
                Vec3f::FORWARD * 5.0 + Vec3f::new(0.5, -0.5, 0.0) * scale,
                Vec3f::ZERO,
                Vec3f::ZERO,
            ),
            MeshRenderer::new(mesh, material_id),
        ))
        .unwrap();

    world
        .spawn((
            Transform::new(
                Vec3f::FORWARD * 5.0 + Vec3f::new(-0.5, -0.5, 0.0) * scale,
                Vec3f::ZERO,
                Vec3f::ZERO,
            ),
            MeshRenderer::new(mesh, material_id),
        ))
        .unwrap();
}

fn main() {
    let mut app = App::new();
    app.systems.add_bundle(GraphicsBundle::default());
    app.systems.add(SystemTrigger::LateStart, TestSystem::new());
    app.run();
}
