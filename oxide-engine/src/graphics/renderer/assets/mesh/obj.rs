use std::{fs::File, io::Read};

use thiserror::Error;

use crate::{
    graphics::renderer::{DefaultVertex, assets::mesh::Mesh},
    math::vec::{Vec2f, Vec3f},
};

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

pub fn load_obj(file_path: &str) -> Result<Mesh, LoadOBJError> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut normals = Vec::new();
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut index_counter = 0;

    for line in contents.lines() {
        let mut values = line.split(' ');
        match values.next() {
            Some("v") => {
                let v1 = values
                    .next()
                    .ok_or(LoadOBJError::NotAllFieldsPresent)?
                    .parse::<f32>()?;
                let v2 = values
                    .next()
                    .ok_or(LoadOBJError::NotAllFieldsPresent)?
                    .parse::<f32>()?;
                let v3 = values
                    .next()
                    .ok_or(LoadOBJError::NotAllFieldsPresent)?
                    .parse::<f32>()?;
                positions.push(Vec3f::new(v1, v2, v3));
            },
            Some("vt") => {
                let t1 = values
                    .next()
                    .ok_or(LoadOBJError::NotAllFieldsPresent)?
                    .parse::<f32>()?;
                let t2 = values
                    .next()
                    .ok_or(LoadOBJError::NotAllFieldsPresent)?
                    .parse::<f32>()?;
                uvs.push(Vec2f::new(t1, t2));
            },
            Some("vn") => {
                let n1 = values
                    .next()
                    .ok_or(LoadOBJError::NotAllFieldsPresent)?
                    .parse::<f32>()?;
                let n2 = values
                    .next()
                    .ok_or(LoadOBJError::NotAllFieldsPresent)?
                    .parse::<f32>()?;
                let n3 = values
                    .next()
                    .ok_or(LoadOBJError::NotAllFieldsPresent)?
                    .parse::<f32>()?;
                normals.push(Vec3f::new(n1, n2, n3));
            },
            Some("f") => {
                for i in 0..=2 {
                    let mut index = values
                        .next()
                        .ok_or(LoadOBJError::NotAllFieldsPresent)?
                        .split('/');
                    let position_id = index.next().map(|x| x.parse::<u32>());
                    let position = match position_id {
                        Some(val) => *positions
                            .get(val? as usize - 1)
                            .ok_or(LoadOBJError::PositionNotFound)?,
                        None => Vec3f::ZERO,
                    };
                    let uv_id = index.next().map(|x| x.parse::<u32>());
                    let uv = match uv_id {
                        Some(val) => *uvs.get(val? as usize - 1).ok_or(LoadOBJError::UvNotFound)?,
                        None => Vec2f::ZERO,
                    };
                    let normal_id = index.next().map(|x| x.parse::<u32>());
                    let normal = match normal_id {
                        Some(val) => *normals
                            .get(val? as usize - 1)
                            .ok_or(LoadOBJError::NormalNotFound)?,
                        None => Vec3f::ZERO,
                    };
                    vertices.push(DefaultVertex::new(position, normal, uv));
                    indices.push(index_counter + i);
                }
                index_counter += 3;
            },
            Some(_) | None => continue,
        }
    }

    Ok(Mesh::new(vertices, indices))
}
