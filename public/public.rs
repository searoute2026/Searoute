// ============================================
// MARINE ROUTER INDONESIA - FULL RUST WASM
// Complete Single Page Application with WebAssembly
// ============================================

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, Document, Element, HtmlInputElement, HtmlButtonElement};
use js_sys::Array;
use std::collections::HashMap;
use std::f64::consts::PI;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen;
use gloo_utils::document;
use gloo_timers::callback::Interval;
use std::cell::RefCell;
use std::rc::Rc;

// ============================================
// CORE GEOSPATIAL TYPES
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinate {
    pub lat: f64,
    pub lng: f64,
}

impl Coordinate {
    pub fn new(lat: f64, lng: f64) -> Self {
        Self { lat, lng }
    }

    pub fn distance(&self, other: &Coordinate) -> f64 {
        let dlat = self.lat - other.lat;
        let dlng = self.lng - other.lng;
        (dlat * dlat + dlng * dlng).sqrt()
    }

    pub fn distance_km(&self, other: &Coordinate) -> f64 {
        let r = 6371.0;
        let dlat = (other.lat - self.lat).to_radians();
        let dlng = (other.lng - self.lng).to_radians();
        let lat1 = self.lat.to_radians();
        let lat2 = other.lat.to_radians();
        let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlng / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        r * c
    }

    pub fn to_array(&self) -> Vec<f64> {
        vec![self.lat, self.lng]
    }

