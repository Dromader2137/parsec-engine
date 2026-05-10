use parsec_engine::assets::assets::mesh::Mesh;
use parsec_engine::cli::{Cooker, run_cli};

fn main() {
    let mut cooker = Cooker::new();
    cooker.register::<Mesh>();
    run_cli(cooker);
}
