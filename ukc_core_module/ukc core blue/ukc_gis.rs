// ============================================
// GIS SYSTEM FOR UKC - Geographic Information System
// Complete maritime navigation and safety analysis
// ============================================

use std::collections::HashMap;
use std::f64::consts::PI;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

// ============================================
// CORE GEOSPATIAL TYPES
// ============================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordinate {
    pub latitude: f64,
    pub longitude: f64,
}

impl Coordinate {
    pub fn new(lat: f64, lng: f64) -> Self {
        Self {
            latitude: lat,
            longitude: lng,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.latitude >= -90.0 && self.latitude <= 90.0 &&
        self.longitude >= -180.0 && self.longitude <= 180.0
    }

    /// Convert to radians
    pub fn to_radians(&self) -> (f64, f64) {
        (self.latitude.to_radians(), self.longitude.to_radians())
    }
}

#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lng: f64,
    pub max_lng: f64,
}

impl BoundingBox {
    pub fn new(min_lat: f64, max_lat: f64, min_lng: f64, max_lng: f64) -> Self {
        Self { min_lat, max_lat, min_lng, max_lng }
    }

    pub fn contains(&self, coord: &Coordinate) -> bool {
        coord.latitude >= self.min_lat && coord.latitude <= self.max_lat &&
        coord.longitude >= self.min_lng && coord.longitude <= self.max_lng
    }

    pub fn center(&self) -> Coordinate {
        Coordinate {
            latitude: (self.min_lat + self.max_lat) / 2.0,
            longitude: (self.min_lng + self.max_lng) / 2.0,
        }
    }

    pub fn expand(&self, factor: f64) -> Self {
        let lat_range = (self.max_lat - self.min_lat) * factor;
        let lng_range = (self.max_lng - self.min_lng) * factor;
        Self {
            min_lat: self.min_lat - lat_range,
            max_lat: self.max_lat + lat_range,
            min_lng: self.min_lng - lng_range,
            max_lng: self.max_lng + lng_range,
        }
    }
}

// ============================================
// DEPTH AND BATHYMETRY
// ============================================

#[derive(Debug, Clone)]
pub struct DepthPoint {
    pub coordinate: Coordinate,
    pub depth: f64, // meters, positive = below sea level
    pub quality: DepthQuality,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DepthQuality {
    High,      // Surveyed
    Medium,    // Chart data
    Low,       // Estimated
    Unknown,
}

#[derive(Debug, Clone)]
pub struct DepthGrid {
    pub cells: HashMap<(i32, i32), Vec<DepthPoint>>,
    pub resolution: f64, // degrees per cell
    pub bounds: BoundingBox,
}

impl DepthGrid {
    pub fn new(resolution: f64, bounds: BoundingBox) -> Self {
        Self {
            cells: HashMap::new(),
            resolution,
            bounds,
        }
    }

    fn cell_key(&self, coord: &Coordinate) -> (i32, i32) {
        let lat_idx = ((coord.latitude - self.bounds.min_lat) / self.resolution) as i32;
        let lng_idx = ((coord.longitude - self.bounds.min_lng) / self.resolution) as i32;
        (lat_idx, lng_idx)
    }

    pub fn add_depth_point(&mut self, point: DepthPoint) {
        let key = self.cell_key(&point.coordinate);
        self.cells.entry(key).or_insert_with(Vec::new).push(point);
    }

    pub fn get_depth(&self, coord: &Coordinate) -> Option<f64> {
        let key = self.cell_key(coord);
        if let Some(points) = self.cells.get(&key) {
            if points.is_empty() {
                return None;
            }
            // Simple average of depths in cell
            let avg = points.iter().map(|p| p.depth).sum::<f64>() / points.len() as f64;
            Some(avg)
        } else {
            None
        }
    }

    /// Interpolate depth using inverse distance weighting
    pub fn interpolate_depth(&self, coord: &Coordinate, max_distance: f64) -> Option<f64> {
        let mut total_weight = 0.0;
        let mut weighted_depth = 0.0;
        let mut found = false;

        // Search nearby cells
        let center = self.cell_key(coord);
        for lat_offset in -2..=2 {
            for lng_offset in -2..=2 {
                let key = (center.0 + lat_offset, center.1 + lng_offset);
                if let Some(points) = self.cells.get(&key) {
                    for point in points {
                        let dist = self.haversine_distance(coord, &point.coordinate);
                        if dist <= max_distance {
                            let weight = 1.0 / (dist + 0.001); // Add small epsilon to avoid division by zero
                            weighted_depth += point.depth * weight;
                            total_weight += weight;
                            found = true;
                        }
                    }
                }
            }
        }

        if found {
            Some(weighted_depth / total_weight)
        } else {
            None
        }
    }

