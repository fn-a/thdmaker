
use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::{Hash, Hasher, BuildHasherDefault};

// /// 10e-5, 0.00001
// pub const N5:u32 = 0x3727C5AC;
// /// 10e-4, 0.0001
// pub const N4:u32 = 0x38D1B717;
// /// 10e-3, 0.001
// pub const N3:u32 = 0x3A83126F;
// /// 10e-2, 0.01
// pub const N2:u32 = 0x3C23D70A;
// /// 10e-1, 0.1
// pub const N1:u32 = 0x3DCCCCCD;
// /// 10e1, 10
// pub const E1:u32 = 0x41200000;
// /// 10e2, 100
// pub const E2:u32 = 0x42C80000;
// /// 10e3, 1000
// pub const E3:u32 = 0x447A0000;
// /// 10e5, 100000
// pub const E5:u32 = 0x47C35000;
// /// 10e6, 1000000
// pub const E6:u32 = 0x49742400;

/// 10e-6, 0.000001
pub const N6:u32 = 0x358637BD;
/// 10e4, 10000
pub const E4:u32 = 0x461C4000;

/// A coordinate wrapper value use for floating point comparisons
#[derive(Debug, Clone, Copy)]
pub struct Cov<const E: u32 = N6>(pub f32);

impl<const E: u32> Cov<E> {
    /// Calculate the difference between two value
    fn diff(&self, other: &Self) -> f32 {
        (self.0 - other.0).abs()
    }

    /// Epsilon value for floating value comparisons
    pub fn epow() -> f32 {
        f32::from_bits(E)
    }

    /// Quantize the value to an integer hash key
    pub fn hash(&self) -> i64 {
        (self.0 * Self::epow()).round() as i64
    }
}

impl From<f32> for Cov {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl Eq for Cov {}

impl PartialEq for Cov {
    fn eq(&self, other: &Self) -> bool {
        self.diff(other) < Self::epow()
    }
}

impl Ord for Cov {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.diff(other) < Self::epow() {
            Ordering::Equal
        } else {
            self.0.total_cmp(&other.0)
        }
    }
}

impl PartialOrd for Cov {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A [`HashMap`] keyed with [`Cov`].
#[allow(unused)]
pub type CovMap<K, V> = HashMap<K, V, BuildHasherDefault<Cov>>;

/// A cheap hasher for the maps the stitching uses.
///
/// The keys are the coordinates of the crossings - two or four small integers -
/// and the default hasher is built to withstand keys an attacker chooses, which
/// these are not. Sketching the bytes of each key instead costs a visible share
/// of the time a layer takes to stitch, and nothing about the output depends on
/// the hash values themselves.
#[derive(Default)]
pub struct Qik(u64);

impl Hasher for Qik {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 = (self.0 ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3);
        }
    }

    #[inline]
    fn write_i32(&mut self, value: i32) {
        self.write_i64(i64::from(value));
    }

    #[inline]
    fn write_i64(&mut self, value: i64) {
        self.0 = (self.0 ^ value as u64).wrapping_mul(0x517c_c1b7_2722_0a95);
        self.0 ^= self.0 >> 29;
    }
}

/// A [`HashMap`] keyed with [`Qik`].
#[allow(unused)]
pub type QikMap<K, V> = HashMap<K, V, BuildHasherDefault<Qik>>;

