//! Mesh toolkit and fallback demo scene.
//!
//! Outlines are 2D points in the slice plane (`Vec2::new(x, z)`); `loft` gives an
//! outline height along Y, `band` turns a pair of outlines into a solid with a
//! hole, `plate` cuts several holes into one outline and `revolve` spins a
//! profile into a curved surface. The scene built from them is a slice-preview
//! test rig: pressing `S` in the viewer slices every mesh and draws the toolpath.

use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_6, PI, TAU};
use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;

// Irregular shape helpers
//
// An outline is a list of 2D points in the slice plane: `Vec2::new(x, z)`.
// `loft` gives an outline height along Y, and `band` turns a pair of outlines
// into a solid with a hole, so concave profiles (an L, a star, a staircase) and
// openings (round, square, off centre) are both just data here.

// The outline in the winding whose triangles face +Y. The slicer reads a
// contour's winding to tell material from hole, so a solid's walls have to face
// out of the part for the preview offsets to land inside it.
fn clockwise(outline: &[Vec2]) -> Vec<Vec2> {
    let mut points = outline.to_vec();

    // Signed area of an outline: positive when it runs counter-clockwise in (x, z).
    let mut doubled = 0.0;
    for (index, point) in outline.iter().enumerate() {
        let next = outline[(index + 1) % outline.len()];
        doubled += point.x * next.y - next.x * point.y;
    }
    let area = doubled / 2.0;

    if area > 0.0 {
        points.reverse();
    }
    points
}

// Is `point` inside the triangle `a b c`? Points exactly on an edge do not
// count, so the doubled vertices of a bridged outline never block an ear.
fn intriangle(point: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    let side = |from: Vec2, to: Vec2| {
        (to.x - from.x) * (point.y - from.y) - (to.y - from.y) * (point.x - from.x)
    };
    let (one, two, three) = (side(a, b), side(b, c), side(c, a));
    (one < 0.0 && two < 0.0 && three < 0.0) || (one > 0.0 && two > 0.0 && three > 0.0)
}

// Ear clipping: triangles for a simple outline, concave corners included.
fn triangulate(outline: &[Vec2]) -> Vec<[usize; 3]> {
    let mut rest: Vec<usize> = (0..outline.len()).collect();
    let mut triangles = Vec::new();
    while rest.len() > 3 {
        let count = rest.len();
        let mut clipped = false;
        for index in 0..count {
            let (before, now, after) = (
                rest[(index + count - 1) % count],
                rest[index],
                rest[(index + 1) % count],
            );
            let (a, b, c) = (outline[before], outline[now], outline[after]);
            // A clockwise outline turns right at a convex corner.
            if (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x) >= 0.0 {
                continue;
            }
            let caught = rest.iter().any(|&other| {
                other != before
                    && other != now
                    && other != after
                    && intriangle(outline[other], a, b, c)
            });
            if caught {
                continue;
            }
            triangles.push([before, now, after]);
            rest.remove(index);
            clipped = true;
            break;
        }
        // A degenerate outline cannot be clipped further: keep what we have.
        if !clipped {
            break;
        }
    }
    if rest.len() == 3 {
        triangles.push([rest[0], rest[1], rest[2]]);
    }
    triangles
}

// A triangle soup with flat normals, so the sharp edges of a prism stay sharp.
fn solid(positions: Vec<[f32; 3]>) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.compute_flat_normals();
    mesh
}

// A solid lofted from a `bottom` outline to an equally long `top` outline: both
// caps are triangulated and matching vertices are joined by walls, so a concave
// outline becomes a prism and a rotated, resized or shifted top gives a
// twisted, tapered or sheared part.
fn loft(bottom: &[Vec2], top: &[Vec2], height: f32) -> Mesh {
    let bottom = clockwise(bottom);
    let top = clockwise(top);
    let count = bottom.len() as u32;
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut emit = |index: u32| {
        let (point, y) = if index < count {
            (bottom[index as usize], 0.0)
        } else {
            (top[(index - count) as usize], height)
        };
        positions.push([point.x, y, point.y]);
    };
    for [a, b, c] in triangulate(&top) {
        for index in [count + a as u32, count + b as u32, count + c as u32] {
            emit(index);
        }
    }
    for [a, b, c] in triangulate(&bottom) {
        for index in [c as u32, b as u32, a as u32] {
            emit(index);
        }
    }
    for index in 0..count {
        let next = (index + 1) % count;
        for corner in [index, next, count + next, index, count + next, count + index] {
            emit(corner);
        }
    }
    solid(positions)
}

