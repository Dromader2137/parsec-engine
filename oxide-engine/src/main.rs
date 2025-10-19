use oxide_engine::{app::App, ecs::system::{System, SystemInput, SystemTrigger}, graphics::{renderer::{draw_queue::{Draw, MeshAndMaterial}, DefaultVertex, VulkanRenderer}, vulkan::{descriptor_set::{DescriptorSetBinding, DescriptorStage, DescriptorType}, shader::{read_shader_code, ShaderType}}, GraphicsBundle}, math::{mat::Matrix4f, vec::Vec3f}};

fn main() {
    let mut app = App::new();
    app.systems.add_bundle(GraphicsBundle::default());
    app.systems.add(System::new(
        SystemTrigger::LateStart,
        |SystemInput { resources, .. }| {
            let mut renderer = resources.get_mut::<VulkanRenderer>().unwrap();

            let vertex = renderer
                .create_shader(
                    &read_shader_code("shaders/simple.spv").unwrap(),
                    ShaderType::Vertex,
                )
                .unwrap();

            let fragment = renderer
                .create_shader(
                    &read_shader_code("shaders/flat.spv").unwrap(),
                    ShaderType::Fragment,
                )
                .unwrap();

            let material_base = renderer
                .create_material_base(
                    vertex,
                    fragment,
                    vec![vec![DescriptorSetBinding::new(
                        0,
                        DescriptorType::UNIFORM_BUFFER,
                        DescriptorStage::VERTEX,
                    )]],
                )
                .unwrap();

            let aspect_ratio = renderer.get_aspect_ratio();

            let mvp_buffer = renderer
                .create_buffer(
                    vec![
                        Matrix4f::perspective(40.0_f32.to_radians(), aspect_ratio, 5.0, 100.0)
                            * Matrix4f::look_at(Vec3f::ZERO, Vec3f::FORWARD, Vec3f::UP)
                            * Matrix4f::translation(Vec3f::FORWARD * 30.0),
                    ],
                )
                .unwrap();

            let _material = renderer
                .create_material(material_base, vec![vec![mvp_buffer]])
                .unwrap();

            let pos = vec![
                Vec3f::new(0.0, 0.0, 0.0),
                Vec3f::new(1.0, 1.0, 1.0),
                Vec3f::new(0.0, 1.0, 1.0),
                Vec3f::new(0.0, 0.0, 0.0),
            ];

            let nor = vec![
                Vec3f::new(0.0, 0.0, 0.0),
                Vec3f::new(-0.966742, -0.255752, 0.0),
                Vec3f::new(-0.966824, 0.255443, 0.0),
                Vec3f::new(-0.092052, 0.995754, 0.0),
            ];

            let indices = vec![1, 2, 3, 0, 2, 3];

            let vertices = pos
                .iter()
                .zip(nor.iter())
                .map(|x| DefaultVertex::new(*x.0, *x.1))
                .collect();

            let _mesh = renderer.create_mesh(vertices, indices).unwrap();
        },
    ));
    app.systems.add(System::new(
        SystemTrigger::Update,
        |SystemInput { resources, .. }| {
            let mut renderer = resources.get_mut::<VulkanRenderer>().unwrap();

            renderer.queue_draw(Draw::MeshAndMaterial(MeshAndMaterial {mesh_id: 0, material_id: 0}));

            renderer
                .update_buffer(
                    0,
                    vec![
                        Matrix4f::perspective(
                            40.0_f32.to_radians(),
                            renderer.get_aspect_ratio(),
                            5.0,
                            100.0,
                        ) * Matrix4f::look_at(Vec3f::ZERO, Vec3f::FORWARD, Vec3f::UP)
                            * Matrix4f::translation(Vec3f::FORWARD * 30.0),
                    ],
                )
                .unwrap();
        },
    ));
    app.run();
}
