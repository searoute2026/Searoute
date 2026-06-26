// ============================================
// UKC NAVIGATOR WITH MAPTILELAYER
// Complete Navigation System with Map Tiles
// Cloudflare Workers Version
// ============================================

use worker::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

// ============================================
// MAP TILE SYSTEM
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapTile {
    pub z: u32,           // Zoom level
    pub x: u32,           // Tile X coordinate
    pub y: u32,           // Tile Y coordinate
    pub data: Vec<u8>,    // Tile image data (PNG/JPEG)
    pub content_type: String,
    pub etag: String,
    pub last_modified: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileRequest {
    pub z: u32,
    pub x: u32,
    pub y: u32,
    pub format: TileFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TileFormat {
    Png,
    Jpeg,
    Webp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileLayer {
    pub id: String,
    pub name: String,
    pub url_template: String,
    pub attribution: String,
    pub min_zoom: u32,
    pub max_zoom: u32,
    pub tile_size: u32,
    pub visible: bool,
    pub opacity: f64,
}

impl Default for TileLayer {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: "OpenStreetMap".to_string(),
            url_template: "https://tile.openstreetmap.org/{z}/{x}/{y}.png".to_string(),
            attribution: "© OpenStreetMap contributors".to_string(),
            min_zoom: 1,
            max_zoom: 19,
            tile_size: 256,
            visible: true,
            opacity: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapOverlay {
    pub id: String,
    pub name: String,
    pub coordinates: Vec<Coordinate>,
    pub style: OverlayStyle,
    pub data: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayStyle {
    pub color: String,
    pub width: f64,
    pub opacity: f64,
    pub fill_color: Option<String>,
    pub dash_array: Option<Vec<f64>>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapView {
    pub center: Coordinate,
    pub zoom: u32,
    pub bearing: f64,
    pub pitch: f64,
    pub bounds: Option<BoundingBox>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapResponse {
    pub tile: Option<MapTile>,
    pub layers: Vec<TileLayer>,
    pub overlays: Vec<MapOverlay>,
    pub view: MapView,
    pub timestamp: u64,
}

// ============================================
// TILE CACHE
// ============================================

pub struct TileCache {
    tiles: Arc<Mutex<HashMap<String, MapTile>>>,
    max_size: usize,
}

impl TileCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            tiles: Arc::new(Mutex::new(HashMap::new())),
            max_size,
        }
    }

    pub fn tile_key(z: u32, x: u32, y: u32) -> String {
        format!("{}/{}/{}.png", z, x, y)
    }

    pub async fn get(&self, z: u32, x: u32, y: u32) -> Option<MapTile> {
        let key = Self::tile_key(z, x, y);
        let cache = self.tiles.lock().await;
        cache.get(&key).cloned()
    }

    pub async fn put(&self, tile: MapTile) {
        let key = Self::tile_key(tile.z, tile.x, tile.y);
        let mut cache = self.tiles.lock().await;
        
        // If cache is full, remove oldest entries
        if cache.len() >= self.max_size {
            let keys: Vec<String> = cache.keys().take(self.max_size / 2).cloned().collect();
            for key in keys {
                cache.remove(&key);
            }
        }
        
        cache.insert(key, tile);
    }

    pub async fn clear(&self) {
        let mut cache = self.tiles.lock().await;
        cache.clear();
    }

    pub async fn size(&self) -> usize {
        let cache = self.tiles.lock().await;
        cache.len()
    }
}

// ============================================
// TILE GENERATOR
// ============================================

pub struct TileGenerator {
    cache: Arc<TileCache>,
    overlays: Arc<Mutex<Vec<MapOverlay>>>,
}

impl TileGenerator {
    pub fn new(cache: Arc<TileCache>) -> Self {
        Self {
            cache,
            overlays: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Generate a tile with overlays
    pub async fn generate_tile(&self, z: u32, x: u32, y: u32) -> Result<MapTile, String> {
        // Check cache first
        if let Some(tile) = self.cache.get(z, x, y).await {
            return Ok(tile);
        }

        // Generate tile data (simplified - in production, fetch from upstream or generate)
        let tile_data = self.render_tile(z, x, y).await?;
        
        let tile = MapTile {
            z,
            x,
            y,
            data: tile_data,
            content_type: "image/png".to_string(),
            etag: format!("{}/{}/{}", z, x, y),
            last_modified: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // Store in cache
        self.cache.put(tile.clone()).await;
        Ok(tile)
    }

    async fn render_tile(&self, z: u32, x: u32, y: u32) -> Result<Vec<u8>, String> {
        // In production, this would generate actual PNG tiles
        // For now, we'll create a simple SVG representation
        
        let overlays = self.overlays.lock().await;
        
        // Convert tile coordinates to geographic bounds
        let bounds = self.tile_to_bounds(z, x, y);
        
        // Generate SVG tile with overlays
        let mut svg = String::new();
        svg.push_str(&format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256">"#));
        
        // Background
        svg.push_str(r#"<rect width="256" height="256" fill="#f0f0f0"/>"#);
        
        // Grid
        svg.push_str(r#"<path d="M0 128 L256 128 M128 0 L128 256" stroke="#cccccc" stroke-width="0.5" opacity="0.5"/>"#);
        
        // Label with tile info
        svg.push_str(&format!(
            r#"<text x="128" y="128" font-family="Arial" font-size="14" fill="#666" text-anchor="middle" dominant-baseline="middle">
            Zoom: {}, X: {}, Y: {}
            </text>"#,
            z, x, y
        ));

        // Add overlays
        for overlay in overlays.iter() {
            if overlay.coordinates.len() >= 2 {
                // Draw line overlay
                let points: Vec<String> = overlay.coordinates
                    .iter()
                    .map(|coord| {
                        let (px, py) = self.geo_to_tile_pixel(coord, &bounds);
                        format!("{},{}", px, py)
                    })
                    .collect();

                svg.push_str(&format!(
                    r#"<polyline points="{}" stroke="{}" stroke-width="{}" opacity="{}" fill="none"/>"#,
                    points.join(" "),
                    overlay.style.color,
                    overlay.style.width,
                    overlay.style.opacity
                ));

                // Add label if present
                if let Some(label) = &overlay.style.label {
                    if let Some(first) = overlay.coordinates.first() {
                        let (px, py) = self.geo_to_tile_pixel(first, &bounds);
                        svg.push_str(&format!(
                            r#"<text x="{}" y="{}" font-family="Arial" font-size="10" fill="{}" text-anchor="middle">{}</text>"#,
                            px, py - 10, overlay.style.color, label
                        ));
                    }
                }
            }
        }

        svg.push_str(r#"</svg>"#);

        // Convert to PNG (simplified - in production use actual image library)
        // For now, return SVG as bytes
        Ok(svg.into_bytes())
    }

    fn tile_to_bounds(&self, z: u32, x: u32, y: u32) -> BoundingBox {
        let n = 2.0_f64.powi(z as i32);
        let lon_deg = |x: u32| -> f64 {
            (x as f64 / n) * 360.0 - 180.0
        };
        let lat_deg = |y: u32| -> f64 {
            let lat_rad = f64::atan(f64::sinh(std::f64::consts::PI * (1.0 - 2.0 * y as f64 / n)));
            lat_rad.to_degrees()
        };

        BoundingBox {
            min_lat: lat_deg(y + 1),
            max_lat: lat_deg(y),
            min_lng: lon_deg(x),
            max_lng: lon_deg(x + 1),
        }
    }

    fn geo_to_tile_pixel(&self, coord: &Coordinate, bounds: &BoundingBox) -> (f64, f64) {
        let px = (coord.longitude - bounds.min_lng) / (bounds.max_lng - bounds.min_lng) * 256.0;
        let py = (bounds.max_lat - coord.latitude) / (bounds.max_lat - bounds.min_lat) * 256.0;
        (px.clamp(0.0, 256.0), py.clamp(0.0, 256.0))
    }

    pub async fn add_overlay(&self, overlay: MapOverlay) {
        let mut overlays = self.overlays.lock().await;
        overlays.push(overlay);
    }

    pub async fn remove_overlay(&self, id: &str) {
        let mut overlays = self.overlays.lock().await;
        overlays.retain(|o| o.id != id);
    }

    pub async fn clear_overlays(&self) {
        let mut overlays = self.overlays.lock().await;
        overlays.clear();
    }
}

// ============================================
// NAVIGATOR SYSTEM
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigatorState {
    pub view: MapView,
    pub layers: Vec<TileLayer>,
    pub active_layer: Option<String>,
    pub waypoints: Vec<Waypoint>,
    pub active_route: Option<RouteWithWaypoints>,
    pub selected_waypoint: Option<String>,
    pub show_ukc: bool,
    pub show_depth: bool,
    pub show_route: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigatorAction {
    pub action_type: NavigatorActionType,
    pub data: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NavigatorActionType {
    Pan,
    Zoom,
    Rotate,
    SelectWaypoint,
    AddWaypoint,
    RemoveWaypoint,
    CreateRoute,
    ToggleLayer,
    ToggleUKC,
    ToggleDepth,
    CalculateUKC,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigatorResponse {
    pub state: NavigatorState,
    pub action_result: Value,
    pub timestamp: u64,
}

// ============================================
// WORKER STATE
// ============================================

pub struct AppState {
    groq_api_key: String,
    tile_cache: Arc<TileCache>,
    tile_generator: Arc<TileGenerator>,
    waypoint_store: Arc<Mutex<WaypointStore>>,
    navigator_state: Arc<Mutex<NavigatorState>>,
    calculator: UKCCalculator,
}

impl AppState {
    pub fn new(env: &Env) -> Self {
        let groq_api_key = env.var("GROQ_API_KEY")
            .map(|v| v.to_string())
            .unwrap_or_else(|_| "".to_string());

        let cache = Arc::new(TileCache::new(1000));
        let generator = Arc::new(TileGenerator::new(cache.clone()));
        
        let waypoint_store = Arc::new(Mutex::new(WaypointStore::new()));
        
        let navigator_state = Arc::new(Mutex::new(NavigatorState {
            view: MapView {
                center: Coordinate::new(-5.0, 106.0),
                zoom: 8,
                bearing: 0.0,
                pitch: 0.0,
                bounds: None,
            },
            layers: vec![
                TileLayer {
                    id: "osm".to_string(),
                    name: "OpenStreetMap".to_string(),
                    url_template: "https://tile.openstreetmap.org/{z}/{x}/{y}.png".to_string(),
                    attribution: "© OpenStreetMap contributors".to_string(),
                    min_zoom: 1,
                    max_zoom: 19,
                    tile_size: 256,
                    visible: true,
                    opacity: 1.0,
                },
                TileLayer {
                    id: "satellite".to_string(),
                    name: "Satellite".to_string(),
                    url_template: "https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}".to_string(),
                    attribution: "© Esri".to_string(),
                    min_zoom: 1,
                    max_zoom: 19,
                    tile_size: 256,
                    visible: false,
                    opacity: 0.8,
                },
                TileLayer {
                    id: "nautical".to_string(),
                    name: "Nautical Chart".to_string(),
                    url_template: "https://tiles.maritime.gov/nautical/{z}/{x}/{y}.png".to_string(),
                    attribution: "© Maritime Navigation".to_string(),
                    min_zoom: 1,
                    max_zoom: 18,
                    tile_size: 256,
                    visible: false,
                    opacity: 0.9,
                },
            ],
            active_layer: Some("osm".to_string()),
            waypoints: Vec::new(),
            active_route: None,
            selected_waypoint: None,
            show_ukc: true,
            show_depth: true,
            show_route: true,
        }));

        Self {
            groq_api_key,
            tile_cache: cache,
            tile_generator: generator,
            waypoint_store,
            navigator_state,
            calculator: UKCCalculator::new(),
        }
    }
}

// ============================================
// WORKER HANDLERS
// ============================================

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Handle CORS
    if req.method() == Method::Options {
        let mut response = Response::empty()?;
        response.headers_mut().set("Access-Control-Allow-Origin", "*")?;
        response.headers_mut().set("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")?;
        response.headers_mut().set("Access-Control-Allow-Headers", "Content-Type, Authorization")?;
        return Ok(response);
    }

    let state = Arc::new(AppState::new(&env));
    let router = Router::new();

    router
        // ============================================
        // TILE ENDPOINTS
        // ============================================

        // Get tile with overlays
        .get("/tiles/:z/:x/:y", |req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            
            let z: u32 = req.param("z").unwrap().parse().unwrap_or(0);
            let x: u32 = req.param("x").unwrap().parse().unwrap_or(0);
            let y: u32 = req.param("y").unwrap().parse().unwrap_or(0);

            match state.tile_generator.generate_tile(z, x, y).await {
                Ok(tile) => {
                    let mut response = Response::from_bytes(tile.data)?;
                    response.headers_mut().set("Content-Type", "image/png")?;
                    response.headers_mut().set("Cache-Control", "public, max-age=86400")?;
                    response.headers_mut().set("ETag", &tile.etag)?;
                    Ok(response)
                }
                Err(e) => {
                    Response::error(format!("Failed to generate tile: {}", e), 500)
                }
            }
        })

        // Get tile from upstream source
        .get("/tiles/proxy/:layer/:z/:x/:y", |req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            
            let layer = req.param("layer").unwrap();
            let z: u32 = req.param("z").unwrap().parse().unwrap_or(0);
            let x: u32 = req.param("x").unwrap().parse().unwrap_or(0);
            let y: u32 = req.param("y").unwrap().parse().unwrap_or(0);

            // Find layer configuration
            let nav_state = state.navigator_state.lock().await;
            let tile_layer = nav_state.layers.iter().find(|l| l.id == layer).cloned();

            if let Some(layer_config) = tile_layer {
                let url = layer_config.url_template
                    .replace("{z}", &z.to_string())
                    .replace("{x}", &x.to_string())
                    .replace("{y}", &y.to_string());

                // Proxy request to tile server
                match reqwest::get(&url).await {
                    Ok(response) => {
                        if response.status().is_success() {
                            let bytes = response.bytes().await.unwrap_or_default();
                            let mut resp = Response::from_bytes(bytes.to_vec())?;
                            resp.headers_mut().set("Content-Type", "image/png")?;
                            resp.headers_mut().set("Cache-Control", "public, max-age=86400")?;
                            return Ok(resp);
                        }
                    }
                    Err(_) => {}
                }
            }

            // Fallback: generate tile
            match state.tile_generator.generate_tile(z, x, y).await {
                Ok(tile) => {
                    let mut response = Response::from_bytes(tile.data)?;
                    response.headers_mut().set("Content-Type", "image/png")?;
                    response.headers_mut().set("Cache-Control", "public, max-age=86400")?;
                    Ok(response)
                }
                Err(e) => {
                    Response::error(format!("Failed to generate tile: {}", e), 500)
                }
            }
        })

        // Get tile layers
        .get("/api/layers", |_req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            let nav_state = state.navigator_state.lock().await;
            
            json_response(&json!({
                "status": "success",
                "layers": nav_state.layers,
                "active_layer": nav_state.active_layer
            }), 200)
        })

        // Toggle layer
        .post("/api/layers/:id/toggle", |req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            let id = req.param("id").unwrap();
            
            let mut nav_state = state.navigator_state.lock().await;
            if let Some(layer) = nav_state.layers.iter_mut().find(|l| l.id == id) {
                layer.visible = !layer.visible;
                json_response(&json!({
                    "status": "success",
                    "layer": layer
                }), 200)
            } else {
                json_response(&json!({
                    "status": "error",
                    "message": format!("Layer {} not found", id)
                }), 404)
            }
        })

        // Set active layer
        .post("/api/layers/active", |req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            
            let body = req.text().await?;
            let data: HashMap<String, String> = serde_json::from_str(&body).unwrap_or_default();
            let layer_id = data.get("layer_id").unwrap_or(&"osm".to_string());

            let mut nav_state = state.navigator_state.lock().await;
            nav_state.active_layer = Some(layer_id.clone());

            json_response(&json!({
                "status": "success",
                "active_layer": layer_id
            }), 200)
        })

        // ============================================
        // NAVIGATOR ENDPOINTS
        // ============================================

        // Get navigator state
        .get("/api/navigator/state", |_req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            let nav_state = state.navigator_state.lock().await;
            
            json_response(&json!({
                "status": "success",
                "state": *nav_state
            }), 200)
        })

        // Update navigator view
        .post("/api/navigator/view", |req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            
            let body = req.text().await?;
            let view: MapView = serde_json::from_str(&body).unwrap_or(MapView {
                center: Coordinate::new(-5.0, 106.0),
                zoom: 8,
                bearing: 0.0,
                pitch: 0.0,
                bounds: None,
            });

            let mut nav_state = state.navigator_state.lock().await;
            nav_state.view = view;

            json_response(&json!({
                "status": "success",
                "view": nav_state.view
            }), 200)
        })

        // Add overlay to map
        .post("/api/navigator/overlay", |req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            
            let body = req.text().await?;
            let overlay: MapOverlay = serde_json::from_str(&body).unwrap();

            state.tile_generator.add_overlay(overlay.clone()).await;

            json_response(&json!({
                "status": "success",
                "overlay": overlay
            }), 201)
        })

        // Remove overlay
        .delete("/api/navigator/overlay/:id", |req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            let id = req.param("id").unwrap();

            state.tile_generator.remove_overlay(&id).await;

            json_response(&json!({
                "status": "success",
                "message": format!("Overlay {} removed", id)
            }), 200)
        })

        // Clear all overlays
        .delete("/api/navigator/overlays", |_req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            state.tile_generator.clear_overlays().await;

            json_response(&json!({
                "status": "success",
                "message": "All overlays cleared"
            }), 200)
        })

        // ============================================
        // UKC ANALYSIS ON MAP
        // ============================================

        // Analyze area for UKC safety
        .post("/api/navigator/analyze-area", |req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            
            let body = req.text().await?;
            let data: HashMap<String, Value> = serde_json::from_str(&body).unwrap_or_default();
            
            let bounds: BoundingBox = match serde_json::from_value(data.get("bounds").unwrap().clone()) {
                Ok(b) => b,
                Err(_) => {
                    return json_response(&json!({
                        "status": "error",
                        "message": "Invalid bounds"
                    }), 400);
                }
            };

            let ship_params: UKCInput = match serde_json::from_value(data.get("ship_params").unwrap().clone()) {
                Ok(p) => p,
                Err(_) => {
                    return json_response(&json!({
                        "status": "error",
                        "message": "Invalid ship parameters"
                    }), 400);
                }
            };

            // Generate heatmap overlay of UKC safety
            let step = 0.1; // degrees
            let mut lat = bounds.min_lat;
            let mut safe_points = Vec::new();
            let mut unsafe_points = Vec::new();

            while lat <= bounds.max_lat {
                let mut lng = bounds.min_lng;
                while lng <= bounds.max_lng {
                    let coord = Coordinate::new(lat, lng);
                    let depth = 15.0 + 5.0 * (lat * 0.5).sin() + 3.0 * (lng * 0.3).cos();
                    
                    let mut input = ship_params.clone();
                    input.water_depth_available = depth;
                    
                    let result = state.calculator.calculate(&input);
                    
                    if result.is_safe {
                        safe_points.push(coord);
                    } else {
                        unsafe_points.push(coord);
                    }
                    
                    lng += step;
                }
                lat += step;
            }

            // Create safety overlay
            let overlay = MapOverlay {
                id: Uuid::new_v4().to_string(),
                name: "UKC Safety Analysis".to_string(),
                coordinates: safe_points,
                style: OverlayStyle {
                    color: "#27ae60".to_string(),
                    width: 2.0,
                    opacity: 0.6,
                    fill_color: Some("rgba(39, 174, 96, 0.2)".to_string()),
                    dash_array: None,
                    label: Some("Safe Area".to_string()),
                },
                data: {
                    let mut map = HashMap::new();
                    map.insert("type".to_string(), json!("ukc_safety"));
                    map.insert("safe_count".to_string(), json!(safe_points.len()));
                    map.insert("unsafe_count".to_string(), json!(unsafe_points.len()));
                    map
                },
            };

            state.tile_generator.add_overlay(overlay.clone()).await;

            json_response(&json!({
                "status": "success",
                "analysis": {
                    "total_points": safe_points.len() + unsafe_points.len(),
                    "safe_points": safe_points.len(),
                    "unsafe_points": unsafe_points.len(),
                    "safety_percentage": (safe_points.len() as f64 / (safe_points.len() + unsafe_points.len()) as f64 * 100.0).round_to_2(),
                },
                "overlay": overlay
            }), 200)
        })

        // ============================================
        // ROUTE VISUALIZATION
        // ============================================

        // Visualize route on map
        .post("/api/navigator/visualize-route", |req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            
            let body = req.text().await?;
            let route: RouteWithWaypoints = serde_json::from_str(&body).unwrap();

            // Create route overlay
            let coords: Vec<Coordinate> = route.waypoints.iter()
                .map(|wp| wp.waypoint.coordinate)
                .collect();

            let color = if route.overall_status == "SAFE" { "#27ae60" } else { "#e74c3c" };

            let route_overlay = MapOverlay {
                id: Uuid::new_v4().to_string(),
                name: route.name.clone(),
                coordinates: coords,
                style: OverlayStyle {
                    color: color.to_string(),
                    width: 4.0,
                    opacity: 0.8,
                    fill_color: None,
                    dash_array: None,
                    label: Some(route.name.clone()),
                },
                data: {
                    let mut map = HashMap::new();
                    map.insert("type".to_string(), json!("route"));
                    map.insert("distance".to_string(), json!(route.total_distance));
                    map.insert("status".to_string(), json!(route.overall_status));
                    map
                },
            };

            state.tile_generator.add_overlay(route_overlay.clone()).await;

            // Also add waypoint markers
            for wp in route.waypoints {
                let wp_overlay = MapOverlay {
                    id: Uuid::new_v4().to_string(),
                    name: wp.waypoint.name.clone(),
                    coordinates: vec![wp.waypoint.coordinate],
                    style: OverlayStyle {
                        color: if wp.safety_margin >= 0.0 { "#27ae60" } else { "#e74c3c" }.to_string(),
                        width: 8.0,
                        opacity: 1.0,
                        fill_color: Some("rgba(255, 255, 255, 0.5)".to_string()),
                        dash_array: None,
                        label: Some(format!("{} (UKC: {:.2}m)", wp.waypoint.name, wp.safety_margin)),
                    },
                    data: {
                        let mut map = HashMap::new();
                        map.insert("type".to_string(), json!("waypoint"));
                        map.insert("ukc".to_string(), json!(wp.ukc_analysis.ukc));
                        map.insert("safety_margin".to_string(), json!(wp.safety_margin));
                        map
                    },
                };
                state.tile_generator.add_overlay(wp_overlay).await;
            }

            // Update navigator state
            let mut nav_state = state.navigator_state.lock().await;
            nav_state.active_route = Some(route.clone());

            json_response(&json!({
                "status": "success",
                "route": route,
                "overlay_count": route.waypoints.len() + 1
            }), 200)
        })

        // ============================================
        // HTML MAP INTERFACE
        // ============================================

        // Serve interactive map HTML
        .get("/map", |_req, _ctx| {
            let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>UKC Navigator - Interactive Map</title>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css" />
    <script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"></script>
    <style>
        body { margin: 0; padding: 0; font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; }
        #map { height: 100vh; width: 100%; }
        #controls {
            position: absolute;
            top: 10px;
            right: 10px;
            background: rgba(255,255,255,0.95);
            padding: 15px;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.3);
            z-index: 1000;
            min-width: 250px;
            max-height: 90vh;
            overflow-y: auto;
        }
        #controls h3 {
            margin: 0 0 10px 0;
            color: #2c3e50;
        }
        #controls .control-group {
            margin-bottom: 10px;
        }
        #controls label {
            display: block;
            font-size: 12px;
            color: #7f8c8d;
            margin-bottom: 2px;
        }
        #controls input, #controls select {
            width: 100%;
            padding: 4px 6px;
            border: 1px solid #ddd;
            border-radius: 4px;
            font-size: 12px;
        }
        #controls button {
            width: 100%;
            padding: 6px;
            margin-top: 4px;
            background: #3498db;
            color: white;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-size: 12px;
        }
        #controls button:hover {
            background: #2980b9;
        }
        #controls button.danger {
            background: #e74c3c;
        }
        #controls button.danger:hover {
            background: #c0392b;
        }
        #controls button.success {
            background: #27ae60;
        }
        #controls button.success:hover {
            background: #229954;
        }
        #info {
            position: absolute;
            bottom: 30px;
            left: 10px;
            background: rgba(255,255,255,0.95);
            padding: 10px 15px;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.3);
            z-index: 1000;
            font-size: 12px;
            max-width: 300px;
        }
        #info .label { color: #7f8c8d; }
        #info .value { font-weight: bold; color: #2c3e50; }
        .status-safe { color: #27ae60; }
        .status-unsafe { color: #e74c3c; }
        .status-caution { color: #f39c12; }
        .legend {
            position: absolute;
            bottom: 30px;
            right: 10px;
            background: rgba(255,255,255,0.95);
            padding: 10px 15px;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.3);
            z-index: 1000;
            font-size: 11px;
        }
        .legend-item {
            display: flex;
            align-items: center;
            margin: 2px 0;
        }
        .legend-color {
            width: 20px;
            height: 4px;
            margin-right: 8px;
            border-radius: 2px;
        }
        .legend-color.circle {
            width: 12px;
            height: 12px;
            border-radius: 50%;
        }
        #loading {
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            z-index: 9999;
            background: rgba(0,0,0,0.8);
            color: white;
            padding: 20px 40px;
            border-radius: 8px;
            display: none;
        }
    </style>
</head>
<body>
    <div id="loading">Loading UKC Analysis...</div>
    <div id="map"></div>
    
    <div id="controls">
        <h3>🚢 UKC Navigator</h3>
        <div class="control-group">
            <label>Ship Draft (m)</label>
            <input type="number" id="shipDraft" value="10.5" step="0.1" min="1" max="30">
        </div>
        <div class="control-group">
            <label>Ship Length (m)</label>
            <input type="number" id="shipLength" value="180" step="1" min="10" max="500">
        </div>
        <div class="control-group">
            <label>Environment</label>
            <select id="environment">
                <option value="Coastal Water">Coastal Water</option>
                <option value="Port Approach">Port Approach</option>
            </select>
        </div>
        <div class="control-group">
            <button class="success" onclick="analyzeArea()">🔍 Analyze Area UKC</button>
        </div>
        <div class="control-group">
            <button onclick="clearOverlays()">🗑️ Clear Overlays</button>
        </div>
        <div class="control-group">
            <button onclick="toggleLayer('satellite')">🛰️ Toggle Satellite</button>
        </div>
        <div class="control-group">
            <label>
                <input type="checkbox" id="showUKC" checked onchange="toggleUKC()">
                Show UKC Analysis
            </label>
        </div>
        <div class="control-group" style="margin-top:10px;border-top:1px solid #eee;padding-top:10px;">
            <button class="success" onclick="loadSampleRoute()">🗺️ Load Sample Route</button>
        </div>
        <div style="font-size:11px;color:#95a5a6;margin-top:10px;">
            Click on map to add waypoint
        </div>
    </div>

    <div id="info">
        <div><span class="label">Status:</span> <span id="statusText" class="status-safe">Ready</span></div>
        <div><span class="label">Waypoints:</span> <span id="waypointCount" class="value">0</span></div>
        <div><span class="label">Min UKC:</span> <span id="minUKC" class="value">-</span></div>
        <div><span class="label">Safety Margin:</span> <span id="safetyMargin" class="value">-</span></div>
    </div>

    <div class="legend">
        <div class="legend-item">
            <div class="legend-color" style="background:#27ae60;"></div>
            <span>Safe (UKC > 1m)</span>
        </div>
        <div class="legend-item">
            <div class="legend-color" style="background:#f39c12;"></div>
            <span>Caution (UKC 0-1m)</span>
        </div>
        <div class="legend-item">
            <div class="legend-color" style="background:#e74c3c;"></div>
            <span>Unsafe (UKC < 0)</span>
        </div>
        <div class="legend-item">
            <div class="legend-color circle" style="background:#3498db;"></div>
            <span>Waypoint</span>
        </div>
    </div>

    <script>
        let map;
        let waypoints = [];
        let overlays = [];
        let ukcData = null;
        let routePolyline = null;

        // Initialize map
        function initMap() {
            map = L.map('map', {
                center: [-5.0, 106.0],
                zoom: 8,
                zoomControl: true
            });

            // Base tile layer
            L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
                attribution: '© OpenStreetMap contributors',
                maxZoom: 19
            }).addTo(map);

            // Add click handler for waypoints
            map.on('click', function(e) {
                const lat = e.latlng.lat;
                const lng = e.latlng.lng;
                addWaypoint(lat, lng);
            });

            updateInfo();
        }

        // Add waypoint
        function addWaypoint(lat, lng) {
            const draft = parseFloat(document.getElementById('shipDraft').value);
            
            // Add marker
            const marker = L.marker([lat, lng], {
                icon: L.divIcon({
                    className: 'waypoint-marker',
                    html: `<div style="background:#3498db;color:white;padding:2px 8px;border-radius:12px;font-size:10px;font-weight:bold;border:2px solid white;box-shadow:0 2px 5px rgba(0,0,0,0.3);">
                            WP${waypoints.length + 1}
                          </div>`,
                    iconSize: [40, 20],
                    iconAnchor: [20, 10]
                })
            }).addTo(map);

            waypoints.push({ lat, lng, marker });
            
            // Get depth
            fetch(`/api/depth?lat=${lat}&lng=${lng}`)
                .then(r => r.json())
                .then(data => {
                    if (data.status === 'success') {
                        const depth = data.depth;
                        // Calculate UKC
                        calculateUKC(lat, lng, depth, draft);
                    }
                });

            updateInfo();
        }

        // Calculate UKC for point
        function calculateUKC(lat, lng, depth, draft) {
            const length = parseFloat(document.getElementById('shipLength').value);
            const environment = document.getElementById('environment').value;

            fetch('/api/calculate', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    ship_name: "Navigator Vessel",
                    length: length,
                    breadth: 28.0,
                    static_draft: draft,
                    draft_trim: 0.3,
                    draft_listing: 0.2,
                    squat: 0.5,
                    wave_motion: 0.8,
                    water_depth_available: depth,
                    environment: environment
                })
            })
            .then(r => r.json())
            .then(data => {
                if (data.status === 'success') {
                    const result = data.data;
                    const ukc = result.safety_margin;
                    
                    // Color based on safety
                    const color = ukc >= 1.0 ? '#27ae60' : (ukc >= 0 ? '#f39c12' : '#e74c3c');
                    const status = ukc >= 1.0 ? 'Safe' : (ukc >= 0 ? 'Caution' : 'Unsafe');
                    
                    // Update marker with UKC info
                    const lastWaypoint = waypoints[waypoints.length - 1];
                    if (lastWaypoint) {
                        const marker = lastWaypoint.marker;
                        const popupContent = `
                            <b>Waypoint ${waypoints.length}</b><br>
                            📍 ${lat.toFixed(4)}, ${lng.toFixed(4)}<br>
                            📏 Depth: ${depth.toFixed(1)}m<br>
                            ⚓ UKC: ${ukc.toFixed(2)}m<br>
                            ✅ Status: ${status}
                        `;
                        marker.bindPopup(popupContent);
                        
                        // Update icon color
                        marker.setIcon(L.divIcon({
                            className: 'waypoint-marker',
                            html: `<div style="background:${color};color:white;padding:2px 8px;border-radius:12px;font-size:10px;font-weight:bold;border:2px solid white;box-shadow:0 2px 5px rgba(0,0,0,0.3);">
                                    WP${waypoints.length}
                                  </div>`,
                            iconSize: [40, 20],
                            iconAnchor: [20, 10]
                        }));
                    }
                    
                    updateInfo();
                }
            });
        }

        // Analyze area for UKC
        function analyzeArea() {
            const bounds = map.getBounds();
            const draft = parseFloat(document.getElementById('shipDraft').value);
            const length = parseFloat(document.getElementById('shipLength').value);
            const environment = document.getElementById('environment').value;

            document.getElementById('loading').style.display = 'block';

            fetch('/api/navigator/analyze-area', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    bounds: {
                        min_lat: bounds.getSouth(),
                        max_lat: bounds.getNorth(),
                        min_lng: bounds.getWest(),
                        max_lng: bounds.getEast()
                    },
                    ship_params: {
                        ship_name: "Navigator Vessel",
                        length: length,
                        breadth: 28.0,
                        static_draft: draft,
                        draft_trim: 0.3,
                        draft_listing: 0.2,
                        squat: 0.5,
                        wave_motion: 0.8,
                        water_depth_available: 15.0,
                        environment: environment
                    }
                })
            })
            .then(r => r.json())
            .then(data => {
                document.getElementById('loading').style.display = 'none';
                if (data.status === 'success') {
                    const analysis = data.analysis;
                    const statusText = document.getElementById('statusText');
                    statusText.textContent = `Analysis Complete: ${analysis.safety_percentage.toFixed(1)}% Safe`;
                    statusText.className = analysis.safety_percentage > 70 ? 'status-safe' : 
                                          (analysis.safety_percentage > 40 ? 'status-caution' : 'status-unsafe');
                    
                    // Refresh map to show overlays
                    refreshMap();
                    
                    alert(`UKC Analysis Complete:\n` +
                          `Safe Points: ${analysis.safe_points}\n` +
                          `Unsafe Points: ${analysis.unsafe_points}\n` +
                          `Safety: ${analysis.safety_percentage.toFixed(1)}%`);
                }
            })
            .catch(err => {
                document.getElementById('loading').style.display = 'none';
                alert('Analysis failed: ' + err.message);
            });
        }

        // Load sample route
        function loadSampleRoute() {
            const draft = parseFloat(document.getElementById('shipDraft').value);
            const length = parseFloat(document.getElementById('shipLength').value);
            const environment = document.getElementById('environment').value;

            const start = { lat: -6.125, lng: 106.655 }; // Jakarta
            const end = { lat: -7.189, lng: 112.730 }; // Surabaya
            
            const waypoints = [
                { lat: -6.5, lng: 107.0 },
                { lat: -6.8, lng: 108.0 },
                { lat: -7.0, lng: 109.0 },
                { lat: -7.1, lng: 110.0 },
                { lat: -7.15, lng: 111.0 }
            ];

            document.getElementById('loading').style.display = 'block';

            fetch('/api/navigate', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    start: { latitude: start.lat, longitude: start.lng },
                    end: { latitude: end.lat, longitude: end.lng },
                    waypoints: waypoints.map(w => ({ latitude: w.lat, longitude: w.lng })),
                    ship_params: {
                        ship_name: "Navigator Vessel",
                        length: length,
                        breadth: 28.0,
                        static_draft: draft,
                        draft_trim: 0.3,
                        draft_listing: 0.2,
                        squat: 0.5,
                        wave_motion: 0.8,
                        water_depth_available: 15.0,
                        environment: environment
                    },
                    avoid_unsafe: true,
                    max_turn_angle: 30.0
                })
            })
            .then(r => r.json())
            .then(data => {
                document.getElementById('loading').style.display = 'none';
                if (data.status === 'success') {
                    const route = data.data.route;
                    
                    // Visualize route
                    fetch('/api/navigator/visualize-route', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify(route)
                    })
                    .then(r => r.json())
                    .then(result => {
                        if (result.status === 'success') {
                            refreshMap();
                            
                            // Update info
                            const statusText = document.getElementById('statusText');
                            statusText.textContent = `Route: ${route.overall_status}`;
                            statusText.className = route.overall_status === 'SAFE' ? 'status-safe' : 'status-unsafe';
                            
                            document.getElementById('waypointCount').textContent = route.waypoints.length;
                            document.getElementById('minUKC').textContent = route.min_ukc.toFixed(2) + 'm';
                            
                            alert(`Route Created!\n` +
                                  `Distance: ${(route.total_distance / 1000).toFixed(1)} km\n` +
                                  `Waypoints: ${route.waypoints.length}\n` +
                                  `Status: ${route.overall_status}\n` +
                                  `Min UKC: ${route.min_ukc.toFixed(2)}m`);
                        }
                    });
                }
            })
            .catch(err => {
                document.getElementById('loading').style.display = 'none';
                alert('Route generation failed: ' + err.message);
            });
        }

        // Clear overlays
        function clearOverlays() {
            if (confirm('Clear all overlays from map?')) {
                fetch('/api/navigator/overlays', { method: 'DELETE' })
                    .then(r => r.json())
                    .then(data => {
                        if (data.status === 'success') {
                            refreshMap();
                            alert('Overlays cleared');
                        }
                    });
            }
        }

        // Toggle layer
        function toggleLayer(layerId) {
            fetch(`/api/layers/${layerId}/toggle`, { method: 'POST' })
                .then(r => r.json())
                .then(data => {
                    if (data.status === 'success') {
                        refreshMap();
                    }
                });
        }

        // Toggle UKC display
        function toggleUKC() {
            const show = document.getElementById('showUKC').checked;
            // Implement UKC visibility toggle
        }

        // Refresh map
        function refreshMap() {
            // Reload tile layer with timestamp to force refresh
            const tileLayer = L.tileLayer(`/tiles/{z}/{x}/{y}?t=${Date.now()}`, {
                maxZoom: 19,
                attribution: 'UKC Navigator'
            });
            
            // Remove old layers and add new
            map.eachLayer(function(layer) {
                if (layer instanceof L.TileLayer) {
                    map.removeLayer(layer);
                }
            });
            
            tileLayer.addTo(map);
            
            // Re-add waypoints
            waypoints.forEach(wp => {
                wp.marker.addTo(map);
            });
        }

        // Update info panel
        function updateInfo() {
            document.getElementById('waypointCount').textContent = waypoints.length;
        }

        // Initialize on load
        window.onload = function() {
            initMap();
            // Load sample waypoints
            setTimeout(loadSampleRoute, 1000);
        };
    </script>
