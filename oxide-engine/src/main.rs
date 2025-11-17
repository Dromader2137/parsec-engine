use oxide_engine::{
    app::App,
    components::{camera::Camera, transform::Transform},
    ecs::system::{System, SystemInput, SystemTrigger},
    graphics::{
        GraphicsBundle,
        renderer::{
            DefaultVertex,
            create_mesh, create_shader,
            draw_queue::{Draw, MeshAndMaterial},
            material_data::{MaterialDescriptorSets, create_material, create_material_base},
            queue_draw,
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
        |SystemInput {
             resources, world, ..
         }| {
            let scale = 1.0;

            world
                .spawn((
                    Camera::new(resources, 40.0_f32.to_radians(), 1.0, 100.0),
                    Transform::new(resources, Vec3f::ZERO, Vec3f::ZERO, Vec3f::ZERO),
                ))
                .unwrap();
            
            world
                .spawn(
                    Transform::new(resources, Vec3f::FORWARD * 5.0 + Vec3f::new(-0.5, -0.5, 0.0) * scale, Vec3f::ZERO, Vec3f::ZERO),
                )
                .unwrap();
            
            world
                .spawn(
                    Transform::new(resources, Vec3f::FORWARD * 6.0, Vec3f::ZERO, Vec3f::ZERO),
                )
                .unwrap();

            let vertex = create_shader(
                resources,
                &read_shader_code("shaders/simple.spv").unwrap(),
                ShaderType::Vertex,
            )
            .unwrap();

            let fragment = create_shader(
                resources,
                &read_shader_code("shaders/flat.spv").unwrap(),
                ShaderType::Fragment,
            )
            .unwrap();

            let material_base = create_material_base(resources, vertex, fragment, vec![
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

            let _material = create_material(resources, material_base, vec![
                MaterialDescriptorSets::ModelMatrixSet,
                MaterialDescriptorSets::ViewMatrixSet,
                MaterialDescriptorSets::ProjectionMatrixSet,
            ])
            .unwrap();

            let pos = vec![
                Vec3f::new(0.0, 0.0, 0.0) * scale,
                Vec3f::new(1.0, 1.0, 0.0) * scale,
                Vec3f::new(0.0, 1.0, 0.0) * scale,
                Vec3f::new(1.0, 0.0, 0.0) * scale,
            ];

            let nor = vec![
                Vec3f::new(0.0, 0.0, 0.0),
                Vec3f::new(1.0, 1.0, 0.0),
                Vec3f::new(0.0, 1.0, 0.0),
                Vec3f::new(1.0, 0.0, 0.0),
            ];

            let indices = vec![0, 2, 1, 0, 1, 3];

            let vertices = pos
                .iter()
                .zip(nor.iter())
                .map(|x| DefaultVertex::new(*x.0, *x.1))
                .collect();

            let _mesh = create_mesh(resources, vertices, indices).unwrap();
        },
    ));
    app.systems.add(System::new(
        SystemTrigger::Update,
        |SystemInput { resources, .. }| {
            queue_draw(
                resources,
                Draw::MeshAndMaterial(MeshAndMaterial {
                    mesh_id: 0,
                    material_id: 0,
                    transform_id: 2,
                    camera_id: 0,
                }),
            );
            queue_draw(
                resources,
                Draw::MeshAndMaterial(MeshAndMaterial {
                    mesh_id: 0,
                    material_id: 0,
                    transform_id: 1,
                    camera_id: 0,
                }),
            );
        },
    ));
    app.run();
}
