use parsec_engine_math::vec::{Vec2f, Vec3f};
use thiserror::Error;

use crate::assets::mesh::CookedMesh;

#[derive(Error, Debug)]
pub enum LoadOBJError {
    #[error("Failed to load OBJ, because of an IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Failed to load OBJ, beacuse of a parsing error: {0}")]
    FloatParseError(#[from] std::num::ParseFloatError),
    #[error("Failed to load OBJ, beacuse of a parsing error: {0}")]
    IntParseError(#[from] std::num::ParseIntError),
    #[error("Failed to load OBJ, because not all fields are present")]
    NotAllFieldsPresent,
    #[error("Failed to load OBJ, because an undefined position is used")]
    PositionNotFound,
    #[error("Failed to load OBJ, because an undefined uv is used")]
    UvNotFound,
    #[error("Failed to load OBJ, because an undefined normal is used")]
    NormalNotFound,
}

pub fn cook_obj(data: &[u8]) -> Result<CookedMesh, LoadOBJError> {
    let position_regex =
        regex::Regex::new(r"v (-?\d+\.\d+) (-?\d+\.\d+) (-?\d+\.\d+)").unwrap();
    let uv_regex = regex::Regex::new(r"vt (-?\d+\.\d+) (-?\d+\.\d+)").unwrap();
    let normal_regex =
        regex::Regex::new(r"vn (-?\d+\.\d+) (-?\d+\.\d+) (-?\d+\.\d+)")
            .unwrap();
    let index_regex = regex::Regex::new(
        r"f (\d+)/(\d+)/(\d+) (\d+)/(\d+)/(\d+) (\d+)/(\d+)/(\d+)",
    )
    .unwrap();

    let contents = String::from_utf8_lossy(data);

    let positions: Vec<Vec3f> = position_regex
        .captures_iter(&contents)
        .map(|m| {
            let (_, groups) = m.extract::<3>();
            let x = groups[0].parse::<f32>().unwrap();
            let y = groups[1].parse::<f32>().unwrap();
            let z = groups[2].parse::<f32>().unwrap();
            Vec3f::new(x, y, z)
        })
        .collect();
    let uvs: Vec<Vec2f> = uv_regex
        .captures_iter(&contents)
        .map(|m| {
            let (_, groups) = m.extract::<2>();
            let x = groups[0].parse::<f32>().unwrap();
            let y = groups[1].parse::<f32>().unwrap();
            Vec2f::new(x, y)
        })
        .collect();
    let normals: Vec<Vec3f> = normal_regex
        .captures_iter(&contents)
        .map(|m| {
            let (_, groups) = m.extract::<3>();
            let x = groups[0].parse::<f32>().unwrap();
            let y = groups[1].parse::<f32>().unwrap();
            let z = groups[2].parse::<f32>().unwrap();
            Vec3f::new(x, y, z)
        })
        .collect();
    let indices: Vec<u32> = index_regex
        .captures_iter(&contents)
        .map(|m| {
            let (_, groups) = m.extract::<9>();
            let v1 = groups[0].parse::<u32>().unwrap();
            let v2 = groups[3].parse::<u32>().unwrap();
            let v3 = groups[6].parse::<u32>().unwrap();
            let v1t = groups[1].parse::<u32>().unwrap();
            let v2t = groups[4].parse::<u32>().unwrap();
            let v3t = groups[7].parse::<u32>().unwrap();
            let v1n = groups[2].parse::<u32>().unwrap();
            let v2n = groups[5].parse::<u32>().unwrap();
            let v3n = groups[8].parse::<u32>().unwrap();
            [v1, v2, v3]
        })
        .flatten()
        .collect();

    Ok(CookedMesh::new(positions, normals, uvs, indices))
}