</body>
</html>
            "#;
            Response::from_html(html)
        })

        // Serve simple HTML map
        .get("/", |_req, _ctx| {
            Response::redirect(Url::parse("/map").unwrap())
        })

        .run(req, env)
        .await
}

// ============================================
// DEPLOYMENT INSTRUCTIONS
// ============================================

/*
DEPLOYMENT:

1. Install Wrangler:
   npm install -g wrangler

2. Create wrangler.toml:
   [name = "ukc-navigator"]
   [main = "src/worker.rs"]
   [compatibility_date = "2024-01-01"]

3. Build:
   cargo build --target wasm32-unknown-unknown --release

4. Deploy:
   wrangler publish

5. Access:
   https://ukc-navigator.your-worker.workers.dev/map
*/

// ============================================
// ADDITIONAL REQUIREMENTS
// ============================================

// Helper function for rounding
pub trait RoundTo {
    fn round_to_2(&self) -> f64;
}

impl RoundTo for f64 {
    fn round_to_2(&self) -> f64 {
        (self * 100.0).round() / 100.0
    }
}

// Helper function for JSON responses
fn json_response<T: Serialize>(data: &T, status_code: u16) -> Result<Response> {
    let json = serde_json::to_string(data).map_err(|e| Error::from(e.to_string()))?;
    let mut response = Response::from_json(&json)?;
    response.with_status(status_code);
    response.headers_mut().set("Access-Control-Allow-Origin", "*")?;
    response.headers_mut().set("Content-Type", "application/json")?;
    Ok(response)
}

// Include other types from previous implementations
// (Coordinate, UKCInput, UKCCalculator, etc.)

// ============================================
// Cargo.toml
// ============================================

/*
[package]
name = "ukc-navigator"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
worker = { version = "0.0.21", features = ["http", "console"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4"] }
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"