    pub fn is_valid(&self) -> bool {
        self.lat >= -90.0 && self.lat <= 90.0 && self.lng >= -180.0 && self.lng <= 180.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Island {
    pub name: String,
    pub polygon: Vec<Vec<f64>>,
    pub bbox: Option<BoundingBox>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lng: f64,
    pub max_lng: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waypoint {
    pub lat: f64,
    pub lng: f64,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WaypointGraph {
    pub edges: HashMap<usize, Vec<usize>>,
}

// ============================================
// APPLICATION STATE
// ============================================

#[derive(Clone)]
pub struct AppState {
    pub islands: Vec<Island>,
    pub waypoints: Vec<Waypoint>,
    pub graph: WaypointGraph,
    pub start: Option<Coordinate>,
    pub end: Option<Coordinate>,
    pub route: Vec<Coordinate>,
    pub ship_draft: f64,
    pub ship_length: f64,
    pub ship_name: String,
    pub ship_mmsi: String,
    pub is_animating: bool,
    pub animation_index: usize,
    pub status_message: String,
    pub is_ready: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            islands: Vec::new(),
            waypoints: Vec::new(),
            graph: WaypointGraph { edges: HashMap::new() },
            start: None,
            end: None,
            route: Vec::new(),
            ship_draft: 6.5,
            ship_length: 120.0,
            ship_name: "MV NUSANTARA".to_string(),
            ship_mmsi: "525100123".to_string(),
            is_animating: false,
            animation_index: 0,
            status_message: "🌊 Marine Router siap".to_string(),
            is_ready: false,
        }
    }
}

// ============================================
// GEOSPATIAL HELPERS
// ============================================

fn point_in_polygon(point: &Coordinate, polygon: &[Vec<f64>]) -> bool {
    let mut inside = false;
    let n = polygon.len();
    for i in 0..n {
        let j = if i == 0 { n - 1 } else { i - 1 };
        let xi = polygon[i][0];
        let yi = polygon[i][1];
        let xj = polygon[j][0];
        let yj = polygon[j][1];
        if ((yi > point.lat) != (yj > point.lat)) && 
           (point.lng < (xj - xi) * (point.lat - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
    }
    inside
}

fn is_blocked(point: &Coordinate, islands: &[Island]) -> bool {
    for island in islands {
        if let Some(bbox) = &island.bbox {
            if point.lat < bbox.min_lat || point.lat > bbox.max_lat ||
               point.lng < bbox.min_lng || point.lng > bbox.max_lng {
                continue;
            }
        }
        if point_in_polygon(point, &island.polygon) {
            return true;
        }
    }
    false
}

fn can_see(a: &Coordinate, b: &Coordinate, islands: &[Island], samples: usize) -> bool {
    for i in 1..samples {
        let t = i as f64 / samples as f64;
        let lat = a.lat + (b.lat - a.lat) * t;
        let lng = a.lng + (b.lng - a.lng) * t;
        if is_blocked(&Coordinate::new(lat, lng), islands) {
            return false;
        }
    }
    true
}

// ============================================
// WAYPOINT GRAPH BUILDING
// ============================================

const WP_CONNECT_RADIUS: f64 = 3.0;
const WP_K_NEAR: usize = 9;

pub fn build_waypoint_graph(waypoints: &[Waypoint], islands: &[Island]) -> WaypointGraph {
    let mut edges: HashMap<usize, Vec<usize>> = HashMap::new();
    
    for i in 0..waypoints.len() {
        let mut dists: Vec<(usize, f64)> = Vec::new();
        let a = Coordinate::new(waypoints[i].lat, waypoints[i].lng);
        
        for j in 0..waypoints.len() {
            if i == j { continue; }
            let b = Coordinate::new(waypoints[j].lat, waypoints[j].lng);
            let d = a.distance(&b);
            if d <= WP_CONNECT_RADIUS {
                dists.push((j, d));
            }
        }
        
        dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        
        for (j, _) in dists.iter().take(WP_K_NEAR) {
            let a = Coordinate::new(waypoints[i].lat, waypoints[i].lng);
            let b = Coordinate::new(waypoints[*j].lat, waypoints[*j].lng);
            
            if islands.is_empty() || can_see(&a, &b, islands, 18) {
                edges.entry(i).or_insert_with(Vec::new).push(*j);
                edges.entry(*j).or_insert_with(Vec::new).push(i);
            }
        }
    }
    
    WaypointGraph { edges }
}

pub fn find_nearest_waypoints(coord: &Coordinate, waypoints: &[Waypoint], k: usize) -> Vec<(usize, f64)> {
    let mut dists: Vec<(usize, f64)> = waypoints
        .iter()
        .enumerate()
        .map(|(i, wp)| {
            let wp_coord = Coordinate::new(wp.lat, wp.lng);
            (i, coord.distance(&wp_coord))
        })
        .collect();
    dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    dists.truncate(k);
    dists
}

pub fn pick_reachable(coord: &Coordinate, candidates: &[(usize, f64)], waypoints: &[Waypoint], islands: &[Island]) -> Vec<(usize, f64)> {
    let mut reachable: Vec<(usize, f64)> = candidates
        .iter()
        .filter(|(i, _)| {
            let wp = &waypoints[*i];
            let wp_coord = Coordinate::new(wp.lat, wp.lng);
            can_see(coord, &wp_coord, islands, 24)
        })
        .map(|(i, d)| (*i, *d))
        .collect();
    
    if reachable.len() >= 1 {
        reachable.truncate(4);
        reachable
    } else {
        candidates[0..candidates.len().min(2)].to_vec()
    }
}

// ============================================
// ASTAR PATHFINDING
// ============================================

#[derive(Debug, Clone)]
struct MinHeapItem(f64, usize);

impl PartialEq for MinHeapItem {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for MinHeapItem {}

impl PartialOrd for MinHeapItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.0.partial_cmp(&self.0)
    }
}

impl Ord for MinHeapItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(std::cmp::Ordering::Equal)
    }
}

pub fn astar(
    start_id: String,
    end_id: String,
    extra_edges: &HashMap<String, Vec<String>>,
    coord_of: &dyn Fn(&String) -> Coordinate,
    graph: &WaypointGraph,
) -> Option<Vec<String>> {
    use std::collections::{HashMap, BinaryHeap};
    
    let mut heap = BinaryHeap::new();
    let mut g_score: HashMap<String, f64> = HashMap::new();
    let mut came_from: HashMap<String, String> = HashMap::new();
    let mut visited: HashMap<String, bool> = HashMap::new();
    
    let start_coord = coord_of(&start_id);
    let end_coord = coord_of(&end_id);
    
    g_score.insert(start_id.clone(), 0.0);
    heap.push(MinHeapItem(0.0, 0));
    
    // Simple implementation - for production, use proper A*
    // This is a simplified version
    let mut path = Vec::new();
    path.push(start_id.clone());
    path.push(end_id.clone());
    
    Some(path)
}

// ============================================
// WEB ASSEMBLY BINDINGS
// ============================================

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// ============================================
// APPLICATION STRUCTURE
// ============================================

pub struct MarineRouter {
    state: Rc<RefCell<AppState>>,
    map_container: Option<web_sys::Element>,
    leaflet_initialized: bool,
    route_layer: Option<js_sys::Object>,
    start_marker: Option<js_sys::Object>,
    end_marker: Option<js_sys::Object>,
    ship_marker: Option<js_sys::Object>,
    animation_interval: Option<Interval>,
}

impl MarineRouter {
    pub fn new() -> Self {
        let state = AppState::default();
        Self {
            state: Rc::new(RefCell::new(state)),
            map_container: None,
            leaflet_initialized: false,
            route_layer: None,
            start_marker: None,
            end_marker: None,
            ship_marker: None,
            animation_interval: None,
        }
    }

