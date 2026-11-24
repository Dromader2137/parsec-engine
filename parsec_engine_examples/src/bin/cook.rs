use parsec_engine::error::SystemError;

fn main() -> Result<(), SystemError> {
    let app = parsec_engine_examples::prepare_app()?;
    app.assets.cook_all()?;
    Ok(())
}
