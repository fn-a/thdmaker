//! AMF ⇄ Bevy conversions.
//!
//! Coordinates are normalised to millimetres and to the Y-up convention Bevy
//! uses, the constellation instances are resolved and their transforms are
//! baked into the vertices (a Bevy mesh carries no transform here), and AMF
//! colours become vertex colours. Nothing intermediate is built.

use glam::{Mat4, Vec3};
use bevy_asset::RenderAssetUsages;
use bevy_color::{ColorToComponents, LinearRgba, Srgba};
use bevy_mesh::{Indices, Mesh, VertexAttributeValues, PrimitiveTopology};
use crate::util::math::{
    identitied, normatrix, normalize, transdirect, transpoint, axis_ztoyup,
};
use super::define::{
    Color as AmfColor, Coordinates as AmfCoordinates, Document as AmfDocument,
    Instance as AmfInstance, Mesh as AmfMesh, Normal as AmfNormal, Object as AmfObject,
    Triangle as AmfTriangle, Vertex as AmfVertex, Volume as AmfVolume,
};

impl AmfCoordinates {
    /// Z-up (`x, y, z`) to Y-up (`x, z, -y`), scaled to millimetres.
    #[inline]
    fn position(&self, scale: f64) -> [f32; 3] {
        [
            (self.x * scale) as f32,
            (self.z * scale) as f32,
            (-self.y * scale) as f32,
        ]
    }
}

impl From<&[f32; 3]> for AmfCoordinates {
    /// Y-up millimetre to Z-up AMF coordinates (the AMF unit is millimetre).
    #[inline]
    fn from(p: &[f32; 3]) -> Self {
        Self {
            x: p[0] as f64,
            y: -p[2] as f64,
            z: p[1] as f64,
        }
    }
}

impl AmfNormal {
    /// Z-up normal to Y-up.
    #[inline]
    fn normal(&self) -> [f32; 3] {
        [self.nx as f32, self.nz as f32, -self.ny as f32]
    }
}

impl AmfColor {
    /// The sRGB colour as a linear colour.
    #[inline]
    fn linear(&self) -> LinearRgba {
        Srgba::new(
            channel(&self.r),
            channel(&self.g),
            channel(&self.b),
            self.a.as_deref().map(channel).unwrap_or(1.0),
        )
        .into()
    }
}

impl From<LinearRgba> for AmfColor {
    /// A linear colour as AMF sRGB strings.
    #[inline]
    fn from(linear: LinearRgba) -> Self {
        let srgba = Srgba::from(linear);
        let channel = |v: f32| format!("{:.6}", v);
        Self {
            r: channel(srgba.red),
            g: channel(srgba.green),
            b: channel(srgba.blue),
            a: Some(channel(srgba.alpha)),
        }
    }
}

impl From<[f32; 4]> for AmfColor {
    /// A linear RGBA as AMF sRGB strings.
    #[inline]
    fn from(rgba: [f32; 4]) -> Self {
        LinearRgba::from_f32_array(rgba).into()
    }
}

/// Parses one AMF colour channel (`1.0`, `0.5`, `50%`) into an sRGB value.
#[inline]
fn channel(value: &str) -> f32 {
    let value = value.trim();
    let parsed = match value.strip_suffix('%') {
        Some(percent) => percent.trim().parse::<f32>().map(|v| v / 100.0),
        None => value.parse::<f32>(),
    };
    parsed.unwrap_or(1.0).clamp(0.0, 1.0)
}

impl AmfInstance {
    /// The transform of an AMF `<instance>`, expressed in the Y-up space Bevy uses.
    ///
    /// The AMF specification composes the instance transform as
    /// `T * Rx * Ry * Rz` for row vectors; in the column vector convention the same
    /// product reads `Rz · Ry · Rx · T`, which is built here (in Z-up space) before
    /// the axis swap conjugation.
    fn matrix(&self, scale: f64) -> Mat4 {
        let t = Vec3::new(
            self.deltax.unwrap_or(0.0) as f32 * scale as f32,
            self.deltay.unwrap_or(0.0) as f32 * scale as f32,
            self.deltaz.unwrap_or(0.0) as f32 * scale as f32,
        );
        let rx = (self.rx.unwrap_or(0.0) as f32).to_radians();
        let ry = (self.ry.unwrap_or(0.0) as f32).to_radians();
        let rz = (self.rz.unwrap_or(0.0) as f32).to_radians();
    
        let m = Mat4::from_rotation_z(rz)
            * Mat4::from_rotation_y(ry)
            * Mat4::from_rotation_x(rx)
            * Mat4::from_translation(t);
    
        axis_ztoyup(&m)
    }
}

impl AmfDocument {
    /// The objects of a document, each with the transform it is placed at.
    ///
    /// An object referenced by a constellation is emitted once per instance, an
    /// unreferenced object at its own coordinates.
    fn placements(&self) -> Vec<(&AmfObject, Mat4)> {
        let scale = self.unit.clone().unwrap_or_default().scale();
    
        let mut instanced = vec![false; self.objects.len()];
        for constellation in &self.constellations {
            for instance in &constellation.instances {
                if let Some(i) = self
                    .objects
                    .iter()
                    .position(|o| o.id == instance.object_id)
                {
                    instanced[i] = true;
                }
            }
        }
    
        let mut out: Vec<(&AmfObject, Mat4)> = Vec::with_capacity(self.objects.len());
        for (i, object) in self.objects.iter().enumerate() {
            if !instanced[i] {
                out.push((object, Mat4::IDENTITY));
            }
        }
    
        for constellation in &self.constellations {
            for instance in &constellation.instances {
                if let Some(object) = self.get_object(instance.object_id) {
                    out.push((object, instance.matrix(scale)));
                }
            }
        }
    
        out
    }
}