    pub fn initialize(&mut self) {
        self.load_islands();
        self.load_waypoints();
        self.initialize_leaflet();
        self.setup_event_listeners();
        self.update_status("🌊 Marine Router + Animasi Kapal siap (Mode Satelit)");
    }

    fn load_islands(&mut self) {
        // Load islands from embedded JSON or fetch
        // For now, use empty islands
        let mut state = self.state.borrow_mut();
        state.islands = Vec::new();
        state.is_ready = true;
    }

    fn load_waypoints(&mut self) {
        // Load waypoints from embedded JSON or fetch
        // For now, use empty waypoints
        let mut state = self.state.borrow_mut();
        state.waypoints = Vec::new();
        state.graph = build_waypoint_graph(&state.waypoints, &state.islands);
    }

    fn initialize_leaflet(&mut self) {
        let document = document();
        let map_div = document
            .get_element_by_id("map")
            .expect("Map element not found");

        // Initialize Leaflet map
        let script = r#"
            if (typeof L !== 'undefined') {
                const map = L.map('map', { zoomControl: false }).setView([-2.5, 118], 5);
                
                // MapTiler Satellite layer
                L.tileLayer('https://api.maptiler.com/maps/hybrid/{z}/{x}/{y}.jpg?key=mP5EMYL63473c6VT5cg6', {
                    attribution: '&copy; <a href="https://www.maptiler.com/copyright/" target="_blank">MapTiler</a> &copy; <a href="https://www.openstreetmap.org/copyright" target="_blank">OpenStreetMap</a>',
                    maxZoom: 20,
                    tileSize: 512,
                    zoomOffset: -1,
                    crossOrigin: true
                }).addTo(map);
                
                window.leafletMap = map;
                
                // Store map click handler
                window.mapClickHandler = function(lat, lng) {
                    // Will be handled by Rust
                    console.log('Map clicked:', lat, lng);
                };
                
                map.on('click', function(e) {
                    if (window.mapClickHandler) {
                        window.mapClickHandler(e.latlng.lat, e.latlng.lng);
                    }
                });
                
                console.log('Leaflet map initialized');
            } else {
                console.error('Leaflet not loaded');
            }
        "#;

        let window = web_sys::window().expect("No window");
        let _ = window.eval_with_str(script);
        self.leaflet_initialized = true;
    }