impl Qik {
    /// The point that stands for `point` in the stitched contours: an already seen
    /// point within a cell of the lookup grid, or `point` itself.
    ///
    /// The ends are welded because the two triangles that met along an edge
    /// interpolate their shared crossing from opposite ends, so their results agree
    /// only to within rounding. Left alone they can land in different cells of the
    /// lookup grid, the pair is then never joined, the loop stays open and every
    /// segment of the contour it belongs to is thrown away.
    pub fn weld(welded: &mut QikMap<Point2D, Point2D>, point: Point2D) -> Point2D {
        // The common case: another end already claimed this cell.
        if let Some(found) = welded.get(&point) {
            *found
        } else {
            // One cell of the grid `Point2D` hashes and compares on. A point that landed
            // a cell over from the end it has to meet is found by looking at the eight
            // cells around this one.
            let step = 1.0 / Cov::<E4>::epow();
            let mut nearest: Option<Point2D> = None;
            let mut closest = step;
            for dx in [-step, 0.0, step] {
                for dz in [-step, 0.0, step] {
                    if let Some(found) = welded.get(&Point2D::new(point.x + dx, point.z + dz)) {
                        let apart = (found.x - point.x).abs().max((found.z - point.z).abs());
                        if apart <= closest {
                            closest = apart;
                            nearest = Some(*found);
                        }
                    }
                }
            }
            match nearest {
                Some(found) => found,
                None => {
                    welded.insert(point, point);
                    point
                }
            }
        }
    }
}

/// Represents a 2D point in the XZ slice plane (Bevy Y-axis up)
#[derive(Debug, Clone, Copy)]
pub struct Point2D {
    pub x: f32,
    pub z: f32,
}

impl Point2D {
    /// Create a new 2D point in XZ plane
    pub fn new(x: f32, z: f32) -> Self {
        Self { x, z }
    }

    /// Check if a point is inside a polygon using ray casting
    pub fn inside(self, polygon: &Polygon2D) -> bool {
        let mut inside = false;
        let n = polygon.points.len();
        if n >= 3 {
            for i in 0..n {
                let j = (i + 1) % n;
                let pi = polygon.points[i];
                let pj = polygon.points[j];
                let pv = (pj.x - pi.x) * (self.z - pi.z) / (pj.z - pi.z) + pi.x;
                
                if ((pi.z > self.z) != (pj.z > self.z)) && (self.x < pv){
                    inside = !inside;
                }
            }
        }
        inside
    }

    /// Calculate the length of the vector
    pub fn length(&self) -> f32 {
        self.x.hypot(self.z)
    }

    /// Normalize the vector
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > Cov::<N6>::epow() {
            Self {
                x: self.x / len,
                z: self.z / len,
            }
        } else {
            Self {
                x: 0.0,
                z: 0.0,
            }
        }
    }

    /// Calculate the cross product of two vectors
    pub fn cross(&self, other: Self) -> f32 {
        self.x * other.z - self.z * other.x
    }

    /// Calculate the dot product of two vectors
    pub fn dot(&self, other: Self) -> f32 {
        self.x * other.x + self.z * other.z
    }

    /// Subtract another vector from this vector
    pub fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            z: self.z - other.z,
        }
    }

    /// Add another vector to this vector
    pub fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            z: self.z + other.z,
        }
    }

    /// Multiply the vector by a scalar
    pub fn mul(&self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            z: self.z * scalar,
        }
    }
}

impl Hash for Point2D {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Cov::<E4>(self.x).hash().hash(state);
        Cov::<E4>(self.z).hash().hash(state);
    }
}

impl Eq for Point2D {}

impl PartialEq for Point2D {
    // Equality is defined on the same quantization grid as Hash, so that
    // equal points always land in the same hash bucket.
    fn eq(&self, other: &Self) -> bool {
        let xeq = Cov::<E4>(self.x).hash() == Cov::<E4>(other.x).hash();
        let zeq = Cov::<E4>(self.z).hash() == Cov::<E4>(other.z).hash();
        xeq && zeq
    }
}

/// Represents a 2D line segment
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment2D {
    pub start: Point2D,
    pub end: Point2D,
}

impl Segment2D {
    /// Create a new line segment from two points
    pub fn new(start: Point2D, end: Point2D) -> Self {
        Self { start, end }
    }

    /// Calculate the length of the line segment
    pub fn length(&self) -> f32 {
        let dx = self.end.x - self.start.x;
        let dz = self.end.z - self.start.z;
        (dx * dx + dz * dz).sqrt()
    }

