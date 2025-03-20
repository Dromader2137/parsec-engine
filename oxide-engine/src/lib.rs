use app::App;

pub mod app;
pub mod error;
pub mod graphics;
pub mod input;
pub mod world;
pub use oxide_engine_macros;

pub fn run() {
    let mut app = App::default_settings();
    app.run();
    app.wait();
}
