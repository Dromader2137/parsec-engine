use std::{collections::HashMap, path::Path, str::SplitWhitespace};

use parsec_engine_math::vec::{Vec2f, Vec3f};

use crate::{
    assets::{
        core::mesh::{InputMesh, Mesh},
        source_file::{AssetCookContext, AssetSourceFileHandler},
    },
    error::ParsecError,
};

#[derive(Debug, thiserror::Error)]
enum ObjError {
    #[error("obj line {0}: invalid number")]
    InvalidNumber(usize),
    #[error("obj line {0}: invalid face index")]
    InvalidIndex(usize),
    #[error("obj line {0}: face has fewer than 3 vertices")]
    DegenerateFace(usize),
}

pub struct ObjFileHandler;

impl AssetSourceFileHandler for ObjFileHandler {
    const EXTENSION: &'static str = "obj";

    fn cook(
        asset_source_path: &Path,
        ctx: AssetCookContext,
    ) -> Result<(), ParsecError> {
        let source = std::fs::read_to_string(asset_source_path)?;
        for (name, mesh) in parse_obj(&source)? {
            ctx.add_asset::<Mesh>(&name, mesh)?;
        }
        Ok(())
    }
}

fn parse_obj(source: &str) -> Result<Vec<(String, InputMesh)>, ObjError> {
    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut normals = Vec::new();
    let mut meshes = vec![(String::from("main"), InputMesh::default())];
    let mut vertex_map = HashMap::new();

    for (line_idx, line) in source.lines().enumerate() {
        let line_num = line_idx + 1;
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("v") => {
                let [x, y, z] = parse_floats(parts, line_num)?;
                positions.push(Vec3f::new(x, y, z));
            },
            Some("vn") => {
                let [x, y, z] = parse_floats(parts, line_num)?;
                normals.push(Vec3f::new(x, y, z));
            },
            Some("vt") => {
                let [u, v] = parse_floats(parts, line_num)?;
                uvs.push(Vec2f::new(u, v));
            },
            Some("o" | "g") => {
                let name = parts.collect::<Vec<_>>().join(" ");
                let name = if name.is_empty() { "main".into() } else { name };
                if meshes.last().is_some_and(|(_, m)| m.indices.is_empty()) {
                    meshes.pop();
                }
                meshes.push((name, InputMesh::default()));
                vertex_map.clear();
            },
            Some("f") => {
                let mesh = &mut meshes.last_mut().unwrap().1;
                let mut face = Vec::new();
                for vertex in parts {
                    let mut indices = vertex.split('/');
                    let p = resolve_index(indices.next(), positions.len(), line_num)?
                        .ok_or(ObjError::InvalidIndex(line_num))?;
                    let t = resolve_index(indices.next(), uvs.len(), line_num)?;
                    let n = resolve_index(indices.next(), normals.len(), line_num)?;
                    let index = *vertex_map.entry((p, t, n)).or_insert_with(|| {
                        mesh.positions.push(positions[p]);
                        mesh.uvs.push(t.map_or(Vec2f::ZERO, |t| uvs[t]));
                        mesh.normals.push(n.map_or(Vec3f::ZERO, |n| normals[n]));
                        mesh.positions.len() as u32 - 1
                    });
                    face.push(index);
                }
                if face.len() < 3 {
                    return Err(ObjError::DegenerateFace(line_num));
                }
                for i in 1..face.len() - 1 {
                    mesh.indices.extend([face[0], face[i], face[i + 1]]);
                }
            },
            _ => {},
        }
    }

    meshes.retain(|(_, m)| !m.indices.is_empty());
    Ok(meshes)
}

fn parse_floats<const N: usize>(
    mut parts: SplitWhitespace,
    line: usize,
) -> Result<[f32; N], ObjError> {
    let mut out = [0.0; N];
    for value in out.iter_mut() {
        *value = parts
            .next()
            .and_then(|p| p.parse().ok())
            .ok_or(ObjError::InvalidNumber(line))?;
    }
    Ok(out)
}

fn resolve_index(
    index: Option<&str>,
    len: usize,
    line: usize,
) -> Result<Option<usize>, ObjError> {
    let Some(index) = index.filter(|i| !i.is_empty()) else {
        return Ok(None);
    };
    let index: isize = index.parse().map_err(|_| ObjError::InvalidIndex(line))?;
    let resolved = match index {
        1.. => Some(index as usize - 1),
        ..0 => len.checked_add_signed(index),
        0 => None,
    };
    resolved
        .filter(|&i| i < len)
        .map(Some)
        .ok_or(ObjError::InvalidIndex(line))
}
