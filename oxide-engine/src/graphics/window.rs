use std::sync::Arc;

use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use super::GraphicsError;

#[derive(Debug)]
pub struct WindowWrapper {
    window: Arc<winit::window::Window>,
}

#[derive(Debug)]
pub enum WindowError {
    CreationError(winit::error::OsError),
}

impl From<WindowError> for GraphicsError {
    fn from(value: WindowError) -> Self {
        GraphicsError::WindowError(value)
    }
}

impl WindowWrapper {
    pub fn new(event_loop: &winit::event_loop::ActiveEventLoop, name: &str) -> Result<Arc<WindowWrapper>, WindowError> {
        let attributes = winit::window::Window::default_attributes()
            .with_transparent(false)
            .with_visible(true)
            .with_title(name);

        let window = match event_loop.create_window(attributes) {
            Ok(val) => val,
            Err(err) => {
                return Err(WindowError::CreationError(err));
            }
        };

        Ok(Arc::new(WindowWrapper {
            window: Arc::new(window),
        }))
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn get_size(&self) -> (u32, u32) {
        let physical_size = self.window.inner_size();
        (physical_size.width, physical_size.height)
    }

    pub fn get_width(&self) -> u32 {
        self.get_size().0
    }

    pub fn get_height(&self) -> u32 {
        self.get_size().1
    }

    pub fn get_physical_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.window.inner_size()
    }

    pub fn get_display_handle(
        &self,
    ) -> Result<winit::raw_window_handle::DisplayHandle<'_>, winit::raw_window_handle::HandleError> {
        self.window.display_handle()
    }

    pub fn get_window_handle(
        &self,
    ) -> Result<winit::raw_window_handle::WindowHandle<'_>, winit::raw_window_handle::HandleError> {
        self.window.window_handle()
    }

    pub fn minimized(&self) -> bool {
        self.get_width() <= 0 || self.get_height() <= 0
    }
}
