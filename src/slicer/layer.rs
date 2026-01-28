use std::collections::HashMap;
use super::Config;
use super::infill::*;
use super::geometry::*;

/// Contour information with type and level
#[derive(Debug, Clone)]
pub struct Contour {
    pub contour: Polygon2D,
    /// The outer is true for even-level contours (material), false for odd-level contours (holes)
    pub outer: bool,
    /// Bounding box of the contour (min, max)
    pub bounding: (Point2D, Point2D),
}

/// Represents a single layer of the slice
#[derive(Debug, Clone)]
pub struct Layer<'a> {
    pub index: usize,
    /// Contours with type and level information
    pub contours: Vec<Contour>,
    pub limbics: Vec<Vec<Segment2D>>,
    /// Infill segments with flag indicating if it should be extruded (true) or not (false)
    pub infills: Vec<(bool, Segment2D)>,
    /// Bounding box of the layer (union of all contour bounding boxes, min, max)
    pub bounding: (Point2D, Point2D),
    config: &'a Config,
}

impl <'a> Layer<'a> {
    /// Create a new layer with the given height and configuration
    pub fn new(index: usize, config: &'a Config) -> Self {
        Self {
            index,
            contours: Vec::new(),
            limbics: Vec::new(),
            infills: Vec::new(),
            bounding: (
                Point2D::new(f32::MAX, f32::MAX), 
                Point2D::new(f32::MIN, f32::MIN)
            ),
            config,
        }
    }

    /// Calculate the height of this layer
    pub fn height(&self) -> f32 {
        self.config.height(self.index)
    }

    /// Generate layer from contours
    pub fn delineate(&mut self) {
        let n = self.contours.len();
        if n > 0 {
            // Generate infill using layer bounding box
            let infills = if self.index == 0 {
                 match self.config.dense_infill {
                    DenseInfill::Grid => {
                        DenseInfill::grid(&self.bounding, &self.config)
                    }
                    DenseInfill::Line => {
                        DenseInfill::line(&self.bounding, &self.config)
                    }
                }
            } else {
                match self.config.sparse_infill {
                    SparseInfill::Grid => {
                        SparseInfill::grid(&self.bounding, &self.config)
                    }
                    SparseInfill::Line => {
                        SparseInfill::line(&self.bounding, &self.config)
                    }
                    SparseInfill::Honeycomb => {
                        SparseInfill::honeycomb(&self.bounding, &self.config)
                    }
                    SparseInfill::Concentric => {
                        SparseInfill::concentric(&self.bounding, &self.config)
                    }
                }
            };

            // Infill segment hit contour points
            let mut infhits: Vec<Vec<(Point2D, bool, f32)>> = Vec::with_capacity(infills.len());

            for i in 0..n {
                // Use the left edge center of bounding box as test point
                // This is more stable than centroid, especially for complex shapes
                let bounding = &self.contours[i].bounding;
                let tespoint = Point2D::new(
                    bounding.0.x,  // left edge (min x)
                    (bounding.0.z + bounding.1.z) / 2.0,  // center z
                );

                // Count ray intersections with all other contours
                // Ray casting: from test point in positive x direction
                let mut counts: usize = 0;
                for j in 0..n {
                    if i != j {
                        let contour = &self.contours[j].contour;
                        counts += contour.raycount(tespoint);
                    }
                }

                let contour = &mut self.contours[i];
                contour.outer = counts % 2 == 0;

                // Determine offset direction based on contour type
                // Outer contour: offset inward (negative distance)
                // Inner contour: offset outward (positive distance)
                let distance = if contour.outer {
                    -self.config.line_width // inward for outer contour
                } else {
                    self.config.line_width // outward for inner contour
                };

                // Generate perimeters for each contour
                let mut current = contour.contour.clone();
                for i in 0..self.config.perimeters {
                    let mut segs = vec![];
                    let n = current.points.len();
                    for i in 0..n {
                        let j = (i + 1) % n;
                        let start = current.points[i];
                        let end = current.points[j];
                        segs.push(Segment2D::new(start, end));
                    }
                    self.limbics.push(segs);
                    if i < self.config.perimeters - 1 {
                        // Offset contour for next perimeter iteration
                        current = current.offset(distance);
                    }
                }

                // Clip infill segments based on contour levels and mark empty travel
                for (i, infill) in infills.iter().enumerate() {
                    let points = contour.contour.intersects(infill);
                    if infhits.len() <= i {
                        infhits.push(vec![]);
                    }
                    for (t, p) in points {
                        infhits[i].push((p, contour.outer, t));
                    }
                }
            }

            for infhit in infhits.iter_mut() {
                let n = infhit.len();
                if n > 0 {
                    infhit.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
                    for i in 0..n - 1 {
                        let j = (i + 1) % n;
                        let k = (i + n - 1) % n;
                        if j > 0 {
                            let curr = infhit[i];
                            let next = infhit[j];
                            let prev = infhit[k];
                            let fill = if curr.1 == true && next.1 == false {
                                true
                            } else if curr.1 == false && next.1 == true {
                                !prev.1
                            } else {
                                curr.1
                            };
                            self.infills.push((fill, Segment2D::new(curr.0, next.0)));
                        }
                    }
                }
            }
        }
    }

