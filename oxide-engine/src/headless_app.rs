use crate::world::World;

#[allow(unused)]
pub struct App {
    world: World,
}

impl App {
    pub fn new() -> App {
        App {
            world: World::new(),
        }
    }

    pub fn run(&mut self) {
        println!("Running Headless!");

        loop {
            println!("Running...");
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}