// A solid with a through hole: the material between an `outer` and an `inner`
// outline of the same length, paired vertex by vertex and given `height`. The
// hole's wall is wound the other way, so it faces the hole.
fn band(outer: &[Vec2], inner: &[Vec2], height: f32) -> Mesh {
    let outer = clockwise(outer);
    let inner = clockwise(inner);
    let count = outer.len() as u32;
    // 0..count outer bottom, count.. outer top, then both for the hole.
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut emit = |index: u32| {
        let (point, y) = match index {
            low if low < count => (outer[low as usize], 0.0),
            high if high < 2 * count => (outer[(high - count) as usize], height),
            low if low < 3 * count => (inner[(low - 2 * count) as usize], 0.0),
            high => (inner[(high - 3 * count) as usize], height),
        };
        positions.push([point.x, y, point.y]);
    };
    for index in 0..count {
        let next = (index + 1) % count;
        let (outer_low, outer_next) = (index, next);
        let (outer_high, outer_top) = (count + index, count + next);
        let (hole_low, hole_next) = (2 * count + index, 2 * count + next);
        let (hole_high, hole_top) = (3 * count + index, 3 * count + next);
        // The band on top, the same band underneath with the triangles flipped.
        for corner in [outer_high, outer_top, hole_top, outer_high, hole_top, hole_high] {
            emit(corner);
        }
        for corner in [hole_next, outer_next, outer_low, hole_low, hole_next, outer_low] {
            emit(corner);
        }
        // The outer wall, then the hole's wall the other way round.
        for corner in [outer_low, outer_next, outer_top, outer_low, outer_top, outer_high] {
            emit(corner);
        }
        for corner in [hole_top, hole_next, hole_low, hole_high, hole_top, hole_low] {
            emit(corner);
        }
    }
    solid(positions)
}

// A plate with several holes: each hole is joined to the outline by a zero-width
// bridge, which is what lets one ear-clipped polygon describe a window or a
// perforated plate. The two sides of a bridge get no wall - a real face there
// would sit inside the material and show up in every slice.
fn plate(outer: &[Vec2], holes: &[Vec<Vec2>], height: f32) -> Mesh {
    let mut outline = clockwise(outer);
    for hole in holes {
        let mut hole = clockwise(hole);
        hole.reverse(); // a hole runs the other way round its outline
        // Bridge the hole to the outline where the two are closest.
        let (mut best, mut best_gap) = ((0usize, 0usize), f32::MAX);
        for (here, point) in outline.iter().enumerate() {
            for (there, other) in hole.iter().enumerate() {
                let gap = point.distance_squared(*other);
                if gap < best_gap {
                    best = (here, there);
                    best_gap = gap;
                }
            }
        }
        let (here, there) = best;
        let mut joined = outline[..=here].to_vec();
        joined.extend_from_slice(&hole[there..]);
        joined.extend_from_slice(&hole[..=there]);
        joined.extend_from_slice(&outline[here..]);
        outline = joined;
    }

    // A bridge edge is one whose reverse is also in the outline.
    let count = outline.len();
    let key = |point: Vec2| ((point.x * 1e4).round() as i64, (point.y * 1e4).round() as i64);
    let mut edges = std::collections::HashSet::new();
    for index in 0..count {
        edges.insert((key(outline[index]), key(outline[(index + 1) % count])));
    }
    let bridge = |from: Vec2, to: Vec2| edges.contains(&(key(to), key(from)));

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut emit = |index: u32| {
        let point = outline[index as usize % count];
        let y = if index < count as u32 { 0.0 } else { height };
        positions.push([point.x, y, point.y]);
    };
    for [a, b, c] in triangulate(&outline) {
        for index in [count as u32 + a as u32, count as u32 + b as u32, count as u32 + c as u32] {
            emit(index);
        }
    }
    for [a, b, c] in triangulate(&outline) {
        for index in [c as u32, b as u32, a as u32] {
            emit(index);
        }
    }
    for index in 0..count {
        let next = (index + 1) % count;
        if bridge(outline[index], outline[next]) {
            continue;
        }
        for corner in [
            index as u32,
            next as u32,
            count as u32 + next as u32,
            index as u32,
            count as u32 + next as u32,
            count as u32 + index as u32,
        ] {
            emit(corner);
        }
    }
    solid(positions)
}