    /// Check if a line segment is inside any of the contours
    pub fn inside(&self, contours: &[Polygon2D]) -> bool {
        let midpoint = Point2D::new(
            (self.start.x + self.end.x) / 2.0,
            (self.start.z + self.end.z) / 2.0,
        );

        for contour in contours {
            if midpoint.inside(contour) {
                return true;
            }
        }

        false
    }

    /// Calculate the intersection point of two line segments
    /// Returns None if segments are parallel or do not intersect
    pub fn intersect(&self, other: &Segment2D) -> Option<Point2D> {
        let x1 = self.start.x;
        let z1 = self.start.z;
        let x2 = self.end.x;
        let z2 = self.end.z;
        let x3 = other.start.x;
        let z3 = other.start.z;
        let x4 = other.end.x;
        let z4 = other.end.z;

        let denom = (x1 - x2) * (z3 - z4) - (z1 - z2) * (x3 - x4);

        // Check if lines are parallel
        if denom.abs() < Cov::<N6>::epow() {
            None
        } else {
            let t = ((x1 - x3) * (z3 - z4) - (z1 - z3) * (x3 - x4)) / denom;
            let u = -((x1 - x2) * (z1 - z3) - (z1 - z2) * (x1 - x3)) / denom;
    
            // Check if intersection point is within both segments
            if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
                let x = x1 + t * (x2 - x1);
                let z = z1 + t * (z2 - z1);
                Some(Point2D::new(x, z))
            } else {
                None
            }
        }
    }
}

/// Represents a 2D polygon (contour)
#[derive(Debug, Clone)]
pub struct Polygon2D {
    pub points: Vec<Point2D>,
}

impl Polygon2D {
    /// Create a new empty polygon
    pub fn new() -> Self {
        Self { points: Vec::new() }
    }

    /// Add a point to the polygon
    pub fn push(&mut self, point: Point2D) {
        self.points.push(point);
    }

    /// Check if the polygon is closed (first and last points are the same or very close)
    pub fn closed(&self) -> bool {
        if self.points.is_empty() {
            false
        } else {
            let i = self.points.len() - 1;
            let x = Cov(self.points[0].x) == Cov(self.points[i].x);
            let z = Cov(self.points[0].z) == Cov(self.points[i].z);
            x && z
        }
    }

    /// Calculate the area of the polygon
    pub fn area(&self) -> f32 {
        if self.points.len() < 3 {
            0.0
        } else {
            let mut area = 0.0;
            let n = self.points.len();
            for i in 0..n {
                let j = (i + 1) % n;
                area += self.points[i].x * self.points[j].z;
                area -= self.points[j].x * self.points[i].z;
            }
            area.abs() / 2.0
        }

    }

    /// Calculate the centroid (center of mass) of the polygon
    pub fn centroid(&self) -> Option<Point2D> {
        if self.points.len() < 3 {
            None
        } else {
            let mut cx = 0.0;
            let mut cz = 0.0;
            let n = self.points.len();
    
            for point in &self.points {
                cx += point.x;
                cz += point.z;
            }
    
            Some(Point2D::new(cx / n as f32, cz / n as f32))
        }
    }

    /// Calculate the bounding box of the polygon
    pub fn bounding(&self) -> Option<(Point2D, Point2D)> {
        if !self.points.is_empty() {
            let mut min_x = self.points[0].x;
            let mut max_x = self.points[0].x;
            let mut min_z = self.points[0].z;
            let mut max_z = self.points[0].z;
    
            for point in &self.points {
                min_x = min_x.min(point.x);
                max_x = max_x.max(point.x);
                min_z = min_z.min(point.z);
                max_z = max_z.max(point.z);
            }
    
            Some((
                Point2D::new(min_x, min_z),
                Point2D::new(max_x, max_z),
            ))
        } else {
            None
        }
    }

