use std::f32::consts;
use super::Config;
use super::geometry::*;

/// Dense infill pattern types
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DenseInfill {
    Grid,
    Line,
}

impl DenseInfill {
    /// Generate grid infill bounding box
    pub fn grid((min, max): &(Point2D, Point2D), config: &Config) -> Vec<Segment2D> {
        let min_x = min.x; let min_z = min.z;
        let max_x = max.x; let max_z = max.z;
        let spacing = config.line_width;
        let mut infills = Vec::new();

        // Lines along Z direction
        let mut z = min_z;
        while z <= max_z {
            let start = Point2D::new(min_x, z);
            let end = Point2D::new(max_x, z);
            let line = Segment2D::new(start, end);

            infills.push(line);

            z += spacing;
        }

        // Lines along X direction
        let mut x = min_x;
        while x <= max_x {
            let start = Point2D::new(x, min_z);
            let end = Point2D::new(x, max_z);
            let line = Segment2D::new(start, end);

            infills.push(line);

            x += spacing;
        }

        infills
    }

    /// Generate line infill bounding box
    pub fn line((min, max): &(Point2D, Point2D), config: &Config) -> Vec<Segment2D> {
        let min_x = min.x; let min_z = min.z;
        let max_x = max.x; let max_z = max.z;
        let spacing = config.line_width;
        let mut infills = Vec::new();

        let mut z = min_z;
        while z <= max_z {
            let start = Point2D::new(min_x, z);
            let end = Point2D::new(max_x, z);
            let line = Segment2D::new(start, end);

            infills.push(line);

            z += spacing;
        }

        infills
    }
}

/// Sparse infill pattern types
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SparseInfill {
    Grid,
    Line,
    Concentric,
    Honeycomb,
}

impl SparseInfill {
    /// Generate grid infill bounding box
    pub fn grid((min, max): &(Point2D, Point2D), config: &Config) -> Vec<Segment2D> {
        let min_x = min.x; let min_z = min.z;
        let max_x = max.x; let max_z = max.z;
        let spacing = config.line_width / config.infill_density;
        let mut infills = Vec::new();

        // Lines along Z direction
        let mut z = min_z;
        while z <= max_z {
            let start = Point2D::new(min_x, z);
            let end = Point2D::new(max_x, z);
            let line = Segment2D::new(start, end);

            infills.push(line);

            z += spacing;
        }

        // Lines along X direction
        let mut x = min_x;
        while x <= max_x {
            let start = Point2D::new(x, min_z);
            let end = Point2D::new(x, max_z);
            let line = Segment2D::new(start, end);

            infills.push(line);

            x += spacing;
        }

        infills
    }

    /// Generate line infill bounding box
    pub fn line((min, max): &(Point2D, Point2D), config: &Config) -> Vec<Segment2D> {
        let min_x = min.x; let min_z = min.z;
        let max_x = max.x; let max_z = max.z;
        let spacing = config.line_width / config.infill_density;
        let mut infills = Vec::new();

        let mut z = min_z;
        while z <= max_z {
            let start = Point2D::new(min_x, z);
            let end = Point2D::new(max_x, z);
            let line = Segment2D::new(start, end);

            infills.push(line);

            z += spacing;
        }

        infills
    }

    /// Generate honeycomb infill bounding box
    pub fn honeycomb((min, max): &(Point2D, Point2D), config: &Config) -> Vec<Segment2D> {
        let min_x = min.x; let min_z = min.z;
        let max_x = max.x; let max_z = max.z;
        let spacing = config.line_width / config.infill_density;
        let hex_size = spacing * 2.0;
        let mut infills = Vec::new();

        let mut row = 0;
        let mut z = min_z;
        while z <= max_z {
            let offset = if row % 2 == 0 { 0.0 } else { hex_size / 2.0 };
            let mut x = min_x + offset;

            while x <= max_x {
                // Create hexagon pattern
                let cx = x;
                let cz = z;

                for i in 0..6 {
                    let angle1 = consts::PI / 3.0 * i as f32;
                    let angle2 = consts::PI / 3.0 * (i + 1) as f32;

                    let start = Point2D::new(
                        cx + hex_size / 2.0 * angle1.cos(),
                        cz + hex_size / 2.0 * angle1.sin(),
                    );
                    let end = Point2D::new(
                        cx + hex_size / 2.0 * angle2.cos(),
                        cz + hex_size / 2.0 * angle2.sin(),
                    );
                    let line = Segment2D::new(start, end);

                    infills.push(line);
                }

                x += hex_size * 1.5;
            }

            z += hex_size * consts::SQRT_2 * 0.75;
            row += 1;
        }

        infills
    }

    /// Generate concentric infill bounding box
    pub fn concentric((min, max): &(Point2D, Point2D), config: &Config) -> Vec<Segment2D> {
        let min_x = min.x; let min_z = min.z;
        let max_x = max.x; let max_z = max.z;
        // Maximum radius of the concentric circle
        let max_r = (max_x - min_x).min(max_z - min_z) / 2.0;
        let offset = config.line_width / config.infill_density;
        // Center of the bounding box
        let center = Point2D {
            x: (min_x + max_x) / 2.0,
            z: (min_z + max_z) / 2.0,
        };
        let circles = (max_r / offset).floor() as usize;
        let mut infills = Vec::new();
        // Number of segments per circle
        let resolution: usize = 64;
        
        for i in 1..=circles {
            let radius = i as f32 * offset;
            let mut previous = None;
            for j in 0..resolution {
                let angle = 2.0 * consts::PI * (j as f32) / (resolution as f32);
                let x = center.x + radius * angle.cos();
                let z = center.z + radius * angle.sin();
                if let Some(start) = previous {
                    let end = Point2D { x, z };
                    infills.push(Segment2D { start, end });
                    previous = Some(end);
                } else {
                    previous = Some(Point2D { x, z });
                }
            }
        }

        infills
    }
}