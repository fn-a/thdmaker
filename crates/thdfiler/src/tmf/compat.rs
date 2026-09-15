//! 3MF ⇄ Bevy conversions.
//!
//! Coordinates are normalised to millimetres and to the Y-up convention Bevy
//! uses, the build items and the component hierarchy are resolved and their
//! transforms are baked into the vertices (a Bevy mesh carries no transform
//! here). A Bevy mesh has no material slots either, so 3MF materials become
//! vertex colours: a single material mesh keeps its index buffer; a mesh that
//! mixes materials is expanded into a triangle soup.

use std::iter;
use glam::{Mat4, Vec3};
use bevy_asset::RenderAssetUsages;
use bevy_color::{ColorToComponents, ColorToPacked, LinearRgba, Srgba};
use bevy_mesh::{Indices, Mesh, PrimitiveTopology};
use crate::util::math::{identitied, transpoint, axis_ztoyup};
use super::define::{
    model::{Model as TmfModel, ObjectContent as TmfObjContent, Resources as TmfResources},
    package::Package as TmfPackage,
    primitive::{
        Color as TmfColor, Matrix3D as TmfMatrix3D, Mesh as TmfMesh, Triangle as TmfTriangle,
        Triangles as TmfTriangles, Vertex as TmfVertex, Vertices as TmfVertices,
    }
};

impl TmfVertex {
    /// Z-up (`x, y, z`) to Y-up (`x, z, -y`), scaled to millimetres.
    #[inline]
    fn yup(&self, scale: f64) -> [f32; 3] {
        [
            (self.x * scale) as f32,
            (self.z * scale) as f32,
            (-self.y * scale) as f32,
        ]
    }
}

impl From<&[f32; 3]> for TmfVertex {
    /// Y-up (`x, y, z`) to the Z-up 3MF convention (`x, -z, y`), for a position
    /// or a normal. The 3MF unit of the loaded models is millimetre.
    #[inline]
    fn from(p: &[f32; 3]) -> Self {
        Self::new(p[0] as f64, -p[2] as f64, p[1] as f64)
    }
}

impl TmfColor {
    /// The sRGB colour as a linear colour.
    #[inline]
    fn linear(&self) -> LinearRgba {
        Srgba::rgba_u8(self.r, self.g, self.b, self.a).into()
    }
}

impl From<LinearRgba> for TmfColor {
    /// A linear colour as the sRGB bytes 3MF stores.
    #[inline]
    fn from(linear: LinearRgba) -> Self {
        let [r, g, b, a] = Srgba::from(linear).to_u8_array();
        Self { r, g, b, a }
    }
}

impl TmfMatrix3D {
    /// The matrix as a Y-up column major transform.
    ///
    /// `Matrix3D` stores the rows `m0..m3` with the translation in the last row
    /// (`p' = p · M`), while `Mat4` is column major (`p' = M · p`): the columns
    /// of the result are the rows of `Matrix3D`.  The translation is scaled to
    /// millimetres and the whole matrix is then conjugated with the axis swap.
    fn yup(&self, scale: f64) -> Mat4 {
        let column = |row: [f64; 3]| Vec3::new(row[0] as f32, row[1] as f32, row[2] as f32);
        let m = Mat4::from_cols(
            column(self.m0).extend(0.0),
            column(self.m1).extend(0.0),
            column(self.m2).extend(0.0),
            Vec3::new(
                (self.m3[0] * scale) as f32,
                (self.m3[1] * scale) as f32,
                (self.m3[2] * scale) as f32,
            )
            .extend(1.0),
        );
        axis_ztoyup(&m)
    }
}

impl TmfResources {
    /// The linear colour of a `(pid, pindex)` property.
    ///
    /// Base materials and the materials extension colour groups are both
    /// searched; `pindex` indexes into the property group (0 based, as
    /// `Package` writes it) and defaults to its first entry.
    fn color(&self, pid: Option<u32>, pindex: Option<u32>) -> Option<LinearRgba> {
        let pid = pid?;
        let index = pindex.unwrap_or(0) as usize;

        if let Some(group) = self.base_materials.iter().find(|group| group.id == pid)
            && let Some(material) = group.materials.get(index)
        {
            Some(material.display_color.linear())
        } else if let Some(materials) = &self.materials
            && let Some(group) = materials.color_groups.iter().find(|group| group.id == pid)
            && let Some(color) = group.colors.get(index)
        {
            Some(color.color.linear())
        } else {
            None
        }
    }
}

impl TmfModel {
    /// Resolves an object (recursing through components) into `out` as
    /// `(mesh, world transform, material group, material index)`.
    fn descend<'a>(
        &'a self,
        objectid: u32,
        transform: Mat4,
        scale: f64,
        depth: usize,
        out: &mut Vec<(&'a TmfMesh, Mat4, Option<u32>, Option<u32>)>,
    ) {
        // Recursion guard for components cycles.
        if depth <= 64 {
            if let Some(object) = self.get_object(objectid) {
                match &object.content {
                    TmfObjContent::Mesh(mesh) => out.push((mesh, transform, object.pid, object.pindex)),
                    TmfObjContent::Components(components) => {
                        for component in &components.components {
                            let local = component
                                .transform
                                .as_ref()
                                .map(|m| m.yup(scale))
                                .unwrap_or(Mat4::IDENTITY);
                            // 3MF applies the component transform first, then the parent
                            // (`component · item` in the row-vector convention).
                            self.descend(
                                component.object_id,
                                transform * local,
                                scale,
                                depth + 1,
                                out,
                            );
                        }
                    }
                    // Beam lattices, boolean shapes, displacement meshes, level sets and
                    // production alternatives have no mesh representation here.
                    _ => {}
                }
            }
        }
    }