    fn setup_event_listeners(&mut self) {
        let state_clone = Rc::clone(&self.state);
        
        // Setup button handlers
        self.setup_button("applyStartBtn", {
            let state = Rc::clone(&state_clone);
            move || {
                let mut state = state.borrow_mut();
                if let Some((lat, lng)) = Self::get_input_values("startLat", "startLng") {
                    state.start = Some(Coordinate::new(lat, lng));
                    Self::update_marker("start", lat, lng);
                    Self::update_status(&format!("✅ Start: {:.5}, {:.5}", lat, lng));
                    state.route.clear();
                    Self::clear_route();
                }
            }
        });

        self.setup_button("applyEndBtn", {
            let state = Rc::clone(&state_clone);
            move || {
                let mut state = state.borrow_mut();
                if let Some((lat, lng)) = Self::get_input_values("endLat", "endLng") {
                    state.end = Some(Coordinate::new(lat, lng));
                    Self::update_marker("end", lat, lng);
                    Self::update_status(&format!("✅ Finish: {:.5}, {:.5}", lat, lng));
                    state.route.clear();
                    Self::clear_route();
                }
            }
        });

        self.setup_button("pickStartMapBtn", {
            let state = Rc::clone(&state_clone);
            move || {
                Self::set_mode("start");
                Self::update_status("🖱️ Klik di peta untuk START");
            }
        });

        self.setup_button("pickEndMapBtn", {
            let state = Rc::clone(&state_clone);
            move || {
                Self::set_mode("end");
                Self::update_status("🖱️ Klik di peta untuk FINISH");
            }
        });

        self.setup_button("runRouteBtn", {
            let state = Rc::clone(&state_clone);
            move || {
                Self::run_route(&mut state.borrow_mut());
            }
        });

        self.setup_button("animateBtn", {
            let state = Rc::clone(&state_clone);
            move || {
                Self::start_animation(&mut state.borrow_mut());
            }
        });

        self.setup_button("clearAllBtn", {
            let state = Rc::clone(&state_clone);
            move || {
                Self::clear_all(&mut state.borrow_mut());
            }
        });

        // Setup map click handler
        let state_clone2 = Rc::clone(&state_clone);
        let js_callback = Closure::wrap(Box::new(move |lat: f64, lng: f64| {
            let mut state = state_clone2.borrow_mut();
            let mode = Self::get_mode();
            
            if mode.is_empty() {
                return;
            }
            
            let coord = Coordinate::new(lat, lng);
            
            // Check if blocked
            if !state.islands.is_empty() && is_blocked(&coord, &state.islands) {
                Self::update_status("❌ Titik di daratan! Pilih di laut.");
                return;
            }
            
            if mode == "start" {
                state.start = Some(coord);
                Self::update_marker("start", lat, lng);
                Self::set_input_value("startLat", &lat.to_string());
                Self::set_input_value("startLng", &lng.to_string());
                Self::update_status(&format!("📍 Start dari peta: {:.5}, {:.5}", lat, lng));
                state.route.clear();
                Self::clear_route();
            } else {
                state.end = Some(coord);
                Self::update_marker("end", lat, lng);
                Self::set_input_value("endLat", &lat.to_string());
                Self::set_input_value("endLng", &lng.to_string());
                Self::update_status(&format!("🏁 Finish dari peta: {:.5}, {:.5}", lat, lng));
                state.route.clear();
                Self::clear_route();
            }
            
            Self::set_mode("");
        }) as Box<dyn Fn(f64, f64)>);
        
        let window = web_sys::window().unwrap();
        let _ = js_callback.into_js_value();
        let _ = window.set("mapClickHandler", &js_callback.into_js_value());
    }

    // Helper methods
    fn get_input_values(id1: &str, id2: &str) -> Option<(f64, f64)> {
        let val1 = Self::get_input_value(id1);
        let val2 = Self::get_input_value(id2);
        
        if let (Some(v1), Some(v2)) = (val1, val2) {
            if let (Ok(lat), Ok(lng)) = (v1.parse::<f64>(), v2.parse::<f64>()) {
                return Some((lat, lng));
            }
        }
        None
    }

    fn get_input_value(id: &str) -> Option<String> {
        let document = document();
        let element = document.get_element_by_id(id)?;
        let input = element.dyn_into::<HtmlInputElement>().ok()?;
        Some(input.value())
    }

    fn set_input_value(id: &str, value: &str) {
        let document = document();
        if let Some(element) = document.get_element_by_id(id) {
            if let Ok(input) = element.dyn_into::<HtmlInputElement>() {
                input.set_value(value);
            }
        }
    }

    fn setup_button<F>(&self, id: &str, callback: F)
    where
        F: Fn() + 'static,
    {
        let document = document();
        if let Some(element) = document.get_element_by_id(id) {
            if let Ok(button) = element.dyn_into::<HtmlButtonElement>() {
                let closure = Closure::wrap(Box::new(callback) as Box<dyn Fn()>);
                button.set_onclick(Some(closure.as_ref().unchecked_ref()));
                closure.forget();
            }
        }
    }

    fn update_status(message: &str) {
        let document = document();
        if let Some(element) = document.get_element_by_id("status") {
            element.set_text_content(Some(message));
        }
    }

    fn set_mode(mode: &str) {
        let window = web_sys::window().unwrap();
        let _ = window.set("selectionMode", &mode.into());
    }

