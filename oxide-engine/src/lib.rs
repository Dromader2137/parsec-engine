use app::App;

pub mod app;
pub mod assets;
pub mod error;
pub mod graphics;
pub mod input;
pub mod math;
pub mod world;

pub use oxide_engine_macros;

pub fn run() {
    let mut app = App::new();
    app.run();
}
