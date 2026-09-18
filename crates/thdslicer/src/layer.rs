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
                        DenseInfill::grid(&self.bounding, self.config)
                    }
                    DenseInfill::Line => {
                        DenseInfill::line(&self.bounding, self.config)
                    }
                }
            } else {
                match self.config.sparse_infill {
                    SparseInfill::Grid => {
                        SparseInfill::grid(&self.bounding, self.config)
                    }
                    SparseInfill::Line => {
                        SparseInfill::line(&self.bounding, self.config)
                    }
                    SparseInfill::Honeycomb => {
                        SparseInfill::honeycomb(&self.bounding, self.config)
                    }
                    SparseInfill::Concentric => {
                        SparseInfill::concentric(&self.bounding, self.config)
                    }
                }
            };

            // Infill segment hit contour points, one bucket per infill line, filled
            // with the crossings of every contour.
            let mut infhits: Vec<Vec<(f32, Point2D)>> = vec![Vec::new(); infills.len()];

            for i in 0..n {
                // Use the left edge center of bounding box as test point
                // This is more stable than centroid, especially for complex shapes
                let bounding = &self.contours[i].bounding;
                let tespoint = Point2D::new(
                    bounding.0.x,  // left edge (min x)
                    (bounding.0.z + bounding.1.z) / 2.0,  // center z
                );

                // Count ray intersections with all other contours.
                // Ray casting: from test point in positive x direction. An even
                // count means the contour is not enclosed by any other, so the
                // material it bounds is solid; an odd count means it is a hole, or
                // an island inside a hole, one level down.
                let mut counts: usize = 0;
                for j in 0..n {
                    if i != j {
                        let contour = &self.contours[j].contour;
                        counts += contour.raycount(tespoint);
                    }
                }

                let contour = &mut self.contours[i];
                contour.outer = counts.is_multiple_of(2);

                // Determine offset direction based on contour type
                // Polygon::offset shrinks with positive distance and expands with negative
                // Outer contour: offset inward (shrink, positive distance)
                // Inner contour (hole): offset outward (expand the hole, negative distance)
                let distance = if contour.outer {
                    self.config.line_width
                } else {
                    -self.config.line_width
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
                    for hit in contour.contour.intersects(infill) {
                        infhits[i].push(hit);
                    }
                }
            }

            for infhit in infhits.iter_mut() {
                let n = infhit.len();
                if n > 0 {
                    infhit.sort_by(|a, b| a.0.total_cmp(&b.0));
                    // Pair consecutive crossings along the infill line: the line
                    // enters the material at the first crossing, leaves it at the
                    // second, and so on, because every contour of the layer - an
                    // outer wall, a hole, an island inside that hole - is part of
                    // the same even-odd fill. The span before the first crossing and
                    // the one after the last are outside the part and are skipped.
                    for (i, pair) in infhit.windows(2).enumerate() {
                        let segment = Segment2D::new(pair[0].1, pair[1].1);
                        // A span of no length is a line that only grazed the
                        // outline: it neither extrudes nor travels.
                        if segment.start != segment.end {
                            self.infills.push((i % 2 == 0, segment));
                        }
                    }
                }
            }
        }
    }

    /// Connect line segments into closed contours
    pub fn adjacence(&mut self, mut segments: Vec<Segment2D>) {
        let mut contours = Vec::new();
        let mut consumes = vec![false; segments.len()];

        // Build adjacency map
        let mut adjacency: QikMap<Point2D, Vec<(usize, bool)>> = QikMap::default();

        let n = segments.len();

        // The ends are welded onto shared points on the way in. The two triangles
        // that met along an edge interpolate their shared crossing from opposite
        // ends, so their results agree only to within rounding, and the map below
        // is keyed on the rounded coordinate: unwelded, the two can fall either
        // side of a key boundary, the pair is then never joined, the loop stays
        // open and every segment of the contour it belongs to is dropped.
        let mut welded: QikMap<Point2D, Point2D> = QikMap::default();

        // The grid `Point2D` hashes and compares on, so that the two ends of a
        // segment are keyed the same way everywhere.
        let cell = |point: Point2D| (Cov::<E4>(point.x).hash(), Cov::<E4>(point.z).hash());

        // Segments already handed over, by the direction they run in. A segment
        // that comes back the other way is the same edge seen from the far side:
        // the material only touches the layer plane there, so the two of them
        // bound nothing and neither belongs to a contour. A segment that comes
        // back the same way is a face the mesh carries twice, and one copy is
        // enough. Both are decided here rather than by looking at the segment
        // before, which only finds a pair that happened to be piled up next to
        // each other.
        let mut handed: QikMap<(i64, i64, i64, i64), usize> = QikMap::default();

        for i in 0..n {
            let mut curr = segments[i];
            curr.start = Qik::weld(&mut welded, curr.start);
            curr.end = Qik::weld(&mut welded, curr.end);
            segments[i] = curr;

            let (sx, sz) = cell(curr.start);
            let (ex, ez) = cell(curr.end);
            if let Some(&other) = handed.get(&(ex, ez, sx, sz)) {
                // Handed over from the far side: the pair cancels.
                consumes[i] = true;
                consumes[other] = true;
                handed.remove(&(ex, ez, sx, sz));
            } else if handed.contains_key(&(sx, sz, ex, ez)) {
                // The same segment again.
                consumes[i] = true;
            } else {
                handed.insert((sx, sz, ex, ez), i);
                adjacency.entry(curr.start)
                    .or_default()
                    .push((i, true));
                adjacency.entry(curr.end)
                    .or_default()
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

                // Twice the signed area, accumulated while the points are pushed.
                // It is the winding of the contour, which tells which way
                // [`Polygon2D::offset`] moves the walls, so carrying it along here
                // saves a second walk over the finished contour.
                let mut doubled = 0.0;
                let mut previous = current.start;

                loop {
                    doubled += previous.x * current.end.z - current.end.x * previous.z;
                    previous = current.end;
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
                    // Every contour is stored counter-clockwise in the (x, z)
                    // plane, whatever direction the triangles it was stitched from
                    // happened to hand it over in. A layer plane that runs exactly
                    // through a flat cap is cut out of the cap's own edges and comes
                    // out the other way round, which would otherwise flip the
                    // perimeters of that layer outwards.
                    if doubled < 0.0 {
                        contour.points.reverse();
                    }

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