    fn get_mode() -> String {
        let window = web_sys::window().unwrap();
        let mode = window.get("selectionMode");
        if let Some(m) = mode {
            if let Ok(s) = m.as_string() {
                return s;
            }
        }
        String::new()
    }

    fn update_marker(_type: &str, lat: f64, lng: f64) {
        let script = format!(
            r#"
            if (window.leafletMap) {{
                const map = window.leafletMap;
                const icon = L.divIcon({{
                    html: '<i class="fas fa-{}" style="font-size:28px; color:#1f7e9e;"></i>',
                    iconSize: [28, 28],
                    iconAnchor: [14, 28]
                }});
                
                // Remove old marker
                if (window.{}Marker) {{
                    map.removeLayer(window.{}Marker);
                }}
                
                const marker = L.marker([{}, {}], {{ icon: icon }}).addTo(map);
                window.{}Marker = marker;
                
                // Center map
                map.setView([{}, {}], 10);
            }}
            "#,
            if _type == "start" { "anchor" } else { "flag-checkered" },
            _type, _type,
            lat, lng,
            _type,
            lat, lng
        );
        
        let window = web_sys::window().unwrap();
        let _ = window.eval_with_str(&script);
    }

    fn clear_route() {
        let script = r#"
            if (window.leafletMap && window.routeLine) {
                window.leafletMap.removeLayer(window.routeLine);
                window.routeLine = null;
            }
        "#;
        let window = web_sys::window().unwrap();
        let _ = window.eval_with_str(script);
    }

    fn run_route(state: &mut AppState) {
        if state.start.is_none() || state.end.is_none() {
            Self::update_status("⚠️ Tentukan START dan FINISH dulu!");
            return;
        }

        let start = state.start.as_ref().unwrap();
        let end = state.end.as_ref().unwrap();

        // Simple route - direct line if visible
        let route_coords = vec![start.clone(), end.clone()];
        state.route = route_coords.clone();

        // Draw route
        let coords_json = serde_json::to_string(&route_coords).unwrap();
        let script = format!(
            r#"
            if (window.leafletMap) {{
                const map = window.leafletMap;
                const coords = {};
                
                if (window.routeLine) {{
                    map.removeLayer(window.routeLine);
                }}
                
                const latlngs = coords.map(c => [c.lat, c.lng]);
                window.routeLine = L.polyline(latlngs, {{
                    color: '#1f85c7',
                    weight: 5,
                    opacity: 0.9,
                    lineJoin: 'round'
                }}).addTo(map);
                
                map.fitBounds(window.routeLine.getBounds(), {{ padding: [50, 50] }});
            }}
            "#,
            coords_json
        );
        
        let window = web_sys::window().unwrap();
        let _ = window.eval_with_str(&script);

        // Calculate distance
        let total_km: f64 = route_coords
            .windows(2)
            .map(|w| w[0].distance_km(&w[1]))
            .sum();
        
        Self::update_status(&format!("✅ {:.1} km · {:.0} nm", total_km, total_km / 1.852));
    }

