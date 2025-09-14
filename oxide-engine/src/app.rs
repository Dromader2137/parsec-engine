use std::ptr::NonNull;

use ash::vk::DescriptorType;

use crate::{
    assets::library::AssetLibrary,
    ecs::{
        system::{System, SystemInput, SystemType, Systems},
        world::World,
    },
    graphics::{
        renderer::DefaultVertex, vulkan::{
            descriptor_set::{DescriptorSetBinding, DescriptorStage},
            shader::{read_shader_code, ShaderType},
        }, Graphics
    },
    input::{Input, InputEvent},
    math::{mat::Matrix4f, vec::Vec3f},
    resources::ResourceCollection,
};

#[allow(unused)]
pub struct App {
    world: World,
    pub systems: Systems,
    assets: AssetLibrary,
    resources: ResourceCollection,
}

impl App {
    pub fn new() -> App {
        App {
            world: World::new(),
            systems: Systems::new(),
            assets: AssetLibrary::new(),
            resources: ResourceCollection::new(),
        }
    }

    pub fn run(&mut self) {
        self.systems
            .push(System::new(SystemType::Start, |SystemInput { resources, .. }| {
                resources.add(Input::new()).unwrap();
            }));
        
        self.systems
            .push(System::new(SystemType::Start, |SystemInput { resources, .. }| {
                resources.add(Graphics::new()).unwrap();
            }));

        self.systems
            .push(System::new(SystemType::LateStart, |SystemInput { resources, .. }| {
                let mut graphics = resources.get_mut::<Graphics>().unwrap();
                let event_loop = resources.get::<ActiveEventLoopStore>().unwrap();
            
                let event_loop_raw = event_loop.get_event_loop();
                graphics.init(event_loop_raw, "Oxide Engine test").unwrap();
            }
            ));

        self.systems
            .push(System::new(SystemType::LateStart, |SystemInput { resources, .. }| {
                let mut graphics = resources.get_mut::<Graphics>().unwrap();

                graphics
                    .add_shader(
                        "vertex",
                        &read_shader_code("shaders/simple.spv").unwrap(),
                        ShaderType::Vertex,
                    )
                    .unwrap();
                graphics
                    .add_shader(
                        "fragment",
                        &read_shader_code("shaders/flat.spv").unwrap(),
                        ShaderType::Fragment,
                    )
                    .unwrap();
                graphics
                    .add_material_base(
                        "simple_base",
                        "vertex",
                        "fragment",
                        vec![vec![DescriptorSetBinding::new(
                            0,
                            DescriptorType::UNIFORM_BUFFER,
                            DescriptorStage::VERTEX,
                        )]],
                    )
                    .unwrap();

                let aspect_ratio = graphics.get_screen_aspect_ratio().unwrap();

                graphics
                    .add_buffer(
                        "mvp",
                        vec![
                            Matrix4f::perspective(40.0_f32.to_radians(), aspect_ratio, 5.0, 100.0)
                                * Matrix4f::look_at(Vec3f::ZERO, Vec3f::FORWARD, Vec3f::UP)
                                * Matrix4f::translation(Vec3f::FORWARD * 30.0),
                        ],
                    )
                    .unwrap();

                graphics
                    .add_material("simple", "simple_base", vec![vec!["mvp"]])
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

                graphics.add_mesh("triangle", vertices, indices).unwrap();
            }));

        self.systems
            .push(System::new(SystemType::Update, |SystemInput { resources, .. }| {
                let graphics = resources.get::<Graphics>().unwrap();

                graphics
                    .renderer()
                    .unwrap()
                    .update_buffer(
                        "mvp",
                        vec![
                            Matrix4f::perspective(
                                40.0_f32.to_radians(),
                                graphics.get_screen_aspect_ratio().unwrap(),
                                5.0,
                                100.0,
                            ) * Matrix4f::look_at(Vec3f::ZERO, Vec3f::FORWARD, Vec3f::UP)
                                * Matrix4f::translation(Vec3f::FORWARD * 30.0),
                        ],
                    )
                    .unwrap();
            }));

        self.systems
            .push(System::new(SystemType::Render, |SystemInput { resources, .. }| {
                let mut graphics = resources.get_mut::<Graphics>().unwrap();
                graphics.render().unwrap();
            }));

        self.systems
            .push(System::new(SystemType::Render, |SystemInput { resources, .. }| {
                let mut input = resources.get_mut::<Input>().unwrap();
                input.keys.clear();
            }));
        
        self.systems
            .push(System::new(SystemType::WindowCursorLeft, |SystemInput { resources, .. }| {
                let mut input = resources.get_mut::<Input>().unwrap();
                input.keys.clear_all();
            }));
        
        self.systems
            .push(System::new(SystemType::KeyboardInput, |SystemInput { resources, .. }| {
                let mut input = resources.get_mut::<Input>().unwrap();
                let event = resources.get::<InputEvent>().unwrap();
                input.keys.process_input_event(*event);
            }));

        self.systems.push(System::new(
            SystemType::Update,
            |SystemInput { resources, .. }| {
                let mut graphics = resources.get_mut::<Graphics>().unwrap();
                graphics.request_redraw().unwrap();
            },
        ));

        self.systems.execute_type(
            SystemType::Start,
            SystemInput {
                world: &mut self.world,
                assets: &mut self.assets,
                resources: &mut self.resources,
            },
        );

        let event_loop = winit::event_loop::EventLoop::new().expect("Valid event loop");
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        event_loop.run_app(self).expect("Correctly working event loop");
    }
}

