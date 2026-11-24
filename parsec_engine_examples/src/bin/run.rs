use parsec_engine::error::SystemError;

fn main() -> Result<(), SystemError> {
    let mut app = parsec_engine_examples::prepare_app()?;
    app.execute()
}
