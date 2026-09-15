//! Tris ⇄ Bevy geometry, behind the `bevy` feature.
//!
//! The conversions move ownership of the attribute arrays; when only a
//! reference is available the arrays are cloned once, element-wise,
//! without any per-vertex arithmetic. The cheapest conversion is the borrow
//! path the slicer uses, which copies `positions` and `indices` only.
//!
//! Matrices need no conversion at all: the tris `Mat4` and the bevy `Mat4` are
//! both glam `Mat4`.

use bevy_asset::RenderAssetUsages;
use bevy_mesh::{Indices, Mesh, PrimitiveTopology, VertexAttributeValues};
use super::define::{Mesh as TrisMesh, Topology as TrisTopology};

impl From<TrisTopology> for PrimitiveTopology {
    #[inline]
    fn from(topology: TrisTopology) -> Self {
        match topology {
            TrisTopology::PointList => Self::PointList,
            TrisTopology::LineList => Self::LineList,
            TrisTopology::LineStrip => Self::LineStrip,
            TrisTopology::TriangleList => Self::TriangleList,
            TrisTopology::TriangleStrip => Self::TriangleStrip,
        }
    }
}

impl From<PrimitiveTopology> for TrisTopology {
    #[inline]
    fn from(topology: PrimitiveTopology) -> Self {
        match topology {
            PrimitiveTopology::PointList => Self::PointList,
            PrimitiveTopology::LineList => Self::LineList,
            PrimitiveTopology::LineStrip => Self::LineStrip,
            PrimitiveTopology::TriangleList => Self::TriangleList,
            PrimitiveTopology::TriangleStrip => Self::TriangleStrip,
        }
    }
}

/// Zero copy: the attribute arrays are moved into the Bevy mesh.
impl From<TrisMesh> for Mesh {
    fn from(tmesh: TrisMesh) -> Self {
        let mut mesh = Mesh::new(
            PrimitiveTopology::from(tmesh.topology),
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, tmesh.positions);
        if let Some(normals) = tmesh.normals {
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        }
        if let Some(uvs) = tmesh.uvs {
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        }
        if let Some(colors) = tmesh.colors {
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        }
        if !tmesh.indices.is_empty() {
            mesh.insert_indices(Indices::U32(tmesh.indices));
        }
        mesh
    }
}

/// Borrowed variant: one clone per attribute array, no per-vertex work.
impl From<&TrisMesh> for Mesh {
    fn from(tmesh: &TrisMesh) -> Self {
        let mut mesh = Mesh::new(
            PrimitiveTopology::from(tmesh.topology),
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, tmesh.positions.clone());
        if let Some(normals) = &tmesh.normals {
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals.clone());
        }
        if let Some(uvs) = &tmesh.uvs {
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs.clone());
        }
        if let Some(colors) = &tmesh.colors {
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors.clone());
        }
        if !tmesh.indices.is_empty() {
            mesh.insert_indices(Indices::U32(tmesh.indices.clone()));
        }
        mesh
    }
}

#[inline]
fn indices_ownmv(indices: Indices) -> Vec<u32> {
    match indices {
        Indices::U32(v) => v,
        Indices::U16(v) => v.into_iter().map(u32::from).collect(),
    }
}

#[inline]
fn indices_clone(indices: &Indices) -> Vec<u32> {
    match indices {
        Indices::U32(v) => v.clone(),
        Indices::U16(v) => v.iter().map(|i| u32::from(*i)).collect(),
    }
}

/// Ownership move of the `Float32x3` attributes.
fn values3_ownmv(values: VertexAttributeValues) -> Option<Vec<[f32; 3]>> {
    match values {
        VertexAttributeValues::Float32x3(v) => Some(v),
        other => other.as_float3().map(<[[f32; 3]]>::to_vec),
    }
}

/// Cloning the `Float32x3` attributes.
fn values3_clone(values: &VertexAttributeValues) -> Option<Vec<[f32; 3]>> {
    match values {
        VertexAttributeValues::Float32x3(v) => Some(v.clone()),
        other => other.as_float3().map(<[[f32; 3]]>::to_vec),
    }
}

/// Ownership move of the `Float32x2` attributes.
fn values2_ownmv(values: VertexAttributeValues) -> Option<Vec<[f32; 2]>> {
    match values {
        VertexAttributeValues::Float32x2(v) => Some(v),
        _ => None,
    }
}