// A solid of revolution: `profile` is a closed outline of (radius, height)
// points swept around the Y axis. A half ellipse becomes a ball, a quarter arc
// a dome, and a small square a ring whose section is square at every angle.
fn revolve(profile: &[Vec2], segments: usize) -> Mesh {
    let mut profile = clockwise(profile);
    // The opposite winding: what a profile has to be before it is revolved, so its
    // triangles end up facing away from the axis.
    profile.reverse();

    let count = profile.len();
    let point = |step: usize, index: usize| -> [f32; 3] {
        let angle = TAU * step as f32 / segments as f32;
        let (radius, height) = (profile[index % count].x, profile[index % count].y);
        [radius * angle.cos(), height, radius * angle.sin()]
    };
    let mut positions: Vec<[f32; 3]> = Vec::new();
    for step in 0..segments {
        for index in 0..count {
            let next = index + 1;
            for corner in [
                point(step, index),
                point(step, next),
                point(step + 1, next),
                point(step, index),
                point(step + 1, next),
                point(step + 1, index),
            ] {
                positions.push(corner);
            }
        }
    }
    solid(positions)
}

// Several triangle soups in one mesh. The slicer walks the parts independently,
// so disjoint solids give one loop per part per layer - which is how the nested
// ring shape reaches three loops on every layer.
fn combined(parts: &[Mesh]) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    for part in parts {
        positions.extend(
            part.try_attribute(Mesh::ATTRIBUTE_POSITION)
                .ok()
                .and_then(|values| values.as_float3())
                .map(<[[f32; 3]]>::to_vec)
                .unwrap_or_default(),
        );
    }
    solid(positions)
}

// The half of an ellipse that faces away from the axis, from pole to pole.
fn ellipse_profile(half_width: f32, half_height: f32, segments: usize) -> Vec<Vec2> {
    (0..=segments)
        .map(|index| {
            let angle = PI * index as f32 / segments as f32;
            Vec2::new(half_width * angle.sin(), -half_height * angle.cos())
        })
        .collect()
}

// A dome: a quarter arc from the base circle up to the apex, closed along the
// axis so the profile is a loop.
fn dome_profile(radius: f32, height: f32, segments: usize) -> Vec<Vec2> {
    let mut profile = vec![Vec2::new(0.0, 0.0), Vec2::new(radius, 0.0)];
    for index in 1..segments {
        let angle = FRAC_PI_2 * index as f32 / segments as f32;
        profile.push(Vec2::new(radius * angle.cos(), height * angle.sin()));
    }
    profile.push(Vec2::new(0.0, height));
    profile
}

// A ring with a square section: revolve it and every slice is an annulus.
fn bar_profile(radius: f32, half_section: f32) -> Vec<Vec2> {
    vec![
        Vec2::new(radius - half_section, -half_section),
        Vec2::new(radius + half_section, -half_section),
        Vec2::new(radius + half_section, half_section),
        Vec2::new(radius - half_section, half_section),
    ]
}

// A vase: the side curves in from the base to a waist and back out to the rim,
// then the profile closes along the axis.
fn vase_profile(base: f32, waist: f32, rim: f32, height: f32, segments: usize) -> Vec<Vec2> {
    let mut profile = Vec::new();
    for index in 0..=segments {
        let t = index as f32 / segments as f32;
        let radius = if t < 0.5 {
            let lower = t * 2.0;
            base + (waist - base) * lower * lower
        } else {
            let upper = (t - 0.5) * 2.0;
            waist + (rim - waist) * upper * (2.0 - upper)
        };
        profile.push(Vec2::new(radius, height * t));
    }
    profile.push(Vec2::new(0.0, height));
    profile.push(Vec2::new(0.0, 0.0));
    profile
}