    /// Connect line segments into closed contours
    pub fn adjacence(&mut self, segments: Vec<Segment2D>) {
        let mut contours = Vec::new();
        let mut consumes = vec![false; segments.len()];

        // Build adjacency map
        let mut adjacency: HashMap<Point2D, Vec<(usize, bool)>> = HashMap::new();

        let n = segments.len();

        for (i, curr) in segments.iter().enumerate() {
            let prev = segments[(i + n - 1) % n];
            let sees = curr.start == prev.end && curr.end == prev.start;
            let ssee = curr.start == prev.start && curr.end == prev.end;
            if sees || ssee {
                consumes[i] = true;
            } else {
                adjacency.entry(curr.start)
                    .or_insert_with(Vec::new)
                    .push((i, true));
                adjacency.entry(curr.end)
                    .or_insert_with(Vec::new)
                    .push((i, false));
            }
        }

        // Initialize layer bounding box
        let mut lmin = Point2D::new(f32::MAX, f32::MAX);
        let mut lmax = Point2D::new(f32::MIN, f32::MIN);

        for i in 0..n {
            if !consumes[i] {
                let mut contour = Polygon2D::new();
                let mut current = segments[i];
                consumes[i] = true;

                // Initialize contour bounding box with first point
                let mut cmin = Point2D::new(f32::MAX, f32::MAX);
                let mut cmax = Point2D::new(f32::MIN, f32::MIN);

                contour.push(current.start);

                if let Some(last) = contour.points.last() {
                    cmin.x = cmin.x.min(last.x);
                    cmin.z = cmin.z.min(last.z);
                    cmax.x = cmax.x.max(last.x);
                    cmax.z = cmax.z.max(last.z);
                }

                loop {
                    contour.push(current.end);

                    if let Some(last) = contour.points.last() {
                        // Update contour bounding box with current end point
                        cmin.x = cmin.x.min(last.x);
                        cmin.z = cmin.z.min(last.z);
                        cmax.x = cmax.x.max(last.x);
                        cmax.z = cmax.z.max(last.z);
                    }

                    let mut keeping = false;
                    // Look up next segment using hash map (O(1))
                    if let Some(neighbors) = adjacency.get(&current.end) {
                        // Find first unused neighbor
                        for &(n, started) in neighbors {
                            if !consumes[n] {
                                let nextseg = segments[n];
                                current = if !started {
                                    // If neighbor's end matches, reverse it
                                    Segment2D::new(nextseg.end, nextseg.start)
                                } else {
                                    nextseg
                                };
                                consumes[n] = true;
                                keeping = true;
                                break;
                            }
                        }
                    }
                    if !keeping {
                        break;
                    }
                }

                // Close the contour if possible
                if contour.closed() {
                    // Can close the contour, first and last point are the same
                    // Remove the last point to avoid duplication
                    contour.points.pop();

                    // Update layer bounding box with this contour's bounding box
                    lmin.x = lmin.x.min(cmin.x);
                    lmin.z = lmin.z.min(cmin.z);
                    lmax.x = lmax.x.max(cmax.x);
                    lmax.z = lmax.z.max(cmax.z);

                    // Create Contour with calculated bounding box
                    contours.push(Contour {
                        contour,
                        outer: true,
                        bounding: (cmin, cmax),
                    });
                }
            }
        }

        self.contours = contours;
        self.bounding = (lmin, lmax);
    }
}