/// Cloning the `Float32x2` attributes, the format tris declares for `uvs`.
fn values2_clone(values: &VertexAttributeValues) -> Option<Vec<[f32; 2]>> {
    match values {
        VertexAttributeValues::Float32x2(v) => Some(v.clone()),
        _ => None,
    }
}

/// Ownership move of the `Float32x4` attributes.
fn values4_ownmv(values: VertexAttributeValues) -> Option<Vec<[f32; 4]>> {
    match values {
        VertexAttributeValues::Float32x4(v) => Some(v),
        _ => None,
    }
}

/// Cloning the `Float32x4` attributes, the format tris declares for `colors`.
fn values4_clone(values: &VertexAttributeValues) -> Option<Vec<[f32; 4]>> {
    match values {
        VertexAttributeValues::Float32x4(v) => Some(v.clone()),
        _ => None,
    }
}

/// Zero copy: the attribute arrays are moved out of the Bevy mesh.
///
/// A Bevy mesh carries no material segmentation, so `primitives` and
/// `materials` stay empty.
impl From<Mesh> for TrisMesh {
    fn from(mut mesh: Mesh) -> Self {
        let topology = TrisTopology::from(mesh.primitive_topology());
        let positions =  match mesh.try_remove_attribute(Mesh::ATTRIBUTE_POSITION) {
            Ok(values) => values3_ownmv(values),
            Err(_) => mesh.try_attribute(Mesh::ATTRIBUTE_POSITION).ok().and_then(values3_clone),
        }.unwrap_or_default();
        
        let normals = match mesh.try_remove_attribute(Mesh::ATTRIBUTE_NORMAL) {
            Ok(values) => values3_ownmv(values),
            Err(_) => mesh.try_attribute(Mesh::ATTRIBUTE_NORMAL).ok().and_then(values3_clone),
        };
        let uvs = match mesh.try_remove_attribute(Mesh::ATTRIBUTE_UV_0) {
            Ok(values) => values2_ownmv(values),
            Err(_) => mesh.try_attribute(Mesh::ATTRIBUTE_UV_0).ok().and_then(values2_clone),
        };
        let colors = match mesh.try_remove_attribute(Mesh::ATTRIBUTE_COLOR) {
            Ok(values) => values4_ownmv(values),
            Err(_) => mesh.try_attribute(Mesh::ATTRIBUTE_COLOR).ok().and_then(values4_clone),
        };
        let indices = match mesh.try_remove_indices() {
            Ok(Some(indices)) => indices_ownmv(indices),
            _ => mesh.try_indices().ok().map(indices_clone).unwrap_or_default(),
        };

        TrisMesh {
            topology,
            positions,
            normals,
            uvs,
            colors,
            indices,
            ..Default::default()
        }
    }
}

/// Borrowed variant: one clone per attribute array.
impl From<&Mesh> for TrisMesh {
    fn from(mesh: &Mesh) -> Self {
        let topology = TrisTopology::from(mesh.primitive_topology());
        let positions = mesh
            .try_attribute(Mesh::ATTRIBUTE_POSITION)
            .ok()
            .and_then(values3_clone)
            .unwrap_or_default();
        let normals = mesh
            .try_attribute(Mesh::ATTRIBUTE_NORMAL)
            .ok()
            .and_then(values3_clone);
        let uvs = mesh
            .try_attribute(Mesh::ATTRIBUTE_UV_0)
            .ok()
            .and_then(values2_clone);
        let colors = mesh
            .try_attribute(Mesh::ATTRIBUTE_COLOR)
            .ok()
            .and_then(values4_clone);
        let indices = mesh.try_indices()
            .ok()
            .map(indices_clone)
            .unwrap_or_default();

        TrisMesh {
            topology,
            positions,
            normals,
            uvs,
            colors,
            indices,
            ..Default::default()
        }
    }
}

impl TrisMesh {
    /// Cheapest borrow conversion for the slicer: `positions` + `indices` only,
    /// no render-only attribute is copied.
    pub fn cheapest(mesh: &Mesh) -> Self {
        let positions = mesh
            .try_attribute(Mesh::ATTRIBUTE_POSITION)
            .ok()
            .and_then(values3_clone)
            .unwrap_or_default();
        let indices = mesh.try_indices()
            .ok()
            .map(indices_clone)
            .unwrap_or_default();

        TrisMesh {
            topology: TrisTopology::from(mesh.primitive_topology()),
            positions,
            indices,
            ..Default::default()
        }
    }
}
