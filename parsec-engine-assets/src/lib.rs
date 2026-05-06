use std::{collections::HashMap, fs::File, io::BufReader};

pub struct AssetHandle;
pub struct AssetLibrary;

pub trait Asset {
    type Cooked;

    fn type_name() -> &'static str;
    fn name(&self) -> &'static str;
    fn cook(reader: BufReader<File>) -> Self::Cooked; 
    fn load();
    fn unload();
    fn reload();
}

pub struct AssetHandler {
    load_fn: Box<fn()>,
    unload_fn: Box<fn()>,
    reload_fn: Box<fn()>
}

pub struct AssetTypes {
    handlers: HashMap<&'static str, Box<>>
}
