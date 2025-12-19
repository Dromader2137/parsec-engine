//! Module responsible for handling windows.

use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::{math::vec::Vec2f, utils::id_counter::IdCounter};

#[derive(Debug)]
pub struct Window {
    id: u32,
    window: winit::window::Window,
    cursor_mode: winit::window::CursorGrabMode,
    cursor_visibility: bool
}

#[derive(Debug, thiserror::Error)]
pub enum WindowError {
    #[error("Failed to create window: {0}")]
    CreationError(winit::error::OsError),
    #[error("Failed to set cursor position: {0}")]
    SetCursorPositionError(winit::error::ExternalError),
    #[error("Failed to set cursor grab mode: {0}")]
    SetCursorModeError(winit::error::ExternalError),
    #[error("Failed to set cursor visibility: {0}")]
    SetCursorVisibilityError(winit::error::ExternalError),
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
            cursor_mode: winit::window::CursorGrabMode::None,
            cursor_visibility: true
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

    pub fn toggle_cursor_visibility(&mut self) {
        let new_visibility = !self.cursor_visibility;
        
        self.window
            .set_cursor_visible(new_visibility);

        self.cursor_visibility = new_visibility;
    }

    pub fn toggle_cursor_lock(
        &mut self,
    ) -> Result<(), WindowError> {
        let new_mode = match self.cursor_mode {
            winit::window::CursorGrabMode::None => winit::window::CursorGrabMode::Locked,
            _ => winit::window::CursorGrabMode::None
        };

        self.window
            .set_cursor_grab(new_mode)
            .map_err(|err| WindowError::SetCursorModeError(err))?;

        self.cursor_mode = new_mode;
        Ok(())
    }

    pub fn set_cursor_position(
        &self,
        position: Vec2f,
    ) -> Result<(), WindowError> {
        self.window
            .set_cursor_position(winit::dpi::LogicalPosition::new(
                position.x, position.y,
            ))
            .map_err(|r| WindowError::SetCursorPositionError(r))
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

    pub fn focused(&self) -> bool { self.window.has_focus() }

    pub fn minimized(&self) -> bool { self.width() <= 0 || self.height() <= 0 }

    pub fn id(&self) -> u32 { self.id }
}