// An outline sampled by direction from `offset`: `radius(angle)` is the distance
// to the boundary. Shapes whose boundary a ray from the centre crosses once (a
// circle, a square, a hexagon, an ellipse) pair vertex by vertex, which is what
// `band` needs for a clean hole.
fn polygon_angle(radius: impl Fn(f32) -> f32, offset: Vec2, count: usize) -> Vec<Vec2> {
    (0..count)
        .map(|index| {
            let angle = TAU * index as f32 / count as f32;
            offset + radius(angle) * Vec2::new(angle.cos(), angle.sin())
        })
        .collect()
}

// Radius of a regular polygon with `sides` sides and circumradius `radius`.
fn polygon_radius(radius: f32, sides: usize) -> impl Fn(f32) -> f32 {
    let step = TAU / sides as f32;
    move |angle: f32| {
        let offset = ((angle % step) + step) % step - step / 2.0;
        radius * (step / 2.0).cos() / offset.cos()
    }
}

// A regular polygon outline with `sides` corners and circumradius `radius`.
fn polygon_outline(radius: f32, sides: usize, phase: f32) -> Vec<Vec2> {
    (0..sides)
        .map(|index| {
            let angle = phase + TAU * index as f32 / sides as f32;
            radius * Vec2::new(angle.cos(), angle.sin())
        })
        .collect()
}

// A rectangle outline with the given half sizes.
fn rectangle_outline(half_x: f32, half_z: f32) -> Vec<Vec2> {
    vec![
        Vec2::new(-half_x, -half_z),
        Vec2::new(half_x, -half_z),
        Vec2::new(half_x, half_z),
        Vec2::new(-half_x, half_z),
    ]
}

// A star outline: `spikes` points trading between the two radii.
fn star_outline(spikes: usize, inner: f32, outer: f32, phase: f32) -> Vec<Vec2> {
    (0..spikes * 2)
        .map(|index| {
            let radius = if index % 2 == 0 { outer } else { inner };
            let angle = phase + TAU * index as f32 / (spikes * 2) as f32;
            radius * Vec2::new(angle.cos(), angle.sin())
        })
        .collect()
}

// An angle bracket: two arms of `arm` and `thickness` wide.
fn bracket_outline(arm: f32, thickness: f32) -> Vec<Vec2> {
    let half = arm / 2.0;
    vec![
        Vec2::new(-half, -half),
        Vec2::new(half, -half),
        Vec2::new(half, -half + thickness),
        Vec2::new(-half + thickness, -half + thickness),
        Vec2::new(-half + thickness, half),
        Vec2::new(-half, half),
    ]
}

// A staircase: `steps` treads of `tread` by `rise`, climbing to the left.
fn stair_outline(steps: usize, tread: f32, rise: f32) -> Vec<Vec2> {
    let (half_width, half_depth) = (
        tread * steps as f32 / 2.0,
        rise * steps as f32 / 2.0,
    );
    let mut outline = vec![
        Vec2::new(-half_width, -half_depth),
        Vec2::new(half_width, -half_depth),
    ];
    for step in 1..=steps {
        let height = -half_depth + rise * step as f32;
        outline.push(Vec2::new(half_width - tread * (step - 1) as f32, height));
        outline.push(Vec2::new(half_width - tread * step as f32, height));
    }
    outline
}

// A plus: arms of `arm` reach with `thickness` wide limbs.
fn cross_outline(arm: f32, thickness: f32) -> Vec<Vec2> {
    let (reach, limb) = (arm / 2.0, thickness / 2.0);
    vec![
        Vec2::new(-limb, -reach),
        Vec2::new(limb, -reach),
        Vec2::new(limb, -limb),
        Vec2::new(reach, -limb),
        Vec2::new(reach, limb),
        Vec2::new(limb, limb),
        Vec2::new(limb, reach),
        Vec2::new(-limb, reach),
        Vec2::new(-limb, limb),
        Vec2::new(-reach, limb),
        Vec2::new(-reach, -limb),
        Vec2::new(-limb, -limb),
    ]
}

