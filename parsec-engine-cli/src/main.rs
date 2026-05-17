use parsec_engine::{
    assets::core::mesh::Mesh,
    cli::{Cooker, run_cli},
};

fn main() {
    let mut cooker = Cooker::new();
    cooker.register::<Mesh>();
    run_cli(cooker);
}
