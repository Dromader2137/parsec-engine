//! Module responsible for handling windows.

use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::{math::vec::Vec2f, utils::id_counter::IdCounter};

#[derive(Debug)]
pub struct Window {
    id: u32,
    window: winit::window::Window,
}

#[derive(Debug)]
pub enum WindowError {
    CreationError(winit::error::OsError),
    SetCursorPositionError(winit::error::ExternalError),
}

static ID_COUNTER: once_cell::sync::Lazy<IdCounter> =
    once_cell::sync::Lazy::new(|| IdCounter::new(0));
impl Window {
    pub fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
        name: &str,
    ) -> Result<Window, WindowError> {
        let attributes = winit::window::Window::default_attributes()
            .with_transparent(false)
            .with_visible(true)
            .with_title(name);

        let window = match event_loop.create_window(attributes) {
            Ok(val) => val,
            Err(err) => {
                return Err(WindowError::CreationError(err));
            },
        };

        Ok(Window {
            id: ID_COUNTER.next(),
            window,
        })
    }

    pub fn request_redraw(&self) { self.window.request_redraw(); }

    pub fn size(&self) -> (u32, u32) {
        let physical_size = self.window.inner_size();
        (physical_size.width, physical_size.height)
    }

    pub fn width(&self) -> u32 { self.size().0 }

    pub fn height(&self) -> u32 { self.size().1 }

    pub fn aspect_ratio(&self) -> f32 {
        if self.height() == 0 {
            return 1.0;
        }
        self.width() as f32 / self.height() as f32
    }

    pub fn physical_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.window.inner_size()
    }

    pub fn set_cursor_position(
        &self,
        position: Vec2f,
    ) -> Result<(), WindowError> {
        self.window
            .set_cursor_position(winit::dpi::LogicalPosition::new(
                position.x, position.y,
            ))
            .map_err(|r| WindowError::SetCursorPositionError(r))?;
        Ok(())
    }

    pub fn raw_display_handle(
        &self,
    ) -> Result<
        winit::raw_window_handle::DisplayHandle<'_>,
        winit::raw_window_handle::HandleError,
    > {
        self.window.display_handle()
    }

    pub fn raw_window_handle(
        &self,
    ) -> Result<
        winit::raw_window_handle::WindowHandle<'_>,
        winit::raw_window_handle::HandleError,
    > {
        self.window.window_handle()
    }

    pub fn minimized(&self) -> bool { self.width() <= 0 || self.height() <= 0 }

    pub fn id(&self) -> u32 { self.id }
}