    /// Offset a polygon inward by a given distance
    /// Less than zero will expand the polygon
    pub fn offset(&self, distance: f32) -> Polygon2D {
        let mut polygon = Polygon2D::new();
        let n = self.points.len();

        // Calculate signed area to determine vertex winding order
        let mut area = 0.0;
        for i in 0..n {
            let j = (i + 1) % n;
            area += (self.points[i].x * self.points[j].z) - (self.points[j].x * self.points[i].z);
        }

        // A positive area means the outline runs counter-clockwise in (x, z), so
        // the left-hand normal `(-z, x)` built below points into the polygon and
        // the inward offset follows it; a clockwise outline turns that normal
        // outwards and the offset has to oppose it. Either way a positive distance
        // moves the boundary towards the inside of the polygon, which is what the
        // callers rely on: line-width pulls a shell inwards, - line-width
        // pushes the wall of a hole outwards into the material around it.
        let actual = if area > 0.0 { distance } else { -distance };

        let epsilon = Cov::<N6>::epow();

        for i in 0..n {
            let prev = self.points[(i + n - 1) % n];
            let curr = self.points[i];
            let next = self.points[(i + 1) % n];

            // Build edge vectors
            let vec1 = curr.sub(prev);
            let vec2 = next.sub(curr);

            // Calculate unit normal vectors (left-hand rule)
            let nrm1 = Point2D::new(-vec1.z, vec1.x).normalize();
            let nrm2 = Point2D::new(-vec2.z, vec2.x).normalize();

            // Check if three points are collinear using cross product
            let len1 = vec1.length();
            let len2 = vec2.length();

            if len1 > epsilon && len2 > epsilon {
                // How far `curr` sits from the straight line through its two
                // neighbours. A vertex on that line is not a corner: it carries no
                // offset point of its own, and the edge it lies on is already
                // covered by the corners at its ends. Measuring a distance rather
                // than a sine keeps the test on the same scale as the coordinates,
                // so a vertex that only rounding noise knocked off the line is
                // skipped as reliably as an exactly straight one - a sine test lets
                // those through, and their offset points then stick out of the
                // loop they were meant to shrink.
                let spanlen = next.sub(prev).length();
                let collinear = spanlen <= epsilon || vec1.cross(vec2).abs() / spanlen < epsilon;

                // Collinear points skip offsetting
                if !collinear {
                    // For corners, calculate the intersection of two offset edges
                    // Edge 1: passes through prev, direction is vec1
                    // Edge 2: passes through curr, direction is vec2
                    // Offset edges: offset by distance d along normals
                    // Calculate intersection using Cramer's rule
                    // Line 1: P1 + t * v1, where P1 = prev + nrm1 * d, v1 = vec1
                    // Line 2: P2 + u * v2, where P2 = curr + nrm2 * d, v2 = vec2
    
                    let p1 = prev.add(nrm1.mul(actual));
                    let p2 = curr.add(nrm2.mul(actual));
    
                    // Solve: p1 + t * v1 = p2 + u * v2
                    // => t * v1 - u * v2 = p2 - p1
                    // => [v1.x  -v2.x] [t] = [p2.x - p1.x]
                    //    [v1.z  -v2.z] [u]   [p2.z - p1.z]
    
                    let det = vec1.x * (-vec2.z) - vec1.z * (-vec2.x);
    
                    let point = if det.abs() < epsilon {
                        // Lines are parallel, use midpoint
                        p1.add(p2).mul(0.5)
                    } else {
                        // Cramer's rule
                        let dx = p2.x - p1.x;
                        let dz = p2.z - p1.z;
    
                        let t = (dx * (-vec2.z) - dz * (-vec2.x)) / det;
    
                        // Intersection point
                        p1.add(vec1.mul(t))
                    };
        
                    polygon.push(point);
                }
            }
        }

        polygon
    }

