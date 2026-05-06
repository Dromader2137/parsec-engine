use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::File,
    io::{Read, Write},
    marker::PhantomData,
    path::Path,
};

pub mod assets;

pub struct AssetHandle<T: Asset> {
    name: &'static str,
    _marker: PhantomData<T>,
}

pub struct AssetLibrary {}

pub trait AssetSource {
    fn parse(bytes: &[u8]) -> Self;
}

pub trait Asset {
    type Source: AssetSource;
    type Cooked: serde::Serialize + 'static;

    const ASSET_TYPE: &'static str;
    const EXTENSIONS: &'static [&'static str];

    fn cook(source: Self::Source) -> Self::Cooked;
    fn load(cooked: Self::Cooked) -> Self;
}

pub struct CookerAssetRegistation {
    extensions: &'static [&'static str],
    cook_fn: Box<fn(&[u8]) -> Vec<u8>>,
}

pub struct Cooker {
    handlers: HashMap<&'static str, CookerAssetRegistation>,
}

fn cook_type_erased<T: Asset>(bytes: &[u8]) -> Vec<u8> {
    let source = T::Source::parse(bytes);
    let out = T::cook(source);
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
        let (_, handler) = self.handlers.iter().find(|(_, v)| {
            v.extensions.contains(&ext)
        }).unwrap();
        let mut input_file = File::open(input).unwrap();
        let mut input_bytes = Vec::new();
        input_file.read_to_end(&mut input_bytes).unwrap();
        let out_bytes = (handler.cook_fn)(input_bytes.as_slice());
        let mut out_file = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(output)
            .unwrap();
        out_file.write_all(&out_bytes).unwrap();
    }
}
