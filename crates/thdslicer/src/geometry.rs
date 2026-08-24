
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

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
        self.partial_cmp(&other).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for Cov {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.diff(other) < Self::epow() {
            Some(Ordering::Equal)
        } else {
            self.0.partial_cmp(&other.0)
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
    fn eq(&self, other: &Self) -> bool {
        Cov(self.x) == Cov(other.x) && Cov(self.z) == Cov(other.z)
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
            if t >= 0.0 && t <= 1.0 && u >= 0.0 && u <= 1.0 {
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

        // If signed area > 0, vertices are clockwise; otherwise counter-clockwise
        // For clockwise polygons, inward offset needs positive distance
        // For counter-clockwise polygons, inward offset needs negative distance
        let actual = if area > 0.0 { -distance } else { distance };

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
                let collinear = {
                    let cross = vec1.cross(vec2).abs() / (len1 * len2);
                    cross < epsilon
                };
                
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
    pub fn intersects(&self, segment: &Segment2D) -> Vec<(f32, Point2D)> {
        let mut points = Vec::new();
        let n = self.points.len();
        let epsilon = Cov::<N6>::epow();

        for i in 0..n {
            let j = (i + 1) % n;
            let p1 = self.points[i];
            let p2 = self.points[j];
            let edge = Segment2D::new(p1, p2);

            if let Some(point) = segment.intersect(&edge) {
                // Calculate parameter t along the segment
                let dx = segment.end.x - segment.start.x;
                let dz = segment.end.z - segment.start.z;
                let length = dx * dx + dz * dz;

                let t = if length > epsilon {
                    ((point.x - segment.start.x) * dx + (point.z - segment.start.z) * dz) / length
                } else {
                    0.0
                };

                // Clamp t to [0, 1] range
                let clamped = t.max(0.0).min(1.0);

                points.push((clamped, point));
            }
        }

        points
    }
}