    /// Check if this polygon is completely inside another polygon
    /// Returns true if all vertices of this polygon are inside the given polygon
    pub fn inside(&self, other: &Polygon2D) -> bool {
        // Empty polygon cannot be inside another polygon
        if self.points.is_empty() {
            false
        } else {
            // Check if all vertices of this polygon are inside the other polygon
            for point in &self.points {
                if !point.inside(other) {
                    return false;
                }
            }
            true
        }
    }

    /// Count intersections of a ray from point (x, z) in positive x direction with this polygon
    /// Used in ray casting algorithm to determine if a point is inside/outside
    pub fn raycount(&self, point: Point2D) -> usize {
        let mut count = 0;
        let n = self.points.len();

        for i in 0..n {
            let j = (i + 1) % n;
            let p1 = self.points[i];
            let p2 = self.points[j];

            // Check if ray (point.x, point.z) -> (+inf, point.z) intersects segment p1-p2
            // Ray is horizontal: y = point.z, x >= point.x

            // Check if segment straddles the ray's y-coordinate
            let z1 = p1.z;
            let z2 = p2.z;
            let zp = point.z;

            // Skip if both endpoints are on same side of ray
            if (z1 > zp) != (z2 > zp) {
                // Calculate x-coordinate of intersection
                // Line equation: x = x1 + (y - y1) * (x2 - x1) / (y2 - y1)
                let x_intersect = p1.x + (zp - z1) * (p2.x - p1.x) / (z2 - z1);
    
                // Check if intersection is to the right of the ray origin
                if Cov(x_intersect) >= Cov(point.x) {
                    count += 1;
                }
            }
        }

        count
    }

    /// Find all intersection points of a segment with this polygon
    /// Returns a vector of intersection points with their parameter values (0.0 to 1.0)
    /// The parameter represents the position along the segment from start (0.0) to end (1.0)
    ///
    /// A polygon vertex that lies exactly on the segment's line is resolved by the
    /// half-open rule `(side1 >= 0.0) != (side2 >= 0.0)`, which counts every
    /// crossing between the two half planes and nothing else: a line passing
    /// through a vertex crosses the outline once, a line that only touches it -
    /// both neighbours on the same side, the infill line tangent to a circle -
    /// crosses zero or two times, and an infill line running along an edge of the
    /// layer keeps that edge as a printed span. Either way the crossings stay paired,
    /// which is what lets Layer delineate read them as alternating outside/inside spans.
    pub fn intersects(&self, segment: &Segment2D) -> Vec<(f32, Point2D)> {
        let mut points = Vec::new();
        let n = self.points.len();

        // The segment as a unit vector, so the tests below measure a real distance
        // from the line and the parameter along the segment is a length.
        let dx = segment.end.x - segment.start.x;
        let dz = segment.end.z - segment.start.z;
        let length = dx.hypot(dz);
        if n >= 3 && length > Cov::<N6>::epow() {
            let dirx = dx / length;
            let dirz = dz / length;
            for i in 0..n {
                let p1 = self.points[i];
                let p2 = self.points[(i + 1) % n];
                // Signed distance of both ends of the edge from the segment's line.
                let one = dirx * (p1.z - segment.start.z) - dirz * (p1.x - segment.start.x);
                let two = dirx * (p2.z - segment.start.z) - dirz * (p2.x - segment.start.x);

                if (one >= 0.0) != (two >= 0.0) {
                    let along = one / (one - two);
                    let point = Point2D::new(
                        p1.x + along * (p2.x - p1.x),
                        p1.z + along * (p2.z - p1.z),
                    );
                    // Parameter along the segment, clamped the way the crossings of an
                    // infill line have to be ordered.
                    let position = (dirx * (point.x - segment.start.x) + dirz * (point.z - segment.start.z)) / length;
                    points.push((position.clamp(0.0, 1.0), point));
                }
            }
        }

        points
    }
}
