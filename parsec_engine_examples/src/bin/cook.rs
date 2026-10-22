use parsec_engine::error::ParsecError;

fn main() -> Result<(), ParsecError> {
    let app = parsec_engine_examples::prepare_app()?;
    app.assets.cook_all()
}
