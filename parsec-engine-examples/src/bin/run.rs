use parsec_engine::error::ParsecError;

fn main() -> Result<(), ParsecError> {
    let mut app = parsec_engine_examples::prepare_app()?;
    app.execute()
}
