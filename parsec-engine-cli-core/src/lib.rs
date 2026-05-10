use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self, File},
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    str::FromStr,
    time::SystemTime,
};

use clap::Parser;
use parsec_engine_assets::{Asset, assets::mesh::Mesh};
use parsec_engine_error::{OptionNoneErr, ParsecError};

/// parsec-engine-cli add <name> <path> // adds an asset
/// parsec-engine-cli remove <name> // removes an asset
/// parsec-engine-cli cook // cooks all assets
///
/// Example project structure:
/// src/
/// Cargo.toml
/// assets/
///   asset1.asset
///   asset2.asset
/// assets.json
///
/// assets.json
///

#[derive(Debug, clap::Parser)]
#[command()]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    Add { name: String, path: PathBuf },
    Remove { name: String },
    Cook,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct AssetDescription {
    name: String,
    path: PathBuf,
    last_cooked: Option<SystemTime>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Manifest {
    verison: u8,
    assets: HashMap<String, AssetDescription>,
}

impl Manifest {
    pub fn new() -> Self {
        Self {
            verison: 0,
            assets: HashMap::new(),
        }
    }
}

fn get_manifest() -> Manifest {
    let try_file = File::options().read(true).open("./assets.json");
    if matches!(
        try_file.as_ref().map_err(|err| err.kind()),
        Err(std::io::ErrorKind::NotFound)
    ) {
        Manifest::new()
    } else {
        let file = try_file.expect("Failed to open assets.json");
        let reader = BufReader::new(file);
        let manifest = serde_json::from_reader(reader)
            .expect("Failed to parse assets.json");
        manifest
    }
}

#[derive(Debug)]
pub enum ManifestWriteError {
    FailedToCreateFile(std::io::Error),
    FailedToWriteTemp(serde_json::Error),
    FailedToSwapFiles(std::io::Error),
}

fn write_manifest(manifest: &Manifest) -> Result<(), ManifestWriteError> {
    let write_file = File::options()
        .write(true)
        .truncate(true)
        .create(true)
        .open("./assets.json.tmp")
        .map_err(|err| ManifestWriteError::FailedToCreateFile(err))?;
    let writer = BufWriter::new(write_file);
    serde_json::to_writer_pretty(writer, manifest)
        .map_err(|err| ManifestWriteError::FailedToWriteTemp(err))?;
    let rename_op = std::fs::rename("./assets.json.tmp", "./assets.json")
        .map_err(|err| ManifestWriteError::FailedToSwapFiles(err));
    if rename_op.is_err() {
        std::fs::remove_file("./assets.json.tmp")
            .expect("Failed to delete assets.json.tmp");
    }
    Ok(())
}

fn get_cook_dir() -> PathBuf {
    fs::create_dir_all("./assets/").unwrap();
    PathBuf::from_str("./assets/").unwrap()
}

fn cook(
    name: &str,
    manifest: &Manifest,
    cooker: &Cooker,
) -> Result<(), ParsecError> {
    let in_path = manifest.assets.iter().find(|a| a.0 == name).none_err()?;
    let out_path = get_cook_dir().join(name).with_extension("asset");
    cooker.cook(&in_path.1.path, &out_path);
    Ok(())
}

pub struct CookerAssetRegistation {
    extensions: &'static [&'static str],
    cook_fn: Box<fn(&[u8], &str) -> Vec<u8>>,
}

pub struct Cooker {
    handlers: HashMap<&'static str, CookerAssetRegistation>,
}

fn cook_type_erased<T: Asset>(data: &[u8], extension: &str) -> Vec<u8> {
    let out = T::cook(data, extension);
    let out_bytes = postcard::to_stdvec(&out).unwrap();
    out_bytes
}

impl Cooker {
    pub fn new() -> Cooker {
        Cooker {
            handlers: HashMap::new(),
        }
    }

    pub fn register<T: Asset>(&mut self) {
        let registation = CookerAssetRegistation {
            extensions: T::EXTENSIONS,
            cook_fn: Box::new(cook_type_erased::<T>),
        };
        self.handlers.insert(T::ASSET_TYPE, registation);
    }

    pub fn cook(&self, input: &Path, output: &Path) {
        let ext = input
            .extension()
            .unwrap_or(&OsStr::new(""))
            .to_str()
            .unwrap();
        let (_, handler) = self
            .handlers
            .iter()
            .find(|(_, v)| v.extensions.contains(&ext))
            .unwrap();
        let bytes = std::fs::read(input).unwrap();
        let out_bytes = (handler.cook_fn)(
            &bytes,
            input.extension().unwrap().to_str().unwrap(),
        );
        let mut out_file = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(output)
            .unwrap();
        out_file.write_all(&out_bytes).unwrap();
    }
}

pub fn run_cli(mut cooker: Cooker) {
    let args = Args::parse();

    let mut manifest = get_manifest();
    cooker.register::<Mesh>();

    match args.command {
        Commands::Add { name, path } => {
            if manifest.assets.contains_key(&name) {
                println!("Asset with this name already existst");
                return;
            }
            manifest.assets.insert(name.clone(), AssetDescription {
                name: name.clone(),
                path,
                last_cooked: None,
            });
            write_manifest(&manifest).unwrap();
            println!("Added asset {}", name);
        },
        Commands::Remove { name } => {
            if !manifest.assets.contains_key(&name) {
                println!("Asset not found");
                return;
            }
            manifest.assets.remove(&name);
            println!("Removed asset {}", name);
        },
        Commands::Cook => {
            println!("Cooking...");
            for name in manifest.assets.keys() {
                cook(name, &manifest, &cooker).unwrap();
            }
        },
    }
}