    /// Haversine distance in meters
    fn haversine_distance(&self, a: &Coordinate, b: &Coordinate) -> f64 {
        let (lat1, lng1) = a.to_radians();
        let (lat2, lng2) = b.to_radians();
        let dlat = lat2 - lat1;
        let dlng = lng2 - lng1;
        let a = (dlat / 2.0).sin().powi(2) + 
                lat1.cos() * lat2.cos() * 
                (dlng / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        const EARTH_RADIUS: f64 = 6371000.0; // meters
        EARTH_RADIUS * c
    }

    /// Load depth data from CSV
    pub fn load_from_csv(&mut self, path: &str) -> Result<(), String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| format!("Failed to read line {}: {}", line_num, e))?;
            if line_num == 0 || line.trim().is_empty() {
                continue; // Skip header or empty lines
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 3 {
                let lat: f64 = parts[0].parse().map_err(|e| format!("Invalid latitude: {}", e))?;
                let lng: f64 = parts[1].parse().map_err(|e| format!("Invalid longitude: {}", e))?;
                let depth: f64 = parts[2].parse().map_err(|e| format!("Invalid depth: {}", e))?;

                self.add_depth_point(DepthPoint {
                    coordinate: Coordinate::new(lat, lng),
                    depth,
                    quality: DepthQuality::Medium,
                });
            }
        }
        Ok(())
    }

    /// Generate synthetic depth data for testing
    pub fn generate_synthetic_data(&mut self) {
        let lat_step = 0.1;
        let lng_step = 0.1;
        let mut lat = self.bounds.min_lat;
        while lat <= self.bounds.max_lat {
            let mut lng = self.bounds.min_lng;
            while lng <= self.bounds.max_lng {
                let depth = 10.0 + 5.0 * (lat * 0.5).sin() + 3.0 * (lng * 0.3).cos();
                self.add_depth_point(DepthPoint {
                    coordinate: Coordinate::new(lat, lng),
                    depth: depth.max(0.0),
                    quality: DepthQuality::Medium,
                });
                lng += lng_step;
            }
            lat += lat_step;
        }
    }
}

// ============================================
// MARITIME FEATURES
// ============================================

#[derive(Debug, Clone)]
pub struct Port {
    pub name: String,
    pub coordinate: Coordinate,
    pub max_draft: f64, // Maximum draft allowed in meters
    pub facilities: Vec<PortFacility>,
}

#[derive(Debug, Clone)]
pub enum PortFacility {
    Container,
    Bulk,
    Oil,
    Gas,
    Passenger,
    Fishing,
    General,
}

#[derive(Debug, Clone)]
pub struct Obstruction {
    pub name: String,
    pub coordinate: Coordinate,
    pub clearance: f64, // Clearance above seabed in meters
    pub obstruction_type: ObstructionType,
}

#[derive(Debug, Clone)]
pub enum ObstructionType {
    Wreck,
    Rock,
    Shoal,
    Pipeline,
    Cable,
    Anchor,
    Other,
}

#[derive(Debug, Clone)]
pub struct NavigationalAid {
    pub name: String,
    pub coordinate: Coordinate,
    pub aid_type: AidType,
    pub range: f64, // Range in nautical miles
    pub characteristic: String,
}

#[derive(Debug, Clone)]
pub enum AidType {
    Lighthouse,
    Buoy,
    Beacon,
    LightVessel,
    Racon,
}

// ============================================
// ROUTE AND PATH PLANNING
// ============================================

#[derive(Debug, Clone)]
pub struct Route {
    pub waypoints: Vec<Waypoint>,
    pub total_distance: f64,
    pub estimated_time: f64, // hours
    pub safety_analysis: RouteSafety,
}

