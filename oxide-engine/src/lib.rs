use app::App;

#[cfg(not(feature = "headless"))]
pub mod app;
#[cfg(feature = "headless")]
pub mod headless_app;
#[cfg(feature = "headless")]
use headless_app as app;
pub mod graphics;
pub mod input;
pub mod world;
pub use oxide_engine_macros;

pub fn run() {
    let mut app = App::new();
    app.run();
}
