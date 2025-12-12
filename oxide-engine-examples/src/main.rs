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
            components::{
                camera::Camera, mesh_renderer::MeshRenderer,
                transform::Transform,
            },
            material_data::{
                MaterialBase, MaterialData, MaterialDescriptorSets,
            },
        },
        shader::ShaderType,
        vulkan::{
            descriptor_set::{
                DescriptorStage, DescriptorType, VulkanDescriptorSetBinding,
            },
            device::VulkanDevice,
            framebuffer::VulkanFramebuffer,
            renderpass::VulkanRenderpass,
            shader::{VulkanShaderModule, read_shader_code},
        },
    },
    input::{Input, InputBundle, key::Noncharacter},
    math::{quat::Quat, vec::Vec3f},
    resources::Resource,
    time::{Time, TimeBundle},
    utils::id_vec::IdVec,
};

#[system]
fn test_system(
    device: Resource<VulkanDevice>,
    framebuffers: Resource<Vec<VulkanFramebuffer>>,
    renderpass: Resource<VulkanRenderpass>,
    mut materials: Resource<IdVec<MaterialData>>,
    mut material_bases: Resource<IdVec<MaterialBase>>,
    mut meshes: Resource<IdVec<Mesh>>,
) {
    let scale = 0.01;

    let vertex = VulkanShaderModule::new(
        &device,
        &read_shader_code("shaders/simple.spv").unwrap(),
        ShaderType::Vertex,
    )
    .unwrap();

    let fragment = VulkanShaderModule::new(
        &device,
        &read_shader_code("shaders/flat.spv").unwrap(),
        ShaderType::Fragment,
    )
    .unwrap();

    let material_base = MaterialBase::new(
        &device,
        &renderpass,
        framebuffers.iter().collect(),
        &vertex,
        &fragment,
        vec![
            vec![
                VulkanDescriptorSetBinding::new(
                    0,
                    DescriptorType::UNIFORM_BUFFER,
                    DescriptorStage::VERTEX,
                ),
                VulkanDescriptorSetBinding::new(
                    1,
                    DescriptorType::UNIFORM_BUFFER,
                    DescriptorStage::VERTEX,
                ),
                VulkanDescriptorSetBinding::new(
                    2,
                    DescriptorType::UNIFORM_BUFFER,
                    DescriptorStage::VERTEX,
                ),
            ],
            vec![VulkanDescriptorSetBinding::new(
                0,
                DescriptorType::UNIFORM_BUFFER,
                DescriptorStage::VERTEX,
            )],
            vec![VulkanDescriptorSetBinding::new(
                0,
                DescriptorType::UNIFORM_BUFFER,
                DescriptorStage::VERTEX,
            )],
        ],
    )
    .unwrap();

    let material = MaterialData::new(&material_base, vec![
        MaterialDescriptorSets::ModelMatrixSet,
        MaterialDescriptorSets::ViewMatrixSet,
        MaterialDescriptorSets::ProjectionMatrixSet,
    ])
    .unwrap();

    material_bases.push(material_base);
    let material_id = materials.push(material);

    let mesh = meshes.push(load_obj("sponza.obj").unwrap());

    World::spawn((
        Camera::new(40.0_f32.to_radians(), 0.5, 30.0),
        Transform::new(
            Vec3f::UP * 2.5,
            Vec3f::ZERO,
            Quat::from_euler(Vec3f::new(0.3, 0.0, 0.0)),
        ),
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
    target_yaw: f32,
    pitch: f32,
    target_pitch: f32,
    fov: f32,
}

#[system]
fn camera_movement(
    mut cameras: Query<(Mut<Transform>, Mut<Camera>, Mut<CameraController>)>,
    input: Resource<Input>,
    time: Resource<Time>,
) {
    for (_, (transform, camera, camera_controller)) in cameras.iter() {
        let delta = input.mouse.positon_delta();
        camera_controller.target_yaw += -delta.x * time.delta_time() * 5.0;
        camera_controller.target_pitch += delta.y * time.delta_time() * 5.0;
        camera_controller.target_pitch =
            camera_controller.target_pitch.clamp(-1.57, 1.57);
        camera_controller.pitch = camera_controller.target_pitch * 0.2
            + camera_controller.pitch * 0.8;
        camera_controller.yaw =
            camera_controller.target_yaw * 0.2 + camera_controller.yaw * 0.8;
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
        if input.keys.is_down("d") {
            transform.position +=
                Vec3f::FORWARD * rotation * time.delta_time() * movement_speed;
        }
        if input.keys.is_down("s") {
            transform.position +=
                Vec3f::BACK * rotation * time.delta_time() * movement_speed;
        }
        if input.keys.is_down("a") {
            transform.position +=
                Vec3f::LEFT * rotation * time.delta_time() * movement_speed;
        }
        if input.keys.is_down("h") {
            transform.position +=
                Vec3f::RIGHT * rotation * time.delta_time() * movement_speed;
        }
        if input.keys.is_down(Noncharacter::Space) {
            transform.position +=
                Vec3f::UP * rotation * time.delta_time() * movement_speed;
        }
        if input.keys.is_down(Noncharacter::Shift) {
            transform.position +=
                Vec3f::DOWN * rotation * time.delta_time() * movement_speed;
        }
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
    app.run();
}