#[derive(Debug, Clone)]
pub struct Waypoint {
    pub coordinate: Coordinate,
    pub name: String,
    pub depth: Option<f64>,
    pub required_depth: Option<f64>,
    pub is_safe: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct RouteSafety {
    pub min_ukc: f64,
    pub max_draft: f64,
    pub unsafe_waypoints: Vec<usize>,
    pub overall_status: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RouteSegment {
    pub start: Coordinate,
    pub end: Coordinate,
    pub distance: f64,
    pub bearing: f64,
    pub depth_profile: Vec<DepthPoint>,
    pub min_depth: f64,
    pub max_depth: f64,
    pub average_depth: f64,
}

#[derive(Debug, Clone)]
pub struct RouteOptimizer {
    pub safety_margin: f64, // Additional safety margin in meters
    pub preferred_depth: f64,
    pub max_turn_angle: f64, // Maximum turn angle in degrees
}

impl RouteOptimizer {
    pub fn new(safety_margin: f64, preferred_depth: f64, max_turn_angle: f64) -> Self {
        Self {
            safety_margin,
            preferred_depth,
            max_turn_angle,
        }
    }

    /// Find safe route between two points using A* algorithm
    pub fn find_route(
        &self,
        start: Coordinate,
        end: Coordinate,
        depth_grid: &DepthGrid,
        ship_draft: f64,
    ) -> Option<Route> {
        let mut open_set = Vec::new();
        let mut closed_set = HashMap::new();
        let start_node = SearchNode::new(start, 0.0, self.heuristic(&start, &end));
        open_set.push(start_node);

        while let Some(current) = open_set.pop() {
            if self.heuristic(&current.coordinate, &end) < 0.01 {
                // Reached destination
                return Some(self.reconstruct_path(&current, &closed_set));
            }

            closed_set.insert(current.coordinate, current.clone());

            // Generate neighbors
            for neighbor in self.generate_neighbors(&current, depth_grid, ship_draft) {
                if closed_set.contains_key(&neighbor.coordinate) {
                    continue;
                }

                let g_score = current.g_score + self.distance(&current.coordinate, &neighbor.coordinate);
                let h_score = self.heuristic(&neighbor.coordinate, &end);
                let f_score = g_score + h_score;

                if let Some(existing) = open_set.iter_mut().find(|n| n.coordinate == neighbor.coordinate) {
                    if g_score < existing.g_score {
                        existing.g_score = g_score;
                        existing.f_score = f_score;
                        existing.parent = Some(Box::new(current.clone()));
                    }
                } else {
                    let mut new_node = neighbor;
                    new_node.g_score = g_score;
                    new_node.f_score = f_score;
                    new_node.parent = Some(Box::new(current.clone()));
                    open_set.push(new_node);
                }
            }

            // Sort by f_score
            open_set.sort_by(|a, b| a.f_score.partial_cmp(&b.f_score).unwrap());
        }

        None
    }

    /// A* helper functions
    fn heuristic(&self, a: &Coordinate, b: &Coordinate) -> f64 {
        self.distance(a, b)
    }

    fn distance(&self, a: &Coordinate, b: &Coordinate) -> f64 {
        let dlat = (b.latitude - a.latitude).to_radians();
        let dlng = (b.longitude - a.longitude).to_radians();
        let lat1 = a.latitude.to_radians();
        let lat2 = b.latitude.to_radians();
        let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlng / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        const EARTH_RADIUS: f64 = 6371000.0;
        EARTH_RADIUS * c
    }

    fn generate_neighbors(
        &self,
        current: &SearchNode,
        depth_grid: &DepthGrid,
        ship_draft: f64,
    ) -> Vec<SearchNode> {
        let mut neighbors = Vec::new();
        let step_size = 0.01; // degrees (~1.1 km)
        let angles = vec![0.0, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0];
        let step_rad = step_size.to_radians();

        for &angle_deg in &angles {
            let angle = angle_deg.to_radians();
            let lat_offset = step_size * angle.cos();
            let lng_offset = step_size * angle.sin();
            let new_lat = current.coordinate.latitude + lat_offset;
            let new_lng = current.coordinate.longitude + lng_offset;

            // Check bounds
            if !(new_lat >= -90.0 && new_lat <= 90.0 && new_lng >= -180.0 && new_lng <= 180.0) {
                continue;
            }

            let coord = Coordinate::new(new_lat, new_lng);
            
            // Check depth
            if let Some(depth) = depth_grid.get_depth(&coord) {
                let required_depth = ship_draft + self.safety_margin;
                if depth >= required_depth {
                    neighbors.push(SearchNode {
                        coordinate: coord,
                        g_score: 0.0,
                        f_score: 0.0,
                        parent: None,
                    });
                }
            }
        }

        neighbors
    }

    fn reconstruct_path(&self, current: &SearchNode, closed_set: &HashMap<Coordinate, SearchNode>) -> Route {
        let mut waypoints = Vec::new();
        let mut node = current.clone();

        while let Some(parent) = node.parent {
            waypoints.push(Waypoint {
                coordinate: node.coordinate,
                name: format!("WP_{}", waypoints.len()),
                depth: None,
                required_depth: None,
                is_safe: None,
            });
            node = *parent;
        }
        waypoints.reverse();

        Route {
            waypoints,
            total_distance: 0.0,
            estimated_time: 0.0,
            safety_analysis: RouteSafety {
                min_ukc: 0.0,
                max_draft: 0.0,
                unsafe_waypoints: Vec::new(),
                overall_status: "UNKNOWN".to_string(),
                warnings: Vec::new(),
            },
        }
    }
}

#[derive(Debug, Clone)]
struct SearchNode {
    coordinate: Coordinate,
    g_score: f64,
    f_score: f64,
    parent: Option<Box<SearchNode>>,
}

impl SearchNode {
    fn new(coordinate: Coordinate, g_score: f64, f_score: f64) -> Self {
        Self {
            coordinate,
            g_score,
            f_score,
            parent: None,
        }
    }
}

// ============================================
// VISUALIZATION AND MAPPING
// ============================================

#[derive(Debug, Clone)]
pub struct MapLayer {
    pub name: String,
    pub visible: bool,
    pub opacity: f64,
    pub features: Vec<MapFeature>,
}

#[derive(Debug, Clone)]
pub enum MapFeature {
    Point {
        coordinate: Coordinate,
        label: String,
        icon: String,
        color: String,
        size: f64,
    },
    Line {
        coordinates: Vec<Coordinate>,
        color: String,
        width: f64,
        style: LineStyle,
    },
    Polygon {
        coordinates: Vec<Coordinate>,
        fill_color: String,
        stroke_color: String,
        opacity: f64,
    },
    Text {
        coordinate: Coordinate,
        text: String,
        color: String,
        size: f64,
    },
}

#[derive(Debug, Clone)]
pub enum LineStyle {
    Solid,
    Dashed,
    Dotted,
    DashDot,
}

#[derive(Debug, Clone)]
pub struct MapConfig {
    pub center: Coordinate,
    pub zoom: f64,
    pub projection: Projection,
    pub units: MapUnits,
}

#[derive(Debug, Clone)]
pub enum Projection {
    Mercator,
    LambertConformal,
    PolarStereographic,
}

#[derive(Debug, Clone)]
pub enum MapUnits {
    Meters,
    Kilometers,
    NauticalMiles,
    Degrees,
}

impl MapUnits {
    pub fn to_meters(&self, value: f64) -> f64 {
        match self {
            MapUnits::Meters => value,
            MapUnits::Kilometers => value * 1000.0,
            MapUnits::NauticalMiles => value * 1852.0,
            MapUnits::Degrees => value * 111319.0, // Approximate
        }
    }
}

pub struct MapRenderer {
    pub config: MapConfig,
    pub layers: Vec<MapLayer>,
    pub width: u32,
    pub height: u32,
}

impl MapRenderer {
    pub fn new(width: u32, height: u32, config: MapConfig) -> Self {
        Self {
            config,
            layers: Vec::new(),
            width,
            height,
        }
    }

    pub fn add_layer(&mut self, layer: MapLayer) {
        self.layers.push(layer);
    }

    /// Convert geographic coordinates to screen coordinates (simplified Mercator projection)
    pub fn geo_to_screen(&self, coord: &Coordinate) -> (f64, f64) {
        let center = &self.config.center;
        let zoom = self.config.zoom;

        // Mercator projection
        let lat_rad = coord.latitude.to_radians();
        let lng_rad = coord.longitude.to_radians();
        let center_lat_rad = center.latitude.to_radians();
        let center_lng_rad = center.longitude.to_radians();

        let x = (lng_rad - center_lng_rad) * zoom * self.width as f64 / (2.0 * PI);
        let y = (lat_rad - center_lat_rad) * zoom * self.height as f64 / (2.0 * PI);

        (x + self.width as f64 / 2.0, -y + self.height as f64 / 2.0)
    }

    /// Generate a simple map representation (text-based)
    pub fn render_text(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("MAP: {}x{}\n", self.width, self.height));
        output.push_str(&format!("Center: {:?}\n", self.config.center));
        output.push_str(&format!("Zoom: {}\n", self.config.zoom));
        output.push_str("=".repeat(60).as_str());
        output.push_str("\n");

        for layer in &self.layers {
            if !layer.visible {
                continue;
            }
            output.push_str(&format!("\nLayer: {} (opacity: {:.2})\n", layer.name, layer.opacity));
            for feature in &layer.features {
                match feature {
                    MapFeature::Point { coordinate, label, icon, color, size } => {
                        let (x, y) = self.geo_to_screen(coordinate);
                        output.push_str(&format!("  Point: {} at ({:.1}, {:.1}) [{}] {}\n", 
                            label, x, y, icon, color));
                    }
                    MapFeature::Line { coordinates, color, width, style } => {
                        output.push_str(&format!("  Line: {} points, {} width, {:?} style\n", 
                            coordinates.len(), width, style));
                        for (i, coord) in coordinates.iter().enumerate() {
                            if i < 3 || i >= coordinates.len() - 3 {
                                let (x, y) = self.geo_to_screen(coord);
                                output.push_str(&format!("    ({:.1}, {:.1}) ", x, y));
                            }
                        }
                        output.push_str("\n");
                    }
                    MapFeature::Polygon { coordinates, fill_color, stroke_color, opacity } => {
                        output.push_str(&format!("  Polygon: {} points, fill: {}, stroke: {}\n", 
                            coordinates.len(), fill_color, stroke_color));
                    }
                    MapFeature::Text { coordinate, text, color, size } => {
                        let (x, y) = self.geo_to_screen(coordinate);
                        output.push_str(&format!("  Text: '{}' at ({:.1}, {:.1})\n", text, x, y));
                    }
                }
            }
        }

        output
    }

    /// Generate HTML/JavaScript map visualization
    pub fn render_html(&self) -> String {
        let mut html = String::new();
        html.push_str(r#"<!DOCTYPE html>
<html>
<head>
    <title>UKC GIS Map</title>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css" />
    <script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"></script>
    <style>
        #map { height: 100vh; width: 100%; margin: 0; padding: 0; }
        .info-panel { position: absolute; top: 10px; right: 10px; background: white; padding: 10px; border-radius: 5px; box-shadow: 0 2px 5px rgba(0,0,0,0.3); z-index: 1000; max-width: 300px; }
        .legend { position: absolute; bottom: 30px; left: 10px; background: white; padding: 10px; border-radius: 5px; box-shadow: 0 2px 5px rgba(0,0,0,0.3); z-index: 1000; }
    </style>
</head>
<body>
    <div id="map"></div>
    <div class="info-panel">
        <h3>UKC Navigation</h3>
        <div id="info-content">Click on map for info</div>
    </div>
    <div class="legend">
        <div><span style="color:#27ae60;">●</span> Safe depth</div>
        <div><span style="color:#e74c3c;">●</span> Unsafe depth</div>
        <div><span style="color:#f39c12;">●</span> Warning</div>
    </div>
    <script>
"#);

        // JavaScript for map rendering
        html.push_str("
        const map = L.map('map').setView([");
        html.push_str(&format!("{}, {}", self.config.center.latitude, self.config.center.longitude));
        html.push_str(&format!("], {:.1});\n", self.config.zoom));

        html.push_str("
        L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
            attribution: '© OpenStreetMap contributors'
        }).addTo(map);
        ");

        // Add features from layers
        for layer in &self.layers {
            if !layer.visible {
                continue;
            }
            for feature in &layer.features {
                match feature {
                    MapFeature::Point { coordinate, label, icon, color, size } => {
                        html.push_str(&format!("
        L.marker([{}, {}], {{
            icon: L.divIcon({{
                className: 'custom-marker',
                html: '<div style=\"background:{};color:white;padding:4px 8px;border-radius:50%;font-size:{}px;\">{}</div>'
            }})
        }}).addTo(map)
        .bindPopup('<b>{}</b><br>Location: {:.4}, {:.4}');
        ", coordinate.latitude, coordinate.longitude, 
        color, size, icon, label, coordinate.latitude, coordinate.longitude));
                    }
                    MapFeature::Line { coordinates, color, width, style } => {
                        let coords_str: String = coordinates.iter()
                            .map(|c| format!("[{}, {}]", c.latitude, c.longitude))
                            .collect::<Vec<_>>()
                            .join(",");
                        html.push_str(&format!("
        L.polyline([{}], {{
            color: '{}',
            weight: {},
            dashArray: '{}'
        }}).addTo(map).bindPopup('Route');
        ", coords_str, color, width, match style {
            LineStyle::Solid => "",
            LineStyle::Dashed => "10, 10",
            LineStyle::Dotted => "3, 3",
            LineStyle::DashDot => "10, 5, 2, 5",
        }));
                    }
                    MapFeature::Polygon { coordinates, fill_color, stroke_color, opacity } => {
                        let coords_str: String = coordinates.iter()
                            .map(|c| format!("[{}, {}]", c.latitude, c.longitude))
                            .collect::<Vec<_>>()
                            .join(",");
                        html.push_str(&format!("
        L.polygon([{}], {{
            color: '{}',
            fillColor: '{}',
            fillOpacity: {},
            weight: 2
        }}).addTo(map).bindPopup('Area');
        ", coords_str, stroke_color, fill_color, opacity));
                    }
                    MapFeature::Text { coordinate, text, color, size } => {
                        html.push_str(&format!("
        L.marker([{}, {}], {{
            icon: L.divIcon({{
                className: 'text-marker',
                html: '<div style=\"color:{};font-size:{}px;font-weight:bold;text-shadow:0 0 3px white;\">{}</div>'
            }})
        }}).addTo(map);
        ", coordinate.latitude, coordinate.longitude, color, size, text));
                    }
                }
            }
        }

        // Add depth grid visualization
        html.push_str("
        // Depth grid simulation
        function addDepthGrid() {
            // This would be populated with actual depth data
        }
        addDepthGrid();

        // Click handler
        map.on('click', function(e) {
            const lat = e.latlng.lat;
            const lng = e.latlng.lng;
            document.getElementById('info-content').innerHTML = 
                `Location: ${lat.toFixed(4)}, ${lng.toFixed(4)}`;
        });
        ");

        html.push_str("
    </script>
</body>
</html>");

        html
    }

    /// Export map as KML file
    pub fn export_kml(&self, path: &str) -> Result<(), String> {
        let mut kml = String::new();
        kml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>
<kml xmlns="http://www.opengis.net/kml/2.2">
<Document>
"#);

        for layer in &self.layers {
            if !layer.visible {
                continue;
            }
            kml.push_str(&format!("  <name>{}</name>\n", layer.name));
            for feature in &layer.features {
                match feature {
                    MapFeature::Point { coordinate, label, .. } => {
                        kml.push_str(&format!("
  <Placemark>
    <name>{}</name>
    <Point>
      <coordinates>{},{}</coordinates>
    </Point>
  </Placemark>
", label, coordinate.longitude, coordinate.latitude));
                    }
                    MapFeature::Line { coordinates, .. } => {
                        let coords_str: String = coordinates.iter()
                            .map(|c| format!("{},{},0", c.longitude, c.latitude))
                            .collect::<Vec<_>>()
                            .join(" ");
                        kml.push_str(&format!("
  <Placemark>
    <LineString>
      <coordinates>{}</coordinates>
    </LineString>
  </Placemark>
", coords_str));
                    }
                    _ => {}
                }
            }
        }

        kml.push_str("</Document></kml>");

        let mut file = File::create(path).map_err(|e| format!("Failed to create file: {}", e))?;
        file.write_all(kml.as_bytes()).map_err(|e| format!("Failed to write file: {}", e))?;
        Ok(())
    }
}

// ============================================
// MARITIME DATABASE
// ============================================

#[derive(Debug, Clone)]
pub struct MaritimeDatabase {
    pub ports: Vec<Port>,
    pub obstructions: Vec<Obstruction>,
    pub aids: Vec<NavigationalAid>,
    pub depth_grid: DepthGrid,
}

impl MaritimeDatabase {
    pub fn new(depth_grid: DepthGrid) -> Self {
        Self {
            ports: Vec::new(),
            obstructions: Vec::new(),
            aids: Vec::new(),
            depth_grid,
        }
    }

    pub fn add_port(&mut self, port: Port) {
        self.ports.push(port);
    }

    pub fn add_obstruction(&mut self, obstruction: Obstruction) {
        self.obstructions.push(obstruction);
    }

    pub fn add_aid(&mut self, aid: NavigationalAid) {
        self.aids.push(aid);
    }

    pub fn find_nearest_port(&self, coord: &Coordinate, max_distance: f64) -> Option<&Port> {
        let mut nearest = None;
        let mut min_dist = f64::INFINITY;

        for port in &self.ports {
            let dist = depth_grid.haversine_distance(coord, &port.coordinate);
            if dist < min_dist && dist <= max_distance {
                min_dist = dist;
                nearest = Some(port);
            }
        }

        nearest
    }

    pub fn find_obstructions_near(&self, coord: &Coordinate, radius: f64) -> Vec<&Obstruction> {
        let mut found = Vec::new();
        for obstruction in &self.obstructions {
            let dist = depth_grid.haversine_distance(coord, &obstruction.coordinate);
            if dist <= radius {
                found.push(obstruction);
            }
        }
        found
    }
}

// ============================================
// SEA ROUTE PRO INTEGRATION
// ============================================

#[derive(Debug, Clone)]
pub struct SeaRoutePro {
    pub calculator: UKCCalculator,
    pub database: MaritimeDatabase,
    pub map_renderer: MapRenderer,
    pub ship_params: UKCInput,
    pub route_history: Vec<Route>,
}

impl SeaRoutePro {
    pub fn new(map_renderer: MapRenderer, database: MaritimeDatabase) -> Self {
        Self {
            calculator: UKCCalculator::new(),
            database,
            map_renderer,
            ship_params: UKCInput::default(),
            route_history: Vec::new(),
        }
    }

    pub fn set_ship_params(&mut self, params: UKCInput) {
        self.ship_params = params;
    }

    /// Plan a voyage with UKC analysis
    pub fn plan_voyage(
        &mut self,
        start: Coordinate,
        end: Coordinate,
        waypoints: Option<Vec<Coordinate>>,
    ) -> Result<Route, String> {
        // Get depth data
        let depth_grid = &self.database.depth_grid;
        
        // Create route optimizer
        let optimizer = RouteOptimizer::new(1.0, 15.0, 30.0);

        // Find route
        let route = match optimizer.find_route(
            start,
            end,
            depth_grid,
            self.ship_params.static_draft,
        ) {
            Some(route) => route,
            None => return Err("No safe route found".to_string()),
        };

        // Add UKC analysis
        let mut route = route;
        let ukc_result = self.calculator.calculate(&self.ship_params);
        
        // Analyze each waypoint
        let mut unsafe_waypoints = Vec::new();
        let mut warnings = Vec::new();
        let mut min_ukc = f64::INFINITY;

        for (i, waypoint) in route.waypoints.iter_mut().enumerate() {
            if let Some(depth) = depth_grid.get_depth(&waypoint.coordinate) {
                waypoint.depth = Some(depth);
                let required = depth_grid.haversine_distance(
                    &waypoint.coordinate,
                    &start,
                );
                waypoint.required_depth = Some(required);
                let mut input = self.ship_params.clone();
                input.water_depth_available = depth;
                let result = self.calculator.calculate(&input);
                waypoint.is_safe = Some(result.is_safe);
                
                if !result.is_safe {
                    unsafe_waypoints.push(i);
                    warnings.push(format!(
                        "Waypoint {} unsafe: depth {:.2}m, requires {:.2}m",
                        i, depth, result.required_depth
                    ));
                }
                
                min_ukc = min_ukc.min(result.safety_margin);
            }
        }

        let overall_status = if unsafe_waypoints.is_empty() {
            "SAFE".to_string()
        } else {
            "UNSAFE".to_string()
        };

        route.safety_analysis = RouteSafety {
            min_ukc,
            max_draft: self.ship_params.static_draft,
            unsafe_waypoints,
            overall_status,
            warnings,
        };

        // Calculate total distance
        route.total_distance = route.waypoints.windows(2)
            .map(|w| {
                depth_grid.haversine_distance(&w[0].coordinate, &w[1].coordinate)
            })
            .sum();

        // Estimate time (assuming 10 knots average)
        route.estimated_time = route.total_distance / (10.0 * 1852.0);

        self.route_history.push(route.clone());
        Ok(route)
    }

    /// Add route to map
    pub fn visualize_route(&mut self, route: &Route) {
        // Create route feature
        let coordinates: Vec<Coordinate> = route.waypoints.iter()
            .map(|wp| wp.coordinate)
            .collect();

        let color = if route.safety_analysis.overall_status == "SAFE" {
            "#27ae60".to_string()
        } else {
            "#e74c3c".to_string()
        };

        let line_feature = MapFeature::Line {
            coordinates,
            color,
            width: 3.0,
            style: LineStyle::Solid,
        };

        // Add waypoint markers
        for (i, waypoint) in route.waypoints.iter().enumerate() {
            let color = if waypoint.is_safe.unwrap_or(false) {
                "#27ae60".to_string()
            } else {
                "#e74c3c".to_string()
            };

            let point_feature = MapFeature::Point {
                coordinate: waypoint.coordinate,
                label: format!("WP-{}", i + 1),
                icon: if waypoint.is_safe.unwrap_or(false) { "✓" } else { "✗" },
                color,
                size: 12.0,
            };

            // Add to map layer
            // This would be handled by the map renderer
        }

        // Update map
        self.map_renderer.add_layer(MapLayer {
            name: format!("Route {}", self.route_history.len()),
            visible: true,
            opacity: 1.0,
            features: vec![line_feature],
        });
    }

    /// Generate voyage report
    pub fn generate_voyage_report(&self, route: &Route) -> String {
        let mut report = String::new();
        report.push_str(&format!("\n{}\n", "=".repeat(70)));
        report.push_str(&format!("{:^70}\n", "VOYAGE REPORT"));
        report.push_str(&format!("{}\n", "=".repeat(70)));
        report.push_str(&format!("Ship: {}\n", self.ship_params.ship_name));
        report.push_str(&format!("Draft: {:.2}m\n", self.ship_params.static_draft));
        report.push_str(&format!("Total Distance: {:.2} km\n", route.total_distance / 1000.0));
        report.push_str(&format!("Estimated Time: {:.1} hours\n", route.estimated_time));
        report.push_str(&format!("Waypoints: {}\n", route.waypoints.len()));
        report.push_str(&format!("Status: {}\n", route.safety_analysis.overall_status));
        report.push_str(&format!("Minimum UKC: {:.2}m\n", route.safety_analysis.min_ukc));
        
        if !route.safety_analysis.warnings.is_empty() {
            report.push_str("\nWARNINGS:\n");
            for warning in &route.safety_analysis.warnings {
                report.push_str(&format!("  - {}\n", warning));
            }
        }

        report.push_str(&format!("{}\n", "=".repeat(70)));
        report
    }

    /// Export map as HTML
    pub fn export_map_html(&self, path: &str) -> Result<(), String> {
        let html = self.map_renderer.render_html();
        let mut file = File::create(path).map_err(|e| format!("Failed to create file: {}", e))?;
        file.write_all(html.as_bytes()).map_err(|e| format!("Failed to write file: {}", e))?;
        Ok(())
    }
}

// ============================================
// MAIN APPLICATION
// ============================================

fn main() -> Result<(), String> {
    println!("=== UKC GIS SYSTEM ===");

    // Initialize map
    let map_config = MapConfig {
        center: Coordinate::new(-5.0, 106.0), // Jakarta Bay
        zoom: 8.0,
        projection: Projection::Mercator,
        units: MapUnits::Meters,
    };

    let map_renderer = MapRenderer::new(1920, 1080, map_config);

    // Initialize depth grid
    let bounds = BoundingBox::new(-10.0, 0.0, 100.0, 115.0);
    let mut depth_grid = DepthGrid::new(0.05, bounds);
    depth_grid.generate_synthetic_data();

    // Initialize database
    let database = MaritimeDatabase::new(depth_grid);

    // Initialize SeaRoute Pro
    let mut searoute = SeaRoutePro::new(map_renderer, database);

    // Set ship parameters
    let ship_params = UKCInput {
        ship_name: "MV Maritime Explorer".to_string(),
        length: 180.0,
        breadth: 28.0,
        static_draft: 10.5,
        draft_trim: 0.3,
        draft_listing: 0.2,
        squat: 0.5,
        wave_motion: 0.8,
        water_depth_available: 20.0,
        environment: Environment::CoastalWater,
    };
    searoute.set_ship_params(ship_params);

    // Plan a voyage
    let start = Coordinate::new(-6.0, 105.0);
    let end = Coordinate::new(-4.0, 110.0);
    
    println!("Planning voyage from {:?} to {:?}", start, end);
    
    match searoute.plan_voyage(start, end, None) {
        Ok(route) => {
            println!("✅ Route found!");
            searoute.visualize_route(&route);
            println!("{}", searoute.generate_voyage_report(&route));
            
            // Export map
            searoute.export_map_html("voyage_map.html")?;
            println!("📊 Map exported to voyage_map.html");
        }
        Err(e) => {
            println!("❌ Route planning failed: {}", e);
        }
    }

    Ok(())
}

// Include UKC Calculator from previous code
// (The UKCCalculator and related types should be defined here or imported)

// Re-export UKC types
pub use crate::ukc::*;

// Module declarations
pub mod ukc {
    // Include the UKC Calculator code here
}
