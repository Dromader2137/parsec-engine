use oxide_engine::{app::App, ecs::system::{System, SystemInput, SystemType}};

fn main() {
    let mut app = App::new("Oxide test".to_string());
    app.systems.add_system(System::new(SystemType::Start, |SystemInput { world, .. }| { 
        world.spawn((0_u32, "asdf")).unwrap();
        world.spawn((0_u32, "asdf")).unwrap();
        world.spawn((0_u32, "asdf")).unwrap();
        world.spawn((0_u32, "asdf")).unwrap();
    }));
    app.systems.add_system(System::new(SystemType::Start, |SystemInput { world, .. }| { 
        let z = world.query::<(u32, &str)>().unwrap();
        println!("{:#?}", world);
        let y = world.query_mut::<(u32, &str)>().unwrap();
        println!("{:#?}", world);
        y.for_each(|x| { println!("{:?}", x); });
        z.for_each(|x| { println!("{:?}", x); });
    }));
    app.run();
}
