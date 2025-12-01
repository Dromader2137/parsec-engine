//! Key related types.

pub type Noncharacter = winit::keyboard::NamedKey;
type Character = winit::keyboard::SmolStr;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum StorageKeyCode {
    Noncharacter(Noncharacter),
    Character(Character)
}
impl From<KeyCode> for StorageKeyCode {
    fn from(value: KeyCode) -> Self {
        match value {
            KeyCode::Noncharacter(special) => StorageKeyCode::Noncharacter(special),
            KeyCode::Character(char) => StorageKeyCode::Character(char.into())
        }
    }
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum KeyCode {
    Noncharacter(Noncharacter),
    Character(&'static str)
}
pub type KeyState = winit::event::ElementState;