impl AmfObject {
    /// Builds a Bevy mesh for one AMF object, folding `transform` into the vertices.
    fn shim(&self, scale: f64, transform: &Mat4) -> Mesh {
        let vertices = &self.mesh.vertices.vertices;
        let mut positions: Vec<[f32; 3]> = vertices
            .iter()
            .map(|v| v.coordinates.position(scale))
            .collect();
        let mut normals: Option<Vec<[f32; 3]>> = if !vertices.is_empty() && vertices.iter().all(|v| !v.normals.is_empty()) {
            Some(vertices.iter().map(|v| v.normals[0].normal()).collect())
        } else {
            None
        };
        let colors: Option<Vec<[f32; 4]>> =
            if !vertices.is_empty() && vertices.iter().all(|v| !v.colors.is_empty()) {
                Some(
                    vertices
                        .iter()
                        .filter_map(|v| v.colors.first().map(AmfColor::linear))
                        .map(ColorToComponents::to_f32_array)
                        .collect(),
                )
            } else {
                self.mesh
                    .volume
                    .colors
                    .first()
                    .map(AmfColor::linear)
                    .or_else(|| self.colors.first().map(AmfColor::linear))
                    .map(|color| vec![color.to_f32_array(); vertices.len()])
            };
        let indices: Vec<u32> = self
            .mesh
            .volume
            .triangles
            .iter()
            .flat_map(|t| [t.v1, t.v2, t.v3])
            .collect();
    
        if !identitied(transform) {
            for p in &mut positions {
                *p = transpoint(transform, p);
            }
            if let Some(normals) = &mut normals {
                let nm = normatrix(transform);
                for n in normals {
                    *n = transdirect(&nm, n);
                    normalize(n);
                }
            }
        }
    
        let mut mesh =
            Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        if let Some(normals) = normals {
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        }
        if let Some(colors) = colors {
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        }
        if !indices.is_empty() {
            mesh.insert_indices(Indices::U32(indices));
        }
        mesh
    }
}

/// One AMF object → Bevy mesh, assuming millimetres and no instance.
impl From<&AmfObject> for Mesh {
    fn from(object: &AmfObject) -> Self {
        object.shim(1.0, &Mat4::IDENTITY)
    }
}

impl From<&AmfMesh> for Mesh {
    fn from(mesh: &AmfMesh) -> Self {
        let object = AmfObject {
            mesh: mesh.clone(),
            ..Default::default()
        };
        Self::from(&object)
    }
}

/// A whole AMF document → one Bevy mesh per placed object.
impl From<&AmfDocument> for Vec<Mesh> {
    fn from(document: &AmfDocument) -> Self {
        let scale = document.unit.clone().unwrap_or_default().scale();
        document.placements()
            .into_iter()
            .filter(|(object, _)| !object.mesh.vertices.vertices.is_empty())
            .map(|(object, transform)| object.shim(scale, &transform))
            .collect()
    }
}

/// Bevy → AMF object, in the Z-up millimetre convention.
impl From<&Mesh> for AmfObject {
    fn from(mesh: &Mesh) -> Self {
        let mut object = AmfObject::default();

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
    
            object.mesh.vertices.vertices = positions
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let mut vertex = AmfVertex {
                        coordinates: AmfCoordinates::from(p),
                        ..Default::default()
                    };
                    if let Some(n) = normals {
                        vertex.normals.push(AmfNormal {
                            nx: n[i][0] as f64,
                            ny: -n[i][2] as f64,
                            nz: n[i][1] as f64,
                        });
                    }
                    if let Some(c) = colors {
                        vertex.colors.push(c[i].into());
                    }
                    vertex
                })
                .collect();
    
            let indices = match mesh.try_indices_option() {
                Ok(Some(indices)) => indices.iter().collect::<Vec<_>>(),
                _ => (0..positions.len()).collect(),
            };
            let index = |i: usize| -> u32 { indices.get(i).copied().unwrap_or(0) as u32 };
    
            object.mesh.volume = AmfVolume {
                triangles: match mesh.primitive_topology() {
                    PrimitiveTopology::TriangleList | PrimitiveTopology::TriangleStrip => {
                        (0..indices.len() / 3)
                            .map(|t| AmfTriangle {
                                v1: index(t * 3),
                                v2: index(t * 3 + 1),
                                v3: index(t * 3 + 2),
                                ..Default::default()
                            })
                            .collect()
                    }
                    _ => Vec::new(),
                },
                colors: colors
                    .and_then(|c| c.first().copied())
                    .map(Into::into)
                    .into_iter()
                    .collect(),
                ..Default::default()
            };
        }

        object
    }
}

impl From<Mesh> for AmfObject {
    fn from(mesh: Mesh) -> Self {
        Self::from(&mesh)
    }
}
