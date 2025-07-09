use oxide_engine::{app::App, ecs::system::{System, SystemInput, SystemType}};

fn main() {
    let mut app = App::new("Oxide test".to_string());
    app.systems.add_system(System::new(SystemType::Start, |SystemInput { world, .. }| { 
        world.spawn((0_u32, "asdf")).unwrap();
    }));
    app.systems.add_system(System::new(SystemType::Start, |SystemInput { world, .. }| { 
        let x = world.query::<(u32, &str)>().unwrap();
        println!("{:#?}", world);
        let y = x.collect::<Vec<_>>();
        println!("{:#?}", world);
        let z = world.query_mut::<(u32, &str)>().unwrap();
        println!("{:#?}", world);
        let u = z.collect::<Vec<_>>();
        println!("{:#?}", world);
        println!("{:?} {:?}", y, u);
    }));
    app.run();
}
