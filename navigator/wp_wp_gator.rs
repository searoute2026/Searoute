// ============================================
// UKC GIS API WITH WAYPOINT SYSTEM
// Cloudflare Workers Version with Complete Navigation
// ============================================

use worker::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

// ============================================
// WAYPOINT SYSTEM
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waypoint {
    pub id: String,
    pub name: String,
    pub coordinate: Coordinate,
    pub depth: Option<f64>,
    pub required_ukc: Option<f64>,
    pub status: WaypointStatus,
    pub properties: HashMap<String, Value>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WaypointStatus {
    Planned,
    Approved,
    Active,
    Reached,
    Skipped,
    Unsafe,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaypointWithUKC {
    pub waypoint: Waypoint,
    pub ukc_analysis: UKCResult,
    pub safety_margin: f64,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteWithWaypoints {
    pub id: String,
    pub name: String,
    pub waypoints: Vec<WaypointWithUKC>,
    pub total_distance: f64,
    pub estimated_time: f64,
    pub max_draft: f64,
    pub min_ukc: f64,
    pub unsafe_waypoints: Vec<usize>,
    pub overall_status: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaypointQuery {
    pub lat_min: Option<f64>,
    pub lat_max: Option<f64>,
    pub lng_min: Option<f64>,
    pub lng_max: Option<f64>,
    pub status: Option<WaypointStatus>,
    pub min_depth: Option<f64>,
    pub max_depth: Option<f64>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaypointBatchRequest {
    pub waypoints: Vec<CreateWaypointRequest>,
    pub ship_params: UKCInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWaypointRequest {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub depth: Option<f64>,
    pub properties: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWaypointRequest {
    pub name: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub depth: Option<f64>,
    pub status: Option<WaypointStatus>,
    pub properties: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigateRequest {
    pub start: Coordinate,
    pub end: Coordinate,
    pub ship_params: UKCInput,
    pub waypoints: Option<Vec<Coordinate>>,
    pub avoid_unsafe: Option<bool>,
    pub max_turn_angle: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationResult {
    pub route: RouteWithWaypoints,
    pub navigation_steps: Vec<NavigationStep>,
    pub total_distance: f64,
    pub estimated_time: f64,
    pub safety_assessment: NavigationSafety,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationStep {
    pub index: usize,
    pub from: Waypoint,
    pub to: Waypoint,
    pub bearing: f64,
    pub distance: f64,
    pub estimated_time: f64,
    pub instructions: String,
    pub depth_profile: Vec<DepthPoint>,
    pub min_depth: f64,
    pub max_depth: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationSafety {
    pub overall_status: String,
    pub min_ukc: f64,
    pub max_draft: f64,
    pub unsafe_segments: Vec<usize>,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthPoint {
    pub coordinate: Coordinate,
    pub depth: f64,
}

// ============================================
// WAYPOINT STORE (In-memory for demo, use Durable Objects for production)
// ============================================

pub struct WaypointStore {
    waypoints: HashMap<String, Waypoint>,
    routes: HashMap<String, RouteWithWaypoints>,
}

impl WaypointStore {
    pub fn new() -> Self {
        Self {
            waypoints: HashMap::new(),
            routes: HashMap::new(),
        }
    }

    pub fn add_waypoint(&mut self, waypoint: Waypoint) -> Result<String, String> {
        let id = waypoint.id.clone();
        if self.waypoints.contains_key(&id) {
            return Err(format!("Waypoint with id {} already exists", id));
        }
        self.waypoints.insert(id.clone(), waypoint);
        Ok(id)
    }

    pub fn get_waypoint(&self, id: &str) -> Option<&Waypoint> {
        self.waypoints.get(id)
    }

    pub fn update_waypoint(&mut self, id: &str, updates: UpdateWaypointRequest) -> Result<Waypoint, String> {
        let waypoint = self.waypoints.get_mut(id)
            .ok_or_else(|| format!("Waypoint {} not found", id))?;

        if let Some(name) = updates.name {
            waypoint.name = name;
        }
        if let Some(lat) = updates.latitude {
            waypoint.coordinate.latitude = lat;
        }
        if let Some(lng) = updates.longitude {
            waypoint.coordinate.longitude = lng;
        }
        if let Some(depth) = updates.depth {
            waypoint.depth = Some(depth);
        }
        if let Some(status) = updates.status {
            waypoint.status = status;
        }
        if let Some(properties) = updates.properties {
            waypoint.properties = properties;
        }
        waypoint.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(waypoint.clone())
    }

    pub fn delete_waypoint(&mut self, id: &str) -> Result<(), String> {
        if self.waypoints.remove(id).is_none() {
            return Err(format!("Waypoint {} not found", id));
        }
        Ok(())
    }

    pub fn list_waypoints(&self, query: &WaypointQuery) -> Vec<Waypoint> {
        let mut results: Vec<Waypoint> = self.waypoints.values().cloned().collect();

        // Apply filters
        if let Some(lat_min) = query.lat_min {
            results.retain(|w| w.coordinate.latitude >= lat_min);
        }
        if let Some(lat_max) = query.lat_max {
            results.retain(|w| w.coordinate.latitude <= lat_max);
        }
        if let Some(lng_min) = query.lng_min {
            results.retain(|w| w.coordinate.longitude >= lng_min);
        }
        if let Some(lng_max) = query.lng_max {
            results.retain(|w| w.coordinate.longitude <= lng_max);
        }
        if let Some(status) = &query.status {
            results.retain(|w| &w.status == status);
        }
        if let Some(min_depth) = query.min_depth {
            results.retain(|w| w.depth.map_or(false, |d| d >= min_depth));
        }
        if let Some(max_depth) = query.max_depth {
            results.retain(|w| w.depth.map_or(false, |d| d <= max_depth));
        }

        // Pagination
        let offset = query.offset.unwrap_or(0);
        let limit = query.limit.unwrap_or(100);
        results.into_iter().skip(offset).take(limit).collect()
    }

    pub fn find_nearby(&self, coord: &Coordinate, radius_km: f64) -> Vec<Waypoint> {
        let mut nearby = Vec::new();
        for waypoint in self.waypoints.values() {
            let distance = haversine_distance(coord, &waypoint.coordinate);
            if distance <= radius_km * 1000.0 {
                nearby.push(waypoint.clone());
            }
        }
        nearby.sort_by(|a, b| {
            let dist_a = haversine_distance(coord, &a.coordinate);
            let dist_b = haversine_distance(coord, &b.coordinate);
            dist_a.partial_cmp(&dist_b).unwrap()
        });
        nearby
    }

    pub fn save_route(&mut self, route: RouteWithWaypoints) -> Result<String, String> {
        let id = route.id.clone();
        if self.routes.contains_key(&id) {
            return Err(format!("Route with id {} already exists", id));
        }
        self.routes.insert(id.clone(), route);
        Ok(id)
    }

    pub fn get_route(&self, id: &str) -> Option<&RouteWithWaypoints> {
        self.routes.get(id)
    }

    pub fn list_routes(&self) -> Vec<RouteWithWaypoints> {
        self.routes.values().cloned().collect()
    }
}

// ============================================
// NAVIGATION ENGINE
// ============================================

pub struct NavigationEngine {
    calculator: UKCCalculator,
}

impl NavigationEngine {
    pub fn new() -> Self {
        Self {
            calculator: UKCCalculator::new(),
        }
    }

    pub fn generate_route(
        &self,
        start: &Coordinate,
        end: &Coordinate,
        waypoints: &[Coordinate],
        ship_params: &UKCInput,
        avoid_unsafe: bool,
        max_turn_angle: f64,
    ) -> NavigationResult {
        let mut all_points = Vec::new();
        all_points.push(start.clone());
        all_points.extend_from_slice(waypoints);
        all_points.push(end.clone());

        let mut route_waypoints = Vec::new();
        let mut navigation_steps = Vec::new();
        let mut total_distance = 0.0;
        let mut min_ukc = f64::INFINITY;
        let mut max_draft = ship_params.static_draft;
        let mut unsafe_segments = Vec::new();
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();

        // Analyze each segment
        for i in 0..all_points.len() - 1 {
            let from = &all_points[i];
            let to = &all_points[i + 1];
            
            let distance = haversine_distance(from, to);
            total_distance += distance;

            // Calculate bearing
            let bearing = calculate_bearing(from, to);

            // Create waypoint with UKC analysis
            let waypoint = self.create_waypoint_with_ukc(from, ship_params);
            route_waypoints.push(waypoint);

            // Generate navigation step
            let step = self.generate_navigation_step(from, to, i, bearing, distance, ship_params);
            navigation_steps.push(step.clone());

            // Update safety metrics
            if step.min_depth > 0.0 {
                let ukc = step.min_depth - ship_params.static_draft;
                if ukc < min_ukc {
                    min_ukc = ukc;
                }
            }

            if step.min_depth < ship_params.static_draft + 1.0 {
                unsafe_segments.push(i);
                warnings.push(format!(
                    "Segment {} has insufficient depth: {:.2}m (requires {:.2}m)",
                    i + 1,
                    step.min_depth,
                    ship_params.static_draft + 1.0
                ));
            }
        }

        // Add final waypoint
        let final_waypoint = self.create_waypoint_with_ukc(end, ship_params);
        route_waypoints.push(final_waypoint);

        // Determine overall status
        let overall_status = if unsafe_segments.is_empty() {
            "SAFE".to_string()
        } else if unsafe_segments.len() < navigation_steps.len() / 2 {
            "CAUTION".to_string()
        } else {
            "UNSAFE".to_string()
        };

        // Generate recommendations
        if overall_status == "CAUTION" {
            recommendations.push("Consider alternative route with deeper water".to_string());
            recommendations.push("Reduce speed to minimize squat".to_string());
        } else if overall_status == "UNSAFE" {
            recommendations.push("Route is unsafe - find alternative route".to_string());
            recommendations.push("Reduce draft or wait for higher tide".to_string());
        }

        if min_ukc < 1.0 {
            recommendations.push("UKC margin is very low - proceed with extreme caution".to_string());
        }

        // Create route
        let route = RouteWithWaypoints {
            id: Uuid::new_v4().to_string(),
            name: format!("Route from {} to {}", 
                if start.latitude == 0.0 && start.longitude == 0.0 { "Start" } else { &format!("{:.2},{:.2}", start.latitude, start.longitude) },
                if end.latitude == 0.0 && end.longitude == 0.0 { "End" } else { &format!("{:.2},{:.2}", end.latitude, end.longitude) }
            ),
            waypoints: route_waypoints,
            total_distance,
            estimated_time: total_distance / (10.0 * 1852.0), // 10 knots average
            max_draft,
            min_ukc: if min_ukc == f64::INFINITY { 0.0 } else { min_ukc },
            unsafe_waypoints: unsafe_segments,
            overall_status,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        NavigationResult {
            route,
            navigation_steps,
            total_distance,
            estimated_time: total_distance / (10.0 * 1852.0),
            safety_assessment: NavigationSafety {
                overall_status,
                min_ukc: if min_ukc == f64::INFINITY { 0.0 } else { min_ukc },
                max_draft,
                unsafe_segments,
                warnings,
                recommendations,
            },
        }
    }

    fn create_waypoint_with_ukc(&self, coord: &Coordinate, ship_params: &UKCInput) -> WaypointWithUKC {
        let input = UKCInput {
            ship_name: ship_params.ship_name.clone(),
            length: ship_params.length,
            breadth: ship_params.breadth,
            static_draft: ship_params.static_draft,
            draft_trim: ship_params.draft_trim,
            draft_listing: ship_params.draft_listing,
            squat: ship_params.squat,
            wave_motion: ship_params.wave_motion,
            water_depth_available: 20.0, // Placeholder, should come from depth data
            environment: ship_params.environment.clone(),
        };

        let ukc_result = self.calculator.calculate(&input);
        let safety_margin = ukc_result.safety_margin;
        
        let mut recommendations = Vec::new();
        if safety_margin < 1.0 {
            recommendations.push("UKC margin is low - proceed with caution".to_string());
        }
        if safety_margin < 0.0 {
            recommendations.push("UKC is negative - route is unsafe".to_string());
        }

        WaypointWithUKC {
            waypoint: Waypoint {
                id: Uuid::new_v4().to_string(),
                name: format!("WP_{:.4}_{:.4}", coord.latitude, coord.longitude),
                coordinate: coord.clone(),
                depth: Some(20.0),
                required_ukc: Some(ukc_result.ukc),
                status: if ukc_result.is_safe { WaypointStatus::Approved } else { WaypointStatus::Unsafe },
                properties: HashMap::new(),
                created_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                updated_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
            ukc_analysis: ukc_result,
            safety_margin,
            recommendations,
        }
    }

    fn generate_navigation_step(
        &self,
        from: &Coordinate,
        to: &Coordinate,
        index: usize,
        bearing: f64,
        distance: f64,
        ship_params: &UKCInput,
    ) -> NavigationStep {
        let depth_profile = vec![
            DepthPoint {
                coordinate: from.clone(),
                depth: 20.0,
            },
            DepthPoint {
                coordinate: to.clone(),
                depth: 18.0,
            },
        ];

        let min_depth = depth_profile.iter().map(|p| p.depth).fold(f64::INFINITY, f64::min);
        let max_depth = depth_profile.iter().map(|p| p.depth).fold(0.0, f64::max);

        let instructions = format!(
            "Proceed from waypoint {} to waypoint {} on bearing {:.1}° for {:.2} km",
            index + 1,
            index + 2,
            bearing,
            distance / 1000.0
        );

        NavigationStep {
            index,
            from: Waypoint {
                id: Uuid::new_v4().to_string(),
                name: format!("Step_{}_from", index),
                coordinate: from.clone(),
                depth: Some(20.0),
                required_ukc: None,
                status: WaypointStatus::Planned,
                properties: HashMap::new(),
                created_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                updated_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
            to: Waypoint {
                id: Uuid::new_v4().to_string(),
                name: format!("Step_{}_to", index),
                coordinate: to.clone(),
                depth: Some(18.0),
                required_ukc: None,
                status: WaypointStatus::Planned,
                properties: HashMap::new(),
                created_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                updated_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
            bearing,
            distance,
            estimated_time: distance / (10.0 * 1852.0),
            instructions,
            depth_profile,
            min_depth,
            max_depth,
        }
    }
}

// ============================================
// HELPER FUNCTIONS
// ============================================

fn haversine_distance(a: &Coordinate, b: &Coordinate) -> f64 {
    let lat1 = a.latitude.to_radians();
    let lon1 = a.longitude.to_radians();
    let lat2 = b.latitude.to_radians();
    let lon2 = b.longitude.to_radians();

    let dlat = lat2 - lat1;
    let dlon = lon2 - lon1;

    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    const EARTH_RADIUS: f64 = 6371000.0;
    EARTH_RADIUS * c
}

fn calculate_bearing(from: &Coordinate, to: &Coordinate) -> f64 {
    let lat1 = from.latitude.to_radians();
    let lon1 = from.longitude.to_radians();
    let lat2 = to.latitude.to_radians();
    let lon2 = to.longitude.to_radians();

    let dlon = lon2 - lon1;
    let x = dlon.sin() * lat2.cos();
    let y = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * dlon.cos();

    let bearing = y.atan2(x).to_degrees();
    (bearing + 360.0) % 360.0
}

// ============================================
// WORKER HANDLERS WITH WAYPOINT API
// ============================================

#[derive(Clone)]
pub struct AppState {
    groq_api_key: String,
    waypoint_store: Arc<Mutex<WaypointStore>>,
    navigation_engine: NavigationEngine,
}

impl AppState {
    pub fn new(env: &Env) -> Self {
        let groq_api_key = env.var("GROQ_API_KEY")
            .map(|v| v.to_string())
            .unwrap_or_else(|_| "".to_string());
        
        Self {
            groq_api_key,
            waypoint_store: Arc::new(Mutex::new(WaypointStore::new())),
            navigation_engine: NavigationEngine::new(),
        }
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

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Handle CORS preflight
    if req.method() == Method::Options {
        let mut response = Response::empty()?;
        response.headers_mut().set("Access-Control-Allow-Origin", "*")?;
        response.headers_mut().set("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")?;
        response.headers_mut().set("Access-Control-Allow-Headers", "Content-Type, Authorization")?;
        return Ok(response);
    }

    let state = AppState::new(&env);
    let router = Router::new();

    router
        // ============================================
        // WAYPOINT MANAGEMENT API
        // ============================================
        
        // Create waypoint
        .post("/api/waypoints", |mut req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            
            let body = req.text().await?;
            let request: CreateWaypointRequest = match serde_json::from_str(&body) {
                Ok(data) => data,
                Err(e) => {
                    return json_response(&json!({
                        "status": "error",
                        "message": format!("Invalid JSON: {}", e)
                    }), 400);
                }
            };

            let waypoint = Waypoint {
                id: Uuid::new_v4().to_string(),
                name: request.name,
                coordinate: Coordinate::new(request.latitude, request.longitude),
                depth: request.depth,
                required_ukc: None,
                status: WaypointStatus::Planned,
                properties: request.properties.unwrap_or_default(),
                created_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                updated_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            };

            let mut store = state.waypoint_store.lock().await;
            match store.add_waypoint(waypoint.clone()) {
                Ok(id) => {
                    json_response(&json!({
                        "status": "success",
                        "message": "Waypoint created successfully",
                        "data": waypoint
                    }), 201)
                }
                Err(e) => {
                    json_response(&json!({
                        "status": "error",
                        "message": e
                    }), 400)
                }
            }
        })

        // Get all waypoints with filters
        .get("/api/waypoints", |req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            
            let url = req.url().unwrap();
            let query: HashMap<String, String> = url.query_pairs()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            let waypoint_query = WaypointQuery {
                lat_min: query.get("lat_min").and_then(|v| v.parse().ok()),
                lat_max: query.get("lat_max").and_then(|v| v.parse().ok()),
                lng_min: query.get("lng_min").and_then(|v| v.parse().ok()),
                lng_max: query.get("lng_max").and_then(|v| v.parse().ok()),
                status: query.get("status").and_then(|v| {
                    match v.as_str() {
                        "Planned" => Some(WaypointStatus::Planned),
                        "Approved" => Some(WaypointStatus::Approved),
                        "Active" => Some(WaypointStatus::Active),
                        "Reached" => Some(WaypointStatus::Reached),
                        "Skipped" => Some(WaypointStatus::Skipped),
                        "Unsafe" => Some(WaypointStatus::Unsafe),
                        "Cancelled" => Some(WaypointStatus::Cancelled),
                        _ => None,
                    }
                }),
                min_depth: query.get("min_depth").and_then(|v| v.parse().ok()),
                max_depth: query.get("max_depth").and_then(|v| v.parse().ok()),
                limit: query.get("limit").and_then(|v| v.parse().ok()),
                offset: query.get("offset").and_then(|v| v.parse().ok()),
            };

            let store = state.waypoint_store.lock().await;
            let waypoints = store.list_waypoints(&waypoint_query);

            json_response(&json!({
                "status": "success",
                "count": waypoints.len(),
                "data": waypoints
            }), 200)
        })

        // Get waypoint by ID
        .get("/api/waypoints/:id", |req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            let id = req.param("id").unwrap();
            
            let store = state.waypoint_store.lock().await;
            match store.get_waypoint(&id) {
                Some(waypoint) => {
                    json_response(&json!({
                        "status": "success",
                        "data": waypoint
                    }), 200)
                }
                None => {
                    json_response(&json!({
                        "status": "error",
                        "message": format!("Waypoint {} not found", id)
                    }), 404)
                }
            }
        })

        // Update waypoint
        .put("/api/waypoints/:id", |mut req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            let id = req.param("id").unwrap();
            
            let body = req.text().await?;
            let updates: UpdateWaypointRequest = match serde_json::from_str(&body) {
                Ok(data) => data,
                Err(e) => {
                    return json_response(&json!({
                        "status": "error",
                        "message": format!("Invalid JSON: {}", e)
                    }), 400);
                }
            };

            let mut store = state.waypoint_store.lock().await;
            match store.update_waypoint(&id, updates) {
                Ok(waypoint) => {
                    json_response(&json!({
                        "status": "success",
                        "message": "Waypoint updated successfully",
                        "data": waypoint
                    }), 200)
                }
                Err(e) => {
                    json_response(&json!({
                        "status": "error",
                        "message": e
                    }), 404)
                }
            }
        })

        // Delete waypoint
        .delete("/api/waypoints/:id", |req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            let id = req.param("id").unwrap();
            
            let mut store = state.waypoint_store.lock().await;
            match store.delete_waypoint(&id) {
                Ok(()) => {
                    json_response(&json!({
                        "status": "success",
                        "message": format!("Waypoint {} deleted", id)
                    }), 200)
                }
                Err(e) => {
                    json_response(&json!({
                        "status": "error",
                        "message": e
                    }), 404)
                }
            }
        })

        // Batch create waypoints
        .post("/api/waypoints/batch", |mut req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            
            let body = req.text().await?;
            let request: WaypointBatchRequest = match serde_json::from_str(&body) {
                Ok(data) => data,
                Err(e) => {
                    return json_response(&json!({
                        "status": "error",
                        "message": format!("Invalid JSON: {}", e)
                    }), 400);
                }
            };

            let mut store = state.waypoint_store.lock().await;
            let mut created = Vec::new();
            let mut errors = Vec::new();

            for (i, wp_req) in request.waypoints.iter().enumerate() {
                let waypoint = Waypoint {
                    id: Uuid::new_v4().to_string(),
                    name: wp_req.name.clone(),
                    coordinate: Coordinate::new(wp_req.latitude, wp_req.longitude),
                    depth: wp_req.depth,
                    required_ukc: None,
                    status: WaypointStatus::Planned,
                    properties: wp_req.properties.clone().unwrap_or_default(),
                    created_at: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    updated_at: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };

                match store.add_waypoint(waypoint.clone()) {
                    Ok(_) => created.push(waypoint),
                    Err(e) => errors.push(format!("Waypoint {}: {}", i, e)),
                }
            }

            json_response(&json!({
                "status": if errors.is_empty() { "success" } else { "partial" },
                "created": created.len(),
                "errors": if errors.is_empty() { null } else { errors },
                "data": created
            }), if errors.is_empty() { 201 } else { 207 })
        })

        // Find nearby waypoints
        .get("/api/waypoints/nearby", |req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            
            let url = req.url().unwrap();
            let query: HashMap<String, String> = url.query_pairs()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            let lat: Option<f64> = query.get("lat").and_then(|v| v.parse().ok());
            let lng: Option<f64> = query.get("lng").and_then(|v| v.parse().ok());
            let radius: f64 = query.get("radius").and_then(|v| v.parse().ok()).unwrap_or(50.0);

            if let (Some(lat), Some(lng)) = (lat, lng) {
                let coord = Coordinate::new(lat, lng);
                let store = state.waypoint_store.lock().await;
                let nearby = store.find_nearby(&coord, radius);

                json_response(&json!({
                    "status": "success",
                    "count": nearby.len(),
                    "data": nearby,
                    "center": {
                        "latitude": lat,
                        "longitude": lng
                    },
                    "radius_km": radius
                }), 200)
            } else {
                json_response(&json!({
                    "status": "error",
                    "message": "Missing 'lat' and 'lng' parameters"
                }), 400)
            }
        })

        // ============================================
        // NAVIGATION AND ROUTING API
        // ============================================

        // Navigate route with waypoints
        .post("/api/navigate", |mut req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            
            let body = req.text().await?;
            let request: NavigateRequest = match serde_json::from_str(&body) {
                Ok(data) => data,
                Err(e) => {
                    return json_response(&json!({
                        "status": "error",
                        "message": format!("Invalid JSON: {}", e)
                    }), 400);
                }
            };

            let navigation_engine = &state.navigation_engine;
            let waypoints = request.waypoints.unwrap_or_default();
            let avoid_unsafe = request.avoid_unsafe.unwrap_or(true);
            let max_turn_angle = request.max_turn_angle.unwrap_or(30.0);

            let result = navigation_engine.generate_route(
                &request.start,
                &request.end,
                &waypoints,
                &request.ship_params,
                avoid_unsafe,
                max_turn_angle,
            );

            // Save route
            let mut store = state.waypoint_store.lock().await;
            let _ = store.save_route(result.route.clone());

            json_response(&json!({
                "status": "success",
                "data": {
                    "route": result.route,
                    "navigation_steps": result.navigation_steps,
                    "total_distance_km": result.total_distance / 1000.0,
                    "estimated_time_hours": result.estimated_time,
                    "safety_assessment": result.safety_assessment
                }
            }), 200)
        })

        // Get route by ID
        .get("/api/routes/:id", |req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            let id = req.param("id").unwrap();
            
            let store = state.waypoint_store.lock().await;
            match store.get_route(&id) {
                Some(route) => {
                    json_response(&json!({
                        "status": "success",
                        "data": route
                    }), 200)
                }
                None => {
                    json_response(&json!({
                        "status": "error",
                        "message": format!("Route {} not found", id)
                    }), 404)
                }
            }
        })

        // List all routes
        .get("/api/routes", |_req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            let store = state.waypoint_store.lock().await;
            let routes = store.list_routes();

            json_response(&json!({
                "status": "success",
                "count": routes.len(),
                "data": routes
            }), 200)
        })

        // ============================================
        // UKC CALCULATION API (with waypoint context)
        // ============================================

        // Calculate UKC for a waypoint
        .post("/api/waypoints/:id/ukc", |mut req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            let id = req.param("id").unwrap();
            
            let body = req.text().await?;
            let ship_params: UKCInput = match serde_json::from_str(&body) {
                Ok(data) => data,
                Err(e) => {
                    return json_response(&json!({
                        "status": "error",
                        "message": format!("Invalid JSON: {}", e)
                    }), 400);
                }
            };

            let mut store = state.waypoint_store.lock().await;
            let waypoint = match store.get_waypoint(&id).cloned() {
                Some(wp) => wp,
                None => {
                    return json_response(&json!({
                        "status": "error",
                        "message": format!("Waypoint {} not found", id)
                    }), 404);
                }
            };

            let depth = waypoint.depth.unwrap_or(20.0);
            let mut input = ship_params.clone();
            input.water_depth_available = depth;

            let calculator = UKCCalculator::new();
            let result = calculator.calculate(&input);

            // Update waypoint with UKC info
            let mut updates = UpdateWaypointRequest {
                name: None,
                latitude: None,
                longitude: None,
                depth: None,
                status: Some(if result.is_safe { WaypointStatus::Approved } else { WaypointStatus::Unsafe }),
                properties: None,
            };

            let mut props = waypoint.properties.clone();
            props.insert("ukc".to_string(), json!(result.ukc));
            props.insert("required_depth".to_string(), json!(result.required_depth));
            props.insert("safety_margin".to_string(), json!(result.safety_margin));
            updates.properties = Some(props);

            let updated = store.update_waypoint(&id, updates).unwrap_or(waypoint);

            json_response(&json!({
                "status": "success",
                "waypoint": updated,
                "ukc_analysis": {
                    "ukc": result.ukc,
                    "required_depth": result.required_depth,
                    "safety_margin": result.safety_margin,
                    "is_safe": result.is_safe,
                    "status": result.status,
                    "summary": result.summary
                }
            }), 200)
        })

        // ============================================
        // SAMPLE WAYPOINTS DATA
        // ============================================
        .get("/api/waypoints/sample", |_req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            let mut store = state.waypoint_store.lock().await;

            // Add sample waypoints
            let samples = vec![
                ("Jakarta", -6.125, 106.655, 15.0),
                ("Surabaya", -7.189, 112.730, 14.0),
                ("Singapore", 1.278, 103.850, 18.0),
                ("Bali", -8.340, 115.092, 20.0),
                ("Makassar", -5.148, 119.432, 16.0),
                ("Manado", 1.493, 124.841, 12.0),
                ("Pekanbaru", 0.507, 101.447, 10.0),
                ("Pontianak", -0.026, 109.342, 13.0),
                ("Ambon", -3.655, 128.190, 11.0),
                ("Jayapura", -2.533, 140.717, 9.0),
            ];

            for (name, lat, lng, depth) in samples {
                let wp = Waypoint {
                    id: Uuid::new_v4().to_string(),
                    name: name.to_string(),
                    coordinate: Coordinate::new(lat, lng),
                    depth: Some(depth),
                    required_ukc: None,
                    status: WaypointStatus::Planned,
                    properties: {
                        let mut map = HashMap::new();
                        map.insert("region".to_string(), json!("Indonesia"));
                        map
                    },
                    created_at: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    updated_at: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };
                let _ = store.add_waypoint(wp);
            }

            json_response(&json!({
                "status": "success",
                "message": "Sample waypoints added successfully",
                "count": samples.len()
            }), 200)
        })

        // ============================================
        // EXISTING API ENDPOINTS (Health, Calculate, etc.)
        // ============================================

        .get("/", |_req, _ctx| {
            Response::ok("🚢 UKC GIS API - Cloudflare Workers with Waypoint System")
        })
        .get("/api/health", |_req, _ctx| {
            let status = json!({
                "status": "healthy",
                "version": "2.0.0",
                "service": "UKC GIS API with Waypoint System",
                "platform": "Cloudflare Workers",
                "features": [
                    "UKC Calculation",
                    "AI Analysis (Groq)",
                    "Waypoint Management",
                    "Route Planning",
                    "Navigation",
                    "Batch Operations"
                ],
                "timestamp": SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            });
            json_response(&status, 200)
        })
        .post("/api/calculate", |mut req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            
            let body = req.text().await?;
            let input: UKCInput = match serde_json::from_str(&body) {
                Ok(data) => data,
                Err(e) => {
                    return json_response(&json!({
                        "status": "error",
                        "message": format!("Invalid JSON: {}", e)
                    }), 400);
                }
            };

            let calculator = UKCCalculator::new();
            let result = calculator.calculate(&input);

            if result.is_valid {
                json_response(&json!({
                    "status": "success",
                    "data": {
                        "dynamic_draft": result.dynamic_draft,
                        "ukc_requirement": result.ukc,
                        "required_depth": result.required_depth,
                        "safety_margin": result.safety_margin,
                        "is_safe": result.is_safe,
                        "status": result.status,
                        "summary": {
                            "total_draft": result.summary.total_draft,
                            "available_margin": result.summary.available_margin,
                            "percentage_margin": result.summary.percentage_margin,
                        }
                    }
                }), 200)
            } else {
                json_response(&json!({
                    "status": "error",
                    "errors": result.errors
                }), 400)
            }
        })
        .post("/api/analyze", |mut req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            
            let body = req.text().await?;
            let request: AIAnalysisRequest = match serde_json::from_str(&body) {
                Ok(data) => data,
                Err(e) => {
                    return json_response(&json!({
                        "status": "error",
                        "message": format!("Invalid JSON: {}", e)
                    }), 400);
                }
            };

            let calculator = UKCCalculator::new();
            let result = calculator.calculate(&request.ship_params);

            if !result.is_valid {
                return json_response(&json!({
                    "status": "error",
                    "errors": result.errors
                }), 400);
            }

            if state.groq_api_key.is_empty() {
                return json_response(&json!({
                    "status": "partial",
                    "message": "AI analysis unavailable: GROQ_API_KEY not set",
                    "ukc_result": {
                        "safety_margin": result.safety_margin,
                        "is_safe": result.is_safe,
                        "dynamic_draft": result.dynamic_draft,
                        "required_depth": result.required_depth,
                    }
                }), 200);
            }

            let groq_client = GroqClient::new(state.groq_api_key.clone());
            let route_info = &request.route_info;
            
            match groq_client.analyze_ukc_safety(&request.ship_params, route_info).await {
                Ok(analysis) => {
                    let recommendations: Vec<String> = analysis
                        .lines()
                        .filter(|line| line.contains("Recommend") || line.contains("Action"))
                        .map(|line| line.to_string())
                        .collect();

                    let risk_level = if result.safety_margin > 2.0 {
                        "LOW".to_string()
                    } else if result.safety_margin > 1.0 {
                        "MEDIUM".to_string()
                    } else {
                        "HIGH".to_string()
                    };

                    let response = AIAnalysisResponse {
                        analysis_id: Uuid::new_v4().to_string(),
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        analysis,
                        recommendations,
                        risk_level,
                        confidence_score: 0.85,
                    };

                    json_response(&json!({
                        "status": "success",
                        "data": response,
                        "ukc_result": {
                            "safety_margin": result.safety_margin,
                            "is_safe": result.is_safe,
                            "dynamic_draft": result.dynamic_draft,
                            "required_depth": result.required_depth,
                        }
                    }), 200)
                }
                Err(e) => {
                    json_response(&json!({
                        "status": "partial",
                        "message": format!("AI analysis failed: {}. UKC calculation available.", e),
                        "ukc_result": {
                            "safety_margin": result.safety_margin,
                            "is_safe": result.is_safe,
                            "dynamic_draft": result.dynamic_draft,
                            "required_depth": result.required_depth,
                        }
                    }), 200)
                }
            }
        })
        .post("/api/optimize-route", |mut req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            
            let body = req.text().await?;
            let request: RouteOptimizationRequest = match serde_json::from_str(&body) {
                Ok(data) => data,
                Err(e) => {
                    return json_response(&json!({
                        "status": "error",
                        "message": format!("Invalid JSON: {}", e)
                    }), 400);
                }
            };

            if state.groq_api_key.is_empty() {
                return json_response(&json!({
                    "status": "error",
                    "message": "Route optimization requires GROQ_API_KEY to be set"
                }), 503);
            }

            let groq_client = GroqClient::new(state.groq_api_key.clone());
            
            match groq_client.optimize_route_ai(
                &request.start,
                &request.end,
                request.ship_draft,
                &request.environmental_conditions,
            ).await {
                Ok(optimization) => {
                    json_response(&json!({
                        "status": "success",
                        "optimization": optimization,
                        "timestamp": SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                    }), 200)
                }
                Err(e) => {
                    json_response(&json!({
                        "status": "error",
                        "message": format!("Route optimization failed: {}", e)
                    }), 500)
                }
            }
        })
        .get("/api/ports", |_req, _ctx| async move {
            let ports = vec![
                json!({
                    "name": "Jakarta",
                    "coordinates": {"latitude": -6.125, "longitude": 106.655},
                    "max_draft": 14.0,
                    "facilities": ["Container", "Bulk"]
                }),
                json!({
                    "name": "Surabaya",
                    "coordinates": {"latitude": -7.189, "longitude": 112.730},
                    "max_draft": 13.0,
                    "facilities": ["Container", "General"]
                }),
                json!({
                    "name": "Singapore",
                    "coordinates": {"latitude": 1.278, "longitude": 103.850},
                    "max_draft": 16.0,
                    "facilities": ["Container", "Oil", "Gas"]
                }),
                json!({
                    "name": "Bali",
                    "coordinates": {"latitude": -8.340, "longitude": 115.092},
                    "max_draft": 12.0,
                    "facilities": ["Passenger", "Fishing"]
                }),
            ];

            json_response(&json!({
                "status": "success",
                "count": ports.len(),
                "data": ports
            }), 200)
        })
        .get("/api/depth", |req, _ctx| async move {
            let url = req.url().unwrap();
            let query: HashMap<String, String> = url.query_pairs()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            let lat: Option<f64> = query.get("lat").and_then(|v| v.parse().ok());
            let lng: Option<f64> = query.get("lng").and_then(|v| v.parse().ok());

            if let (Some(lat), Some(lng)) = (lat, lng) {
                let depth = 15.0 + 5.0 * (lat * 0.5).sin() + 3.0 * (lng * 0.3).cos();
                let depth = depth.max(0.0);

                json_response(&json!({
                    "status": "success",
                    "depth": depth,
                    "coordinates": {
                        "latitude": lat,
                        "longitude": lng
                    }
                }), 200)
            } else {
                json_response(&json!({
                    "status": "error",
                    "message": "Missing 'lat' or 'lng' parameters"
                }), 400)
            }
        })
        .get("/api/system", |_req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            let store = state.waypoint_store.lock().await;
            let routes = store.list_routes();

            json_response(&json!({
                "status": "success",
                "system": {
                    "platform": "Cloudflare Workers",
                    "runtime": "WebAssembly",
                    "groq_available": state.groq_api_key.is_empty() == false,
                    "waypoints_count": store.waypoints.len(),
                    "routes_count": routes.len(),
                    "features": [
                        "UKC Calculation",
                        "AI Analysis",
                        "Route Optimization",
                        "Waypoint Management",
                        "Navigation",
                        "Batch Operations",
                        "Port Data",
                        "Depth Data"
                    ]
                }
            }), 200)
        })

        .run(req, env)
        .await
}

// ============================================
// DEPLOYMENT INSTRUCTIONS
// ============================================

/*
DEPLOYMENT TO CLOUDFLARE WORKERS:

1. Install Wrangler CLI:
   npm install -g wrangler

2. Create wrangler.toml:
   [name = "ukc-gis-api"]
   [main = "src/worker.rs"]
   [compatibility_date = "2024-01-01"]

3. Add environment variables:
   [vars]
   GROQ_API_KEY = "your_groq_api_key_here"

4. Build and deploy:
   cargo build --target wasm32-unknown-unknown --release
   wrangler publish

5. Set secrets:
   wrangler secret put GROQ_API_KEY

6. Test your API:
   curl https://ukc-gis-api.your-worker.workers.dev/api/health
   curl https://ukc-gis-api.your-worker.workers.dev/api/waypoints/sample
*/

// ============================================
// FILE STRUCTURE
// ============================================

/*
src/
├── worker.rs          (main file with all code)
├── lib.rs             (for module exports)
└── ukc_types.rs       (optional: separate module)

Cargo.toml:
[package]
name = "ukc-gis-worker"
version = "2.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
worker = { version = "0.0.21", features = ["http", "console"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4"] }
async-trait = "0.1"