// A ring sector: the outer arc, one end, the inner arc and the other end, so it
// is a single simple outline (a whole ring would not be).
fn sector_outline(inner: f32, outer: f32, turns: f32, segments: usize) -> Vec<Vec2> {
    let sweep = TAU * turns;
    let mut outline = Vec::new();
    for index in 0..=segments {
        let angle = sweep * index as f32 / segments as f32;
        outline.push(outer * Vec2::new(angle.cos(), angle.sin()));
    }
    for index in (0..=segments).rev() {
        let angle = sweep * index as f32 / segments as f32;
        outline.push(inner * Vec2::new(angle.cos(), angle.sin()));
    }
    outline
}

// The outline shifted, scaled or turned in the slice plane, which is how the
// lofted tops of the tapered, sheared and twisted shapes are built.
fn shifted(outline: &[Vec2], offset: Vec2) -> Vec<Vec2> {
    outline.iter().map(|point| *point + offset).collect()
}

fn turned(outline: &[Vec2], angle: f32) -> Vec<Vec2> {
    let (sin, cos) = angle.sin_cos();
    outline
        .iter()
        .map(|point| Vec2::new(point.x * cos - point.y * sin, point.x * sin + point.y * cos))
        .collect()
}

// The lowest y a mesh reaches under `rotation`, so every shape - however it is
// tilted - can be rested on the bed (y = 0) instead of guessing its height.
fn lowest(meshes: &Assets<Mesh>, mesh: &Handle<Mesh>, rotation: Quat) -> f32 {
    let Some(points) = meshes
        .get(mesh)
        .and_then(|mesh| mesh.try_attribute(Mesh::ATTRIBUTE_POSITION).ok())
        .and_then(|values| values.as_float3())
    else {
        return 0.0;
    };
    points
        .iter()
        .map(|point| (rotation * Vec3::from(*point)).y)
        .reduce(f32::min)
        .unwrap_or(0.0)
}

// One fallback shape: its mesh, where it sits on the bed and the angle it is
// sliced at. The angle lives on the entity transform, which is the matrix the
// slicer maps the mesh through, so a rotation here becomes a rotated cross
// section in the preview.
fn shape(
    meshes: &Assets<Mesh>,
    material: Handle<StandardMaterial>,
    name: &str,
    mesh: Handle<Mesh>,
    rotation: Quat,
    (x, z): (f32, f32),
) -> impl Scene {
    let name = name.to_string();
    let translation = Vec3::new(x, -lowest(meshes, &mesh, rotation), z);
    bsn! {
        Name(name)
        Mesh3d(mesh)
        MeshMaterial3d::<StandardMaterial>(material)
        Transform { rotation: rotation, translation: translation }
    }
}

// The rig is a plain grid on the bed: columns run along X, rows along Z, and the
// rows are ordered by height so the tallest shapes stand nearest the default
// camera. The far corner - where a tall shape would fall outside the view - is
// left empty.
// The centre of the grid cell at the given column (X) and row (Z).
fn spot(column: usize, row: usize) -> (f32, f32) {
    (
        -12.0 + column as f32 * 5.0,
        -12.0 + row as f32 * 5.0,
    )
}

