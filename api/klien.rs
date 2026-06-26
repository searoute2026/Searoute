// ============================================
// UKC GIS API - CLOUDFLARE WORKERS VERSION
// Deployable on Cloudflare Workers with Rust WASM
// ============================================

use worker::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;
use reqwest::Client;
use uuid::Uuid;

// ============================================
// CORE GEOSPATIAL TYPES (Simplified for Workers)
// ============================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lng: f64,
    pub max_lng: f64,
}

// ============================================
// UKC TYPES
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Environment {
    #[serde(rename = "Port Approach")]
    PortApproach,
    #[serde(rename = "Coastal Water")]
    CoastalWater,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::PortApproach => "Port Approach",
            Environment::CoastalWater => "Coastal Water",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UKCInput {
    pub ship_name: String,
    pub length: f64,
    pub breadth: f64,
    pub static_draft: f64,
    pub draft_trim: f64,
    pub draft_listing: f64,
    pub squat: f64,
    pub wave_motion: f64,
    pub water_depth_available: f64,
    pub environment: Environment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UKCResult {
    pub is_valid: bool,
    pub ship_name: String,
    pub dynamic_draft: f64,
    pub ukc: f64,
    pub required_depth: f64,
    pub status: String,
    pub safety_margin: f64,
    pub is_safe: bool,
    pub summary: UKCSummary,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UKCSummary {
    pub total_draft: f64,
    pub available_margin: f64,
    pub percentage_margin: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteInfo {
    pub total_distance: f64,
    pub waypoint_count: usize,
    pub min_depth: f64,
    pub max_depth: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalConditions {
    pub wind_speed: f64,
    pub wave_height: f64,
    pub current_speed: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub visibility: f64,
    pub wind_speed: f64,
    pub wind_direction: String,
    pub current_speed: f64,
    pub current_direction: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAnalysisRequest {
    pub ship_params: UKCInput,
    pub route_info: RouteInfo,
    pub environmental_conditions: Option<EnvironmentalConditions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAnalysisResponse {
    pub analysis_id: String,
    pub timestamp: u64,
    pub analysis: String,
    pub recommendations: Vec<String>,
    pub risk_level: String,
    pub confidence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteOptimizationRequest {
    pub start: Coordinate,
    pub end: Coordinate,
    pub ship_draft: f64,
    pub environmental_conditions: EnvironmentalConditions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortApproachRequest {
    pub port_name: String,
    pub ship_params: UKCInput,
    pub weather_data: WeatherData,
}

// ============================================
// GROQ CLIENT FOR WORKERS
// ============================================

#[derive(Debug, Clone)]
pub struct GroqMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GroqRequest {
    pub model: String,
    pub messages: Vec<GroqMessage>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f64>,
    pub stream: Option<bool>,
    pub stop: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GroqResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<GroqChoice>,
    pub usage: GroqUsage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GroqChoice {
    pub index: u32,
    pub message: GroqMessageResponse,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GroqMessageResponse {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GroqUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

pub struct GroqClient {
    api_key: String,
    base_url: String,
}

impl GroqClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.groq.com/openai/v1".to_string(),
        }
    }

    pub async fn chat_completion(&self, request: &GroqRequest) -> Result<GroqResponse, String> {
        let url = format!("{}/chat/completions", self.base_url);
        
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(format!("API error: {}", text));
        }

        let groq_response: GroqResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(groq_response)
    }

    pub async fn analyze_ukc_safety(
        &self,
        ship_params: &UKCInput,
        route_info: &RouteInfo,
    ) -> Result<String, String> {
        let prompt = format!(
            r#"Perform a comprehensive UKC (Under Keel Clearance) safety analysis:
            
            Vessel Information:
            - Name: {}
            - Length: {:.1}m
            - Breadth: {:.1}m
            - Static Draft: {:.1}m
            - Draft due to Trim: {:.1}m
            - Draft due to Listing: {:.1}m
            - Squat: {:.1}m
            - Wave Motion: {:.1}m
            
            Route Information:
            - Total Distance: {:.1} km
            - Number of Waypoints: {}
            - Environment: {}
            - Minimum Depth: {:.1}m
            - Maximum Depth: {:.1}m
            
            Provide:
            1. UKC calculation results
            2. Safety margin assessment
            3. Risk factors
            4. Recommended actions
            5. Safe speed recommendations"#,
            ship_params.ship_name,
            ship_params.length,
            ship_params.breadth,
            ship_params.static_draft,
            ship_params.draft_trim,
            ship_params.draft_listing,
            ship_params.squat,
            ship_params.wave_motion,
            route_info.total_distance / 1000.0,
            route_info.waypoint_count,
            ship_params.environment.as_str(),
            route_info.min_depth,
            route_info.max_depth
        );

        let request = GroqRequest {
            model: "llama-3.1-70b-versatile".to_string(),
            messages: vec![
                GroqMessage {
                    role: "system".to_string(),
                    content: "You are a maritime safety expert specializing in Under Keel Clearance (UKC) analysis. Provide detailed, actionable safety recommendations.".to_string(),
                },
                GroqMessage {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            temperature: Some(0.7),
            max_tokens: Some(1000),
            top_p: Some(0.9),
            stream: Some(false),
            stop: None,
        };

        let response = self.chat_completion(&request).await?;
        
        if let Some(choice) = response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err("No response from Groq".to_string())
        }
    }

    pub async fn optimize_route_ai(
        &self,
        start: &Coordinate,
        end: &Coordinate,
        ship_draft: f64,
        environmental_conditions: &EnvironmentalConditions,
    ) -> Result<String, String> {
        let prompt = format!(
            r#"As a maritime route optimization expert, provide the optimal route between:
            Start: ({:.4}, {:.4})
            End: ({:.4}, {:.4})
            Ship Draft: {:.1}m
            Environmental Conditions:
            - Wind Speed: {:.1} knots
            - Wave Height: {:.1}m
            - Current Speed: {:.1} knots
            
            Provide:
            1. Recommended route waypoints (latitude, longitude)
            2. Estimated time and distance
            3. Safety considerations
            4. Alternative routes if needed
            
            Format as structured data."#,
            start.latitude, start.longitude,
            end.latitude, end.longitude,
            ship_draft,
            environmental_conditions.wind_speed,
            environmental_conditions.wave_height,
            environmental_conditions.current_speed
        );

        let request = GroqRequest {
            model: "llama-3.1-70b-versatile".to_string(),
            messages: vec![
                GroqMessage {
                    role: "system".to_string(),
                    content: "You are an expert maritime navigator with 20 years of experience.".to_string(),
                },
                GroqMessage {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            temperature: Some(0.5),
            max_tokens: Some(800),
            top_p: Some(0.9),
            stream: Some(false),
            stop: None,
        };

        let response = self.chat_completion(&request).await?;
        
        if let Some(choice) = response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err("No response from Groq".to_string())
        }
    }
}

// ============================================
// UKC CALCULATOR
// ============================================

#[derive(Debug, Clone)]
pub struct UKCCalculator;

impl UKCCalculator {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate_dynamic_draft(
        &self,
        draft_trim: f64,
        draft_listing: f64,
        squat: f64,
        wave_motion: f64,
    ) -> f64 {
        let max_draft = draft_trim.max(draft_listing);
        let dynamic_draft = max_draft + squat + wave_motion;
        (dynamic_draft * 100.0).round() / 100.0
    }

    pub fn calculate_ukc_requirement(
        &self,
        environment: &Environment,
        static_draft: f64,
        dynamic_draft: f64,
    ) -> f64 {
        match environment {
            Environment::PortApproach => {
                (1.0).max(0.10 * static_draft)
            }
            Environment::CoastalWater => {
                0.20 * dynamic_draft
            }
        }
    }

    pub fn calculate_required_depth(&self, dynamic_draft: f64, ukc: f64) -> f64 {
        (dynamic_draft + ukc)
    }

    pub fn determine_status(&self, available_depth: f64, required_depth: f64) -> (String, f64, bool) {
        let safety_margin = (available_depth - required_depth);
        let is_safe = safety_margin >= 0.0;
        let status = if is_safe {
            "ACCEPTABLE".to_string()
        } else {
            "NOT ACCEPTABLE".to_string()
        };
        (status, safety_margin, is_safe)
    }

    pub fn calculate(&self, input: &UKCInput) -> UKCResult {
        let mut errors = Vec::new();

        // Validate inputs
        if input.ship_name.trim().is_empty() {
            errors.push("Ship Name is required".to_string());
        }

        let numeric_fields = vec![
            ("Length", input.length),
            ("Breadth", input.breadth),
            ("Static Draft", input.static_draft),
            ("Draft due to Trim", input.draft_trim),
            ("Draft due to Listing", input.draft_listing),
            ("Squat", input.squat),
            ("Wave Motion", input.wave_motion),
            ("Water Depth Available", input.water_depth_available),
        ];

        for (name, value) in numeric_fields {
            if value < 0.0 {
                errors.push(format!("{} cannot be negative", name));
            }
        }

        // Validate draft doesn't exceed depth
        if input.static_draft > 0.0 && input.water_depth_available > 0.0 {
            let max_draft = input.draft_trim.max(input.draft_listing);
            let total_draft = max_draft + input.squat + input.wave_motion;
            if total_draft > input.water_depth_available {
                errors.push(format!(
                    "Total draft ({:.2}m) exceeds available water depth ({:.2}m)",
                    total_draft, input.water_depth_available
                ));
            }
        }

        if !errors.is_empty() {
            return UKCResult {
                is_valid: false,
                ship_name: input.ship_name.clone(),
                dynamic_draft: 0.0,
                ukc: 0.0,
                required_depth: 0.0,
                status: "INVALID".to_string(),
                safety_margin: 0.0,
                is_safe: false,
                summary: UKCSummary {
                    total_draft: 0.0,
                    available_margin: 0.0,
                    percentage_margin: 0.0,
                },
                errors,
            };
        }

        // Calculate
        let dynamic_draft = self.calculate_dynamic_draft(
            input.draft_trim,
            input.draft_listing,
            input.squat,
            input.wave_motion,
        );

        let ukc = self.calculate_ukc_requirement(
            &input.environment,
            input.static_draft,
            dynamic_draft,
        );

        let required_depth = self.calculate_required_depth(dynamic_draft, ukc);
        let (status, safety_margin, is_safe) = self.determine_status(
            input.water_depth_available,
            required_depth,
        );

        let total_draft = (dynamic_draft + input.static_draft);
        let available_margin = (input.water_depth_available - required_depth);
        let percentage_margin = if input.water_depth_available > 0.0 {
            ((input.water_depth_available - required_depth) / input.water_depth_available * 100.0)
        } else {
            0.0
        };

        UKCResult {
            is_valid: true,
            ship_name: input.ship_name.clone(),
            dynamic_draft,
            ukc: (ukc * 100.0).round() / 100.0,
            required_depth: (required_depth * 100.0).round() / 100.0,
            status,
            safety_margin: (safety_margin * 100.0).round() / 100.0,
            is_safe,
            summary: UKCSummary {
                total_draft: (total_draft * 100.0).round() / 100.0,
                available_margin: (available_margin * 100.0).round() / 100.0,
                percentage_margin: (percentage_margin * 100.0).round() / 100.0,
            },
            errors: Vec::new(),
        }
    }
}

// ============================================
// WORKER HANDLERS
// ============================================

#[derive(Clone)]
pub struct AppState {
    groq_api_key: String,
}

impl AppState {
    pub fn new(env: &Env) -> Self {
        let groq_api_key = env.var("GROQ_API_KEY")
            .map(|v| v.to_string())
            .unwrap_or_else(|_| "".to_string());
        Self { groq_api_key }
    }
}

// Helper function to create JSON response
fn json_response<T: Serialize>(data: &T, status_code: u16) -> Result<Response> {
    let json = serde_json::to_string(data).map_err(|e| Error::from(e.to_string()))?;
    let response = Response::from_json(&json)?;
    Ok(response.with_status(status_code))
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
        .get("/", |_req, _ctx| {
            Response::ok("🚢 UKC GIS API - Cloudflare Workers")
        })
        .get("/api/health", |_req, _ctx| {
            let status = json!({
                "status": "healthy",
                "version": "1.0.0",
                "service": "UKC GIS API with Groq AI",
                "platform": "Cloudflare Workers",
                "timestamp": SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            });
            json_response(&status, 200)
        })
        .post("/api/calculate", |mut req, ctx| async move {
            let state = ctx.data::<AppState>().unwrap();
            
            // Parse request body
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

            // Calculate UKC
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
            
            // Parse request
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

            // First perform standard calculation
            let calculator = UKCCalculator::new();
            let result = calculator.calculate(&request.ship_params);

            if !result.is_valid {
                return json_response(&json!({
                    "status": "error",
                    "errors": result.errors
                }), 400);
            }

            // Check if Groq API key is available
            if state.groq_api_key.is_empty() {
                // Fallback: return UKC calculation without AI analysis
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

            // Perform AI analysis
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
                    // Return UKC calculation even if AI fails
                    json_response(&json!({
                        "status": "partial",
                        "message": format!("AI analysis failed: {}. UKC calculation is still available.", e),
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
            
            // Parse request
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

            // Check if Groq API key is available
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
        .get("/api/ports", |_req, ctx| async move {
            // Sample ports data
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
            ];

            json_response(&json!({
                "status": "success",
                "count": ports.len(),
                "ports": ports
            }), 200)
        })
        .get("/api/depth", |req, ctx| async move {
            let url = req.url().unwrap();
            let query: HashMap<String, String> = url.query_pairs()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            let lat: Option<f64> = query.get("lat").and_then(|v| v.parse().ok());
            let lng: Option<f64> = query.get("lng").and_then(|v| v.parse().ok());

            if let (Some(lat), Some(lng)) = (lat, lng) {
                // Simulate depth data (in production, fetch from real database)
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
            json_response(&json!({
                "status": "success",
                "system": {
                    "platform": "Cloudflare Workers",
                    "runtime": "WebAssembly",
                    "groq_available": ctx.data::<AppState>().unwrap().groq_api_key.is_empty() == false,
                    "features": ["UKC Calculation", "AI Analysis", "Route Optimization", "Port Data"]
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
*/

// ============================================
// Cargo.toml
// ============================================

/*
[package]
name = "ukc-gis-worker"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
worker = { version = "0.0.21", features = ["http", "console"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.11", features = ["json"] }
uuid = { version = "1.0", features = ["v4"] }
