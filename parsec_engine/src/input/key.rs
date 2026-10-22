//! Key related types.

pub type Noncharacter = winit::keyboard::NamedKey;
pub type KeyState = winit::event::ElementState;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum StorageKeyCode {
    Noncharacter(Noncharacter),
    Character(winit::keyboard::SmolStr),
}

pub trait KeyCode {
    fn into_storage_key_code(&self) -> StorageKeyCode;
}

impl KeyCode for Noncharacter {
    fn into_storage_key_code(&self) -> StorageKeyCode {
        StorageKeyCode::Noncharacter(*self)
    }
}

impl KeyCode for &'static str {
    fn into_storage_key_code(&self) -> StorageKeyCode {
        StorageKeyCode::Character((*self).into())
    }
}