pub struct ActiveEventLoopStore {
    event_loop: NonNull<winit::event_loop::ActiveEventLoop>
}

impl ActiveEventLoopStore {
    fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> ActiveEventLoopStore {
        ActiveEventLoopStore { event_loop: NonNull::from_ref(event_loop) }
    }

    fn get_event_loop(&self) -> &winit::event_loop::ActiveEventLoop {
        unsafe { self.event_loop.as_ref() }
    }
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.resources.add(ActiveEventLoopStore::new(event_loop)).unwrap();

        self.systems.execute_type(
            SystemType::LateStart,
            SystemInput {
                world: &mut self.world,
                assets: &mut self.assets,
                resources: &mut self.resources,
            },
        );

        self.resources.remove::<ActiveEventLoopStore>().unwrap();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::KeyboardInput {
                event: winit::event::KeyEvent {
                    physical_key, state, ..
                },
                ..
            } => {
                if let winit::keyboard::PhysicalKey::Code(key_code) = physical_key {
                    self.resources.add(InputEvent::new(key_code.into(), state.into())).unwrap();
                }

                self.systems.execute_type(
                    SystemType::KeyboardInput,
                    SystemInput {
                        world: &mut self.world,
                        assets: &mut self.assets,
                        resources: &mut self.resources,
                    },
                );

                let _ = self.resources.remove::<InputEvent>();
            }
            winit::event::WindowEvent::CursorLeft { device_id: _ } => {
                self.systems.execute_type(
                    SystemType::WindowCursorLeft,
                    SystemInput {
                        world: &mut self.world,
                        assets: &mut self.assets,
                        resources: &mut self.resources,
                    },
                );
            }
            winit::event::WindowEvent::Resized(_) => {
                self.systems.execute_type(
                    SystemType::WindowResized,
                    SystemInput {
                        world: &mut self.world,
                        assets: &mut self.assets,
                        resources: &mut self.resources,
                    },
                );
            }
            winit::event::WindowEvent::CloseRequested => {
                self.systems.execute_type(
                    SystemType::End,
                    SystemInput {
                        world: &mut self.world,
                        assets: &mut self.assets,
                        resources: &mut self.resources,
                    },
                );
                event_loop.exit();
            }
            winit::event::WindowEvent::RedrawRequested => {
                self.systems.execute_type(
                    SystemType::Render,
                    SystemInput {
                        world: &mut self.world,
                        assets: &mut self.assets,
                        resources: &mut self.resources,
                    },
                );
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.systems.execute_type(
            SystemType::Update,
            SystemInput {
                world: &mut self.world,
                assets: &mut self.assets,
                resources: &mut self.resources,
            },
        );
    }
}
