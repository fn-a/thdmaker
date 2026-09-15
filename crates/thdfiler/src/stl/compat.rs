//! STL ⇄ Bevy conversions.
//!
//! A STL facet soup becomes a Bevy mesh in a single pass: Z-up to Y-up, one
//! flat normal per facet (derived from the winding order when the file stores a
//! zero one) and facet colours as vertex colours. Nothing intermediate is
//! built.

use glam::Vec3;
use bevy_asset::RenderAssetUsages;
use bevy_color::{ColorToComponents, ColorToPacked, LinearRgba, Srgba};
use bevy_mesh::{Indices, Mesh, PrimitiveTopology, VertexAttributeValues};
use super::define::{
    Color as StlColor, Format as StlFormat, Mesh as StlMesh, Vertex as StlVertex,
};

impl StlVertex {
    /// Z-up (`x, y, z`) to Y-up (`x, z, -y`), for a position or a normal.
    #[inline]
    fn yup(&self) -> [f32; 3] {
        [self.x, self.z, -self.y]
    }

    /// Y-up (`x, y, z`) to the Z-up STL convention (`x, -z, y`), for a position
    /// or a normal.
    #[inline]
    fn zup(p: &[f32; 3]) -> Self {
        Self {
            x: p[0],
            y: -p[2],
            z: p[1],
        }
    }
}

impl StlColor {
    /// The sRGB facet colour as a linear colour.
    #[inline]
    fn linear(&self) -> LinearRgba {
        Srgba::rgba_u8(self.r, self.g, self.b, self.a).into()
    }
}

impl From<LinearRgba> for StlColor {
    /// A linear colour as the sRGB bytes STL stores.
    #[inline]
    fn from(linear: LinearRgba) -> Self {
        let [r, g, b, a] = Srgba::from(linear).to_u8_array();
        Self { r, g, b, a }
    }
}

/// Triangle normal from three points, in their own space.
#[inline]
fn trgnormal(a: &[f32; 3], b: &[f32; 3], c: &[f32; 3]) -> [f32; 3] {
    let a = Vec3::from_array(*a);
    let b = Vec3::from_array(*b);
    let c = Vec3::from_array(*c);
    (b - a).cross(c - a).normalize_or_zero().to_array()
}

/// STL → Bevy.
impl From<&StlMesh> for Mesh {
    /// Builds a Bevy mesh for the facet soup.
    ///
    /// STL has no shared vertices, so each triangle carries its own three
    /// vertices.
    fn from(stl: &StlMesh) -> Self {
        let triangles = stl.indices.len();
        let mut positions = Vec::with_capacity(triangles * 3);
        let mut normals = Vec::with_capacity(triangles * 3);
        let facolors = stl.colors.as_ref().filter(|colors| !colors.is_empty());
        let mut colors: Option<Vec<[f32; 4]>> = facolors.map(|_| Vec::with_capacity(triangles * 3));

        for (t, idx) in stl.indices.iter().enumerate() {
            let p = [
                stl.vertices[idx[0]].yup(),
                stl.vertices[idx[1]].yup(),
                stl.vertices[idx[2]].yup(),
            ];
            positions.extend_from_slice(&p);

            // Prefer the stored facet normal but fall back to the winding
            // order: STL exporters regularly write a zero normal.
            let stored = stl.normals.get(t).map(StlVertex::yup).unwrap_or([0.0; 3]);
            let normal = if Vec3::from_array(stored).length_squared() > 1e-12 {
                stored
            } else {
                trgnormal(&p[0], &p[1], &p[2])
            };
            normals.extend_from_slice(&[normal; 3]);

            if let (Some(facet_colors), Some(colors)) = (facolors, colors.as_mut()) {
                if let Some(color) = facet_colors.get(t) {
                    let rgba = color.linear().to_f32_array();
                    colors.extend_from_slice(&[rgba; 3]);
                }
            }
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_indices(Indices::U32((0..triangles as u32 * 3).collect()));
        if let Some(colors) = colors {
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        }
        mesh
    }
}

impl From<StlMesh> for Mesh {
    fn from(stl: StlMesh) -> Self {
        Self::from(&stl)
    }
}

/// Bevy → STL: a binary facet soup in the Z-up STL convention.
impl From<&Mesh> for StlMesh {
    fn from(mesh: &Mesh) -> Self {
        let mut stl = StlMesh {
            format: StlFormat::Binary,
            ..Default::default()
        };

        if let Some(positions) = mesh
            .try_attribute(Mesh::ATTRIBUTE_POSITION)
            .ok()
            .and_then(|v| v.as_float3())
        {

            let normals = mesh
                .try_attribute(Mesh::ATTRIBUTE_NORMAL)
                .ok()
                .and_then(|v| v.as_float3());
            let colors = mesh
                .try_attribute(Mesh::ATTRIBUTE_COLOR)
                .ok()
                .and_then(|v| match v {
                    VertexAttributeValues::Float32x4(c) => Some(c.as_slice()),
                    _ => None,
                });
    
            let indices: Vec<usize> = match mesh.try_indices_option() {
                Ok(Some(indices)) => indices.iter().collect(),
                _ => (0..positions.len()).collect(),
            };
            let index = |i: usize| -> usize { indices.get(i).copied().unwrap_or(0) };
    
            for t in 0..indices.len() / 3 {
                let idx = [index(t * 3), index(t * 3 + 1), index(t * 3 + 2)];
                let p = [
                    StlVertex::zup(&positions[idx[0]]),
                    StlVertex::zup(&positions[idx[1]]),
                    StlVertex::zup(&positions[idx[2]]),
                ];
    
                let base = stl.vertices.len();
                stl.vertices.extend_from_slice(&p);
                stl.indices.push([base, base + 1, base + 2]);
    
                // The facet normal: the mean of the vertex normals, or the winding
                // order when the mesh carries none.
                let normal = match normals {
                    Some(normals) => {
                        let n = (Vec3::from_array(normals[idx[0]])
                            + Vec3::from_array(normals[idx[1]])
                            + Vec3::from_array(normals[idx[2]]))
                        .normalize_or_zero();
                        StlVertex::zup(&n.to_array())
                    }
                    None => StlVertex::zup(&trgnormal(
                        &[p[0].x, p[0].y, p[0].z],
                        &[p[1].x, p[1].y, p[1].z],
                        &[p[2].x, p[2].y, p[2].z],
                    )),
                };
                stl.normals.extend_from_slice(&[normal; 3]);
    
                if let Some(colors) = colors {
                    let [r, g, b, a] = colors[idx[0]];
                    stl.colors
                        .get_or_insert_with(Vec::new)
                        .push(StlColor::from(LinearRgba::new(r, g, b, a)));
                }
            }
    
            stl.color = stl.colors.as_ref().and_then(|colors| colors.first().copied());
        }

        stl
    }
}

impl From<Mesh> for StlMesh {
    fn from(mesh: Mesh) -> Self {
        Self::from(&mesh)
    }
}
