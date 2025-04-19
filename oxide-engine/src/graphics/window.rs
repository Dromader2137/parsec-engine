use std::sync::Arc;

use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

#[derive(Debug)]
pub struct WindowWrapper {
    window: Arc<winit::window::Window>,
}

#[derive(Debug, Clone)]
pub enum WindowError {
    CreationError(String),
}

impl WindowWrapper {
    pub fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> Result<WindowWrapper, WindowError> {
        let mut attributes = winit::window::Window::default_attributes();
        attributes.transparent = false;
        attributes.visible = true;
        let window = match event_loop.create_window(attributes) {
            Ok(val) => val,
            Err(err) => {
                return Err(WindowError::CreationError(format!("{:?}", err)));
            }
        };

        Ok(WindowWrapper {
            window: Arc::new(window),
        })
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn get_size(&self) -> (u32, u32) {
        let physical_size = self.window.inner_size();
        (physical_size.width, physical_size.width)
    }

    pub fn get_physical_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.window.inner_size()
    }

    pub fn get_display_handle(&self) -> Result<winit::raw_window_handle::DisplayHandle<'_>, winit::raw_window_handle::HandleError> {
        self.window.display_handle()
    }
    
    pub fn get_window_handle(&self) -> Result<winit::raw_window_handle::WindowHandle<'_>, winit::raw_window_handle::HandleError> {
        self.window.window_handle()
    }
}
