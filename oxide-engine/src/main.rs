use oxide_engine::{
    app::App,
    ecs::system::{System, SystemInput, SystemTrigger},
    graphics::{
        GraphicsBundle,
        renderer::{
            DefaultVertex,
            assets::mesh::Mesh,
            components::{camera::Camera, mesh_renderer::MeshRenderer, transform::Transform},
            create_shader,
            material_data::{MaterialDescriptorSets, create_material, create_material_base},
        },
        vulkan::{
            descriptor_set::{DescriptorSetBinding, DescriptorStage, DescriptorType},
            shader::{ShaderType, read_shader_code},
        },
    },
    math::vec::Vec3f,
};

fn main() {
    let mut app = App::new();
    app.systems.add_bundle(GraphicsBundle::default());
    app.systems.add(System::new(
        SystemTrigger::LateStart,
        |SystemInput { world, assets }| {
            let scale = 1.0;

            let vertex = create_shader(
                &read_shader_code("shaders/simple.spv").unwrap(),
                ShaderType::Vertex,
            )
            .unwrap();

            let fragment = create_shader(
                &read_shader_code("shaders/flat.spv").unwrap(),
                ShaderType::Fragment,
            )
            .unwrap();

            let material_base = create_material_base(vertex, fragment, vec![
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

            let material = create_material(material_base, vec![
                MaterialDescriptorSets::ModelMatrixSet,
                MaterialDescriptorSets::ViewMatrixSet,
                MaterialDescriptorSets::ProjectionMatrixSet,
            ])
            .unwrap();

            let vertices = vec![
                DefaultVertex::new(Vec3f::new(0.0, 0.0, 0.0) * scale, Vec3f::new(0.0, 0.0, 0.0)),
                DefaultVertex::new(Vec3f::new(1.0, 1.0, 0.0) * scale, Vec3f::new(1.0, 1.0, 0.0)),
                DefaultVertex::new(Vec3f::new(0.0, 1.0, 0.0) * scale, Vec3f::new(0.0, 1.0, 0.0)),
                DefaultVertex::new(Vec3f::new(1.0, 0.0, 0.0) * scale, Vec3f::new(1.0, 0.0, 0.0)),
            ];

            let indices = vec![0, 2, 1, 0, 1, 3];

            let mesh = assets.add(Mesh::new(vertices, indices), world).unwrap();

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
                    MeshRenderer::new(mesh, material),
                ))
                .unwrap();

            world
                .spawn((
                    Transform::new(
                        Vec3f::FORWARD * 5.0 + Vec3f::new(-0.5, -0.5, 0.0) * scale,
                        Vec3f::ZERO,
                        Vec3f::ZERO,
                    ),
                    MeshRenderer::new(mesh, material),
                ))
                .unwrap();
        },
    ));

    app.run();
}
