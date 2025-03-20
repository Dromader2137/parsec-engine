#[derive(Debug)]
pub enum EngineError {
    Init(String),
    Graphics(String),
}