    /// Every mesh of the build items, with its world transform and material.
    fn placements(&self) -> Vec<(&TmfMesh, Mat4, Option<u32>, Option<u32>)> {
        let scale = self.unit.millimeter_into(1.0) as f64;
        let mut out = Vec::new();
        for item in &self.build.items {
            let transform = item
                .transform
                .as_ref()
                .map(|m| m.yup(scale))
                .unwrap_or(Mat4::IDENTITY);
            self.descend(item.object_id, transform, scale, 0, &mut out);
        }
        out
    }
}

impl TmfMesh {
    /// Builds a Bevy mesh for one object of `resources`, folding `transform` into the vertices.
    fn shim(&self, resources: &TmfResources, scale: f64, pid: Option<u32>, pindex: Option<u32>, transform: &Mat4) -> Mesh {
        let mut positions: Vec<[f32; 3]> = self
            .vertices
            .vertices
            .iter()
            .map(|vertex| vertex.yup(scale))
            .collect();
        let indices: Vec<u32> = self
            .triangles
            .triangles
            .iter()
            .flat_map(|t| [t.v1, t.v2, t.v3])
            .collect();
        // The triangle level property overrides the object level one.
        let slots: Vec<[f32; 4]> = self
            .triangles
            .triangles
            .iter()
            .map(|triangle| {
                resources
                    .color(triangle.pid.or(pid), triangle.p1.or(pindex))
                    .unwrap_or(LinearRgba::WHITE)
                    .to_f32_array()
            })
            .collect();

        if !identitied(transform) {
            for p in &mut positions {
                *p = transpoint(transform, p);
            }
        }

        let single = slots
            .first()
            .copied()
            .filter(|first| slots.iter().all(|slot| slot == first));

        let (positions, indices, colors) = match (single, slots.is_empty()) {
            // No material at all.
            (_, true) => (positions, indices, None),
            // One material: keep the index buffer, one colour per vertex.
            (Some(color), false) => {
                let colors = vec![color; positions.len()];
                (positions, indices, Some(colors))
            }
            // Mixed materials: expand so each triangle carries its own colour.
            (None, false) => {
                let mut soup = Vec::with_capacity(indices.len());
                let mut colors = Vec::with_capacity(indices.len());
                for (t, chunk) in indices.chunks(3).enumerate() {
                    for &i in chunk {
                        soup.push(positions[i as usize]);
                        colors.push(slots[t]);
                    }
                }
                (soup, Vec::new(), Some(colors))
            }
        };

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        if let Some(colors) = colors {
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        }
        if !indices.is_empty() {
            mesh.insert_indices(Indices::U32(indices));
        }
        mesh
    }
}

/// A bare 3MF mesh → Bevy mesh.
impl From<&TmfMesh> for Mesh {
    fn from(mesh: &TmfMesh) -> Self {
        mesh.shim(
            &TmfResources::default(),
            1.0,
            None,
            None,
            &Mat4::IDENTITY,
        )
    }
}

/// The build items of a 3MF model → one Bevy mesh per object.
impl From<&TmfModel> for Vec<Mesh> {
    fn from(model: &TmfModel) -> Self {
        let scale = model.unit.millimeter_into(1.0) as f64;
        model
            .placements()
            .into_iter()
            .map(|(mesh, transform, pid, pindex)| {
                mesh.shim(&model.resources, scale, pid, pindex, &transform)
            })
            .collect()
    }
}

/// Every model of a package → one Bevy mesh per object (the production
/// extension keeps its parts in the sub models).
impl From<&TmfPackage> for Vec<Mesh> {
    fn from(package: &TmfPackage) -> Self {
        let mut meshes = Vec::new();
        for model in iter::once(&package.model).chain(package.models.values()) {
            meshes.extend(Vec::<Mesh>::from(model));
        }
        meshes
    }
}

/// Bevy → 3MF mesh, in the Z-up 3MF convention.
impl From<&Mesh> for TmfMesh {
    fn from(mesh: &Mesh) -> Self {
        if let Some(positions) = mesh
            .try_attribute(Mesh::ATTRIBUTE_POSITION)
            .ok()
            .and_then(|v| v.as_float3())
        {
            let vertices = positions
                .iter()
                .map(TmfVertex::from)
                .collect();
            let indices: Vec<usize> = match mesh.try_indices_option() {
                Ok(Some(indices)) => indices.iter().collect(),
                _ => (0..positions.len()).collect(),
            };
            let index = |i: usize| -> u32 { indices.get(i).copied().unwrap_or(0) as u32 };
    
            let triangles = match mesh.primitive_topology() {
                PrimitiveTopology::TriangleList | PrimitiveTopology::TriangleStrip => {
                    (0..indices.len() / 3)
                        .map(|t| TmfTriangle::new(index(t * 3), index(t * 3 + 1), index(t * 3 + 2)))
                        .collect()
                }
                _ => Vec::new(),
            };
    
            TmfMesh {
                vertices: TmfVertices { vertices },
                triangles: TmfTriangles { triangles },
                ..Default::default()
            }
        } else {
            TmfMesh::default()
        }
    }
}

impl From<Mesh> for TmfMesh {
    fn from(mesh: Mesh) -> Self {
        Self::from(&mesh)
    }
}
