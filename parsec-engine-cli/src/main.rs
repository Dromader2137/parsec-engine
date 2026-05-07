use parsec_engine_assets::assets::mesh::Mesh;
use parsec_engine_cli_core::{Cooker, run_cli};

fn main() {
    let mut cooker = Cooker::new();
    cooker.register::<Mesh>();
    run_cli(cooker);
}