// Fallback demo shapes shown when no models were loaded.
//
// The set is a slice-preview test rig: pressing `S` slices every mesh and draws
// the toolpath, so the shapes below walk through the cases the slicer has to get
// right - concave profiles, several holes in one layer, curved surfaces, and
// cross sections that twist, taper or shear with height.
//
// All of them are laid out on one square grid (`grid_spot`): five columns by five
// rows, one shape per cell, tallest nearest the camera and the far cells left to
// the low shapes so nothing tall falls outside the view.
pub fn demo(
    meshes: &mut Assets<Mesh>,
    material: Handle<StandardMaterial>,
) -> impl SceneList {
    // irregular profiles, lofted into prisms
    let square = polygon_outline(1.35, 4, FRAC_PI_4);
    let bracket = meshes.add(loft(
        &bracket_outline(2.3, 0.95),
        &bracket_outline(2.3, 0.95),
        2.6,
    ));
    let star = meshes.add(loft(
        &star_outline(5, 0.98, 1.6, 0.0),
        &star_outline(5, 0.98, 1.6, 0.0),
        2.6,
    ));
    let stair = meshes.add(loft(
        &stair_outline(3, 0.92, 0.66),
        &stair_outline(3, 0.92, 0.66),
        2.6,
    ));
    let cross = meshes.add(loft(
        &cross_outline(2.4, 1.05),
        &cross_outline(2.4, 1.05),
        2.4,
    ));
    // A twisted square: the cross section turns 40° from bottom to top.
    let twisted = meshes.add(loft(&square, &turned(&square, 0.7), 3.4));

    // curved surfaces------------------------
    let ellipsoid = meshes.add(revolve(&ellipse_profile(1.5, 1.15, 24), 40));
    let dome = meshes.add(revolve(&dome_profile(1.7, 1.6, 24), 40));
    // A ring whose section is square, so the curve is in the sweep.
    let bar = meshes.add(revolve(&bar_profile(1.35, 0.45), 40));
    let capsule = meshes.add(Capsule3d::new(0.7, 1.2));

    // one hole, round or off centre----------
    let disc = |radius: f32| polygon_angle(move |_| radius, Vec2::ZERO, 32);
    let hole = |radius: f32, offset: Vec2| polygon_angle(move |_| radius, offset, 32);
    let pipe = meshes.add(band(&disc(1.5), &disc(0.6), 2.8));
    let eccentric = meshes.add(band(&disc(1.8), &hole(0.55, Vec2::new(0.4, 0.0)), 1.3));
    let hex_plate = meshes.add(band(
        &polygon_angle(polygon_radius(1.8, 6), Vec2::ZERO, 32),
        &hole(0.5, Vec2::new(0.25, 0.15)),
        1.3,
    ));
    let half_ring = sector_outline(0.7, 1.6, 0.5, 24);
    let horseshoe = meshes.add(loft(&half_ring, &half_ring, 1.2));
    let torus = meshes.add(Torus::new(0.5, 1.5));

    // several holes in one layer, and a ring that nests an island --------
    let three_holes = meshes.add(plate(
        &rectangle_outline(2.2, 1.0),
        &[
            hole(0.3, Vec2::new(-1.5, 0.0)),
            hole(0.3, Vec2::ZERO),
            hole(0.3, Vec2::new(1.5, 0.0)),
        ],
        1.2,
    ));
    // A window: four square holes, so every layer carries five loops.
    let window = meshes.add(plate(
        &rectangle_outline(1.85, 1.85),
        &[
            shifted(&rectangle_outline(0.3, 0.3), Vec2::new(-0.75, -0.75)),
            shifted(&rectangle_outline(0.3, 0.3), Vec2::new(0.75, -0.75)),
            shifted(&rectangle_outline(0.3, 0.3), Vec2::new(-0.75, 0.75)),
            shifted(&rectangle_outline(0.3, 0.3), Vec2::new(0.75, 0.75)),
        ],
        1.2,
    ));
    // Nested ring: an outer ring with an island floating in its hole.
    let bullseye = meshes.add(combined(&[
        band(&disc(1.65), &disc(0.8), 1.2),
        loft(&disc(0.45), &disc(0.45), 1.2),
    ]));
    // A double ring: two concentric rings in one piece, so a layer that cuts it
    // carries four loops.
    let double_ring = meshes.add(combined(&[
        band(&disc(2.2), &disc(1.35), 1.2),
        band(&disc(1.2), &disc(0.35), 1.2),
    ]));
    // A quarter ring: a quarter turn of an annulus, used at two angles.
    let quarter = sector_outline(0.7, 1.6, 0.25, 14);
    let quarter_ring = meshes.add(loft(&quarter, &quarter, 1.2));
    // A turned vase: the side narrows to a waist and flares out again.
    let vase = meshes.add(revolve(&vase_profile(1.7, 0.95, 1.25, 2.2, 20), 40));
    // A hexagonal nut and a triangular plate, each with one hole.
    let hex_nut = meshes.add(band(
        &polygon_angle(polygon_radius(1.9, 6), Vec2::ZERO, 32),
        &polygon_angle(polygon_radius(0.9, 6), Vec2::ZERO, 32),
        1.2,
    ));
    let tri_plate = meshes.add(band(
        &polygon_angle(polygon_radius(2.8, 3), Vec2::ZERO, 30),
        &polygon_angle(|_| 0.55, Vec2::ZERO, 30),
        1.2,
    ));

    // A lofted solid lies in the slice plane, so turning one -90° about X stands
    // its outline up as a vertical section, while a turn about Y only spins it
    // inside the plane.
    let upright = Quat::from_rotation_x(-FRAC_PI_2);
    let spin = Quat::from_rotation_y(FRAC_PI_6);
    let slant = Quat::from_rotation_x(FRAC_PI_4);
    let roll = Quat::from_rotation_z(FRAC_PI_2);

    bsn_list![
        // Near rows: the tall shapes, each carrying an angle.
        shape(meshes, material.clone(), "bracket / 45° about X, 30° about Y", bracket.clone(), spin * slant, spot(4, 4)),
        shape(meshes, material.clone(), "cross / 35° about X, 45° about Y", cross.clone(), Quat::from_rotation_y(FRAC_PI_4) * Quat::from_rotation_x(0.61), spot(3, 4)),
        shape(meshes, material.clone(), "staircase / 30° about Y", stair.clone(), spin, spot(2, 4)),
        shape(meshes, material.clone(), "quarter ring / standing, 30° about Y", quarter_ring.clone(), spin * upright, spot(1, 4)),
        shape(meshes, material.clone(), "nested ring / island in the hole", bullseye.clone(), Quat::IDENTITY, spot(0, 4)),

        shape(meshes, material.clone(), "twisted prism / 40° twist over 3.4", twisted.clone(), Quat::IDENTITY, spot(4, 3)),
        shape(meshes, material.clone(), "star prism / 5 spikes", star.clone(), Quat::IDENTITY, spot(3, 3)),
        shape(meshes, material.clone(), "capsule / lying on its side", capsule.clone(), roll, spot(2, 3)),
        shape(meshes, material.clone(), "window / four square holes", window.clone(), Quat::IDENTITY, spot(1, 3)),
        shape(meshes, material.clone(), "square-section ring / revolved", bar.clone(), Quat::IDENTITY, spot(0, 3)),

        // Middle row: one hole each, round or off centre.
        shape(meshes, material.clone(), "pipe / thin wall, 2.8 tall", pipe.clone(), Quat::IDENTITY, spot(4, 2)),
        shape(meshes, material.clone(), "dome / revolved half sphere", dome.clone(), Quat::IDENTITY, spot(3, 2)),
        shape(meshes, material.clone(), "three holes / one outline, 90° about Y", three_holes.clone(), Quat::from_rotation_y(FRAC_PI_2), spot(2, 2)),
        shape(meshes, material.clone(), "torus / flat, two loops", torus.clone(), Quat::IDENTITY, spot(1, 2)),
        shape(meshes, material.clone(), "eccentric ring / hole off centre", eccentric.clone(), Quat::IDENTITY, spot(0, 2)),

        // Far rows: curved surfaces and the rest of the rings.
        shape(meshes, material.clone(), "ellipsoid / 3.0 x 2.3", ellipsoid.clone(), Quat::IDENTITY, spot(4, 1)),
        shape(meshes, material.clone(), "torus / 60° about X, 30° about Y", torus.clone(), spin * Quat::from_rotation_x(1.0), spot(3, 1)),
        shape(meshes, material.clone(), "hex plate / round hole off centre", hex_plate.clone(), Quat::from_rotation_y(0.26), spot(2, 1)),
        shape(meshes, material.clone(), "quarter ring / flat, 45° about Y", quarter_ring.clone(), Quat::from_rotation_y(FRAC_PI_4), spot(1, 1)),

        shape(meshes, material.clone(), "torus / 45° about X", torus.clone(), slant, spot(4, 0)),
        shape(meshes, material.clone(), "double ring / four loops a layer", double_ring.clone(), Quat::IDENTITY, spot(3, 0)),
        shape(meshes, material.clone(), "half ring / flat horseshoe", horseshoe.clone(), Quat::IDENTITY, spot(2, 0)),

        // The far cells: nothing tall may stand there.
        shape(meshes, material.clone(), "vase / revolved with a waist", vase.clone(), Quat::IDENTITY, spot(0, 1)),
        shape(meshes, material.clone(), "hex nut / hexagonal hole", hex_nut.clone(), Quat::IDENTITY, spot(0, 0)),
        shape(meshes, material.clone(), "triangle plate / round hole", tri_plate.clone(), Quat::from_rotation_y(0.3), spot(1, 0)),
    ]
}