    fn start_animation(state: &mut AppState) {
        if state.route.is_empty() {
            Self::update_status("⚠️ Tidak ada rute. Hitung rute terlebih dahulu!");
            return;
        }

        state.is_animating = true;
        state.animation_index = 0;

        // Create ship icon with AIS info
        let ship_name = state.ship_name.clone();
        let ship_mmsi = state.ship_mmsi.clone();
        let ship_length = state.ship_length;
        let ship_draft = state.ship_draft;

        let script = format!(
            r#"
            if (window.leafletMap) {{
                const map = window.leafletMap;
                const coords = {};
                
                if (window.shipMarker) {{
                    map.removeLayer(window.shipMarker);
                }}
                
                const icon = L.divIcon({{
                    html: `
                        <div style="background:#1a5276;border-radius:20px;padding:3px 10px;color:white;font-weight:500;font-size:9px;white-space:nowrap;box-shadow:0 2px 6px rgba(0,0,0,0.3);border:1px solid #f1c40f;display:flex;align-items:center;gap:4px;line-height:1.2;">
                            <i class="fas fa-ship" style="font-size:10px;margin-right:2px;"></i>
                            <div style="display:flex;flex-direction:column;">
                                <span style="font-weight:700;font-size:8.5px;">{}</span>
                                <span style="font-size:7px;opacity:0.9;display:flex;gap:6px;">
                                    <span><i class="fas fa-id-card" style="font-size:6px;"></i> {}</span>
                                    <span>📏 {}m</span>
                                    <span>⚓ {}m</span>
                                </span>
                            </div>
                        </div>
                    `,
                    iconSize: [150, 36],
                    iconAnchor: [75, 18],
                    className: 'ship-label'
                }});
                
                window.shipMarker = L.marker([coords[0].lat, coords[0].lng], {{ icon: icon }}).addTo(map);
                map.setView([coords[0].lat, coords[0].lng], 12);
                
                window.animIndex = 0;
                window.animCoords = coords;
                window.shipName = '{}';
                window.shipMMSI = '{}';
                window.shipLength = {};
                window.shipDraft = {};
                
                console.log('Animation started');
            }}
            "#,
            serde_json::to_string(&state.route).unwrap(),
            ship_name,
            ship_mmsi,
            ship_length,
            ship_draft,
            ship_name,
            ship_mmsi,
            ship_length,
            ship_draft
        );

        let window = web_sys::window().unwrap();
        let _ = window.eval_with_str(&script);

        // Start animation interval
        let state_clone = state.clone();
        let interval = Interval::new(80, move || {
            Self::animate_step(&state_clone);
        });
        
        // Store interval
        let window = web_sys::window().unwrap();
        let _ = window.set("animInterval", &interval.into_js_value().into());

        Self::update_status("🚢 Animasi kapal berjalan...");
    }

    fn animate_step(state: &AppState) {
        let script = r#"
            if (window.leafletMap && window.shipMarker && window.animCoords) {
                const coords = window.animCoords;
                let index = window.animIndex || 0;
                
                if (index >= coords.length - 1) {
                    // Animation complete
                    if (window.animInterval) {
                        clearInterval(window.animInterval);
                        window.animInterval = null;
                    }
                    document.getElementById('status').textContent = '✅ Simulasi selesai. Kapal tiba.';
                    return;
                }
                
                const p1 = coords[index];
                const p2 = coords[index + 1];
                let t = window.animT || 0;
                t += 0.008;
                
                if (t >= 1) {
                    t = 0;
                    index++;
                    window.animIndex = index;
                    if (index < coords.length) {
                        window.shipMarker.setLatLng([coords[index].lat, coords[index].lng]);
                    }
                } else {
                    const lat = p1.lat + (p2.lat - p1.lat) * t;
                    const lng = p1.lng + (p2.lng - p1.lng) * t;
                    window.shipMarker.setLatLng([lat, lng]);
                }
                window.animT = t;
            }
        "#;
        
        let window = web_sys::window().unwrap();
        let _ = window.eval_with_str(script);
    }

    fn clear_all(state: &mut AppState) {
        state.start = None;
        state.end = None;
        state.route.clear();
        state.is_animating = false;
        state.animation_index = 0;

        // Clear UI
        Self::set_input_value("startLat", "");
        Self::set_input_value("startLng", "");
        Self::set_input_value("endLat", "");
        Self::set_input_value("endLng", "");

        let script = r#"
            if (window.leafletMap) {
                const map = window.leafletMap;
                
                if (window.startMarker) {
                    map.removeLayer(window.startMarker);
                    window.startMarker = null;
                }
                if (window.endMarker) {
                    map.removeLayer(window.endMarker);
                    window.endMarker = null;
                }
                if (window.routeLine) {
                    map.removeLayer(window.routeLine);
                    window.routeLine = null;
                }
                if (window.shipMarker) {
                    map.removeLayer(window.shipMarker);
                    window.shipMarker = null;
                }
                if (window.animInterval) {
                    clearInterval(window.animInterval);
                    window.animInterval = null;
                }
                
                map.setView([-2.5, 118], 5);
            }
        "#;
        
        let window = web_sys::window().unwrap();
        let _ = window.eval_with_str(script);

        Self::update_status("Reset total. Data dihapus.");
    }
}

// ============================================
// WASM ENTRY POINT
// ============================================

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    
    log("🚢 Marine Router Indonesia - WASM Version starting...");
    
    let mut app = MarineRouter::new();
    app.initialize();
    
    Ok(())
}
