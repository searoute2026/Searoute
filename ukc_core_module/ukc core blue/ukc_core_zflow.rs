// UKC Calculation Module - Under Keel Clearance
// Reusable module for maritime safety analysis

use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    PortApproach,
    CoastalWater,
}

impl Environment {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "Port Approach" => Ok(Environment::PortApproach),
            "Coastal Water" => Ok(Environment::CoastalWater),
            _ => Err(format!(
                "Invalid environment type. Use 'Port Approach' or 'Coastal Water'"
            )),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::PortApproach => "Port Approach",
            Environment::CoastalWater => "Coastal Water",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub all_inputs_valid: bool,
    pub errors: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            all_inputs_valid: false,
            errors: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
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

impl Default for UKCInput {
    fn default() -> Self {
        Self {
            ship_name: "Unknown".to_string(),
            length: 0.0,
            breadth: 0.0,
            static_draft: 0.0,
            draft_trim: 0.0,
            draft_listing: 0.0,
            squat: 0.0,
            wave_motion: 0.0,
            water_depth_available: 0.0,
            environment: Environment::CoastalWater,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DynamicDraftParams {
    pub static_draft: f64,
    pub draft_trim: f64,
    pub draft_listing: f64,
    pub squat: f64,
    pub wave_motion: f64,
}

#[derive(Debug, Clone)]
pub struct UKCResult {
    pub is_valid: bool,
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
    pub dynamic_draft: f64,
    pub ukc: f64,
    pub required_depth: f64,
    pub status: String,
    pub safety_margin: f64,
    pub is_safe: bool,
    pub summary: UKCSummary,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UKCSummary {
    pub total_draft: f64,
    pub available_margin: f64,
    pub percentage_margin: f64,
}

impl UKCSummary {
    fn new(static_draft: f64, dynamic_draft: f64, water_depth_available: f64, required_depth: f64) -> Self {
        let total_draft = (dynamic_draft + static_draft).round_to_2();
        let available_margin = (water_depth_available - required_depth).round_to_2();
        let percentage_margin = if water_depth_available > 0.0 {
            ((water_depth_available - required_depth) / water_depth_available * 100.0).round_to_2()
        } else {
            0.0
        };
        Self {
            total_draft,
            available_margin,
            percentage_margin,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Waypoint {
    pub latitude: f64,
    pub longitude: f64,
    pub depth: f64,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WaypointSafety {
    pub waypoint: Waypoint,
    pub depth: f64,
    pub is_safe: bool,
    pub status: String,
    pub safety_margin: f64,
    pub required_depth: f64,
    pub dynamic_draft: f64,
    pub ukc: f64,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RouteSafety {
    pub total_waypoints: usize,
    pub unsafe_waypoints: usize,
    pub min_safety_margin: f64,
    pub max_required_depth: f64,
    pub results: Vec<RouteWaypointResult>,
    pub overall_status: String,
    pub safe_percentage: f64,
}

#[derive(Debug, Clone)]
pub struct RouteWaypointResult {
    pub index: usize,
    pub lat: f64,
    pub lng: f64,
    pub depth: f64,
    pub is_safe: bool,
    pub result: UKCResult,
}

#[derive(Debug, Clone)]
pub struct Route {
    pub coordinates: Vec<(f64, f64)>,
    pub distance: Option<f64>,
    pub safety_analysis: Option<RouteSafety>,
    pub score: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct BestRouteResult {
    pub best_route: Option<Route>,
    pub all_routes: Vec<Route>,
    pub best_score: f64,
}

#[derive(Debug, Clone)]
pub struct VoyageData {
    pub route_analysis: RouteSafety,
    pub ship_params: UKCInput,
    pub departure: String,
    pub destination: String,
}

// Helper trait for rounding
pub trait RoundTo {
    fn round_to_2(&self) -> f64;
}

impl RoundTo for f64 {
    fn round_to_2(&self) -> f64 {
        (self * 100.0).round() / 100.0
    }
}

pub struct UKCCalculator {
    validations: ValidationResult,
}

impl UKCCalculator {
    pub fn new() -> Self {
        Self {
            validations: ValidationResult::new(),
        }
    }

    /// Calculate Dynamic Draft
    pub fn calculate_dynamic_draft(&self, params: &DynamicDraftParams) -> f64 {
        let max_draft = params.draft_trim.max(params.draft_listing);
        let dynamic_draft = max_draft + params.squat + params.wave_motion;
        dynamic_draft.round_to_2()
    }

    /// Calculate UKC Requirement based on environment
    pub fn calculate_ukc_requirement(
        &self,
        environment: &Environment,
        static_draft: f64,
        dynamic_draft: f64,
    ) -> f64 {
        match environment {
            Environment::PortApproach => {
                // UKC = max(1.0 meter, 10% × Static Draft)
                (1.0).max(0.10 * static_draft).round_to_2()
            }
            Environment::CoastalWater => {
                // UKC = 20% × Dynamic Draft
                (0.20 * dynamic_draft).round_to_2()
            }
        }
    }

    /// Calculate Required Water Depth
    pub fn calculate_required_depth(&self, dynamic_draft: f64, ukc: f64) -> f64 {
        (dynamic_draft + ukc).round_to_2()
    }

    /// Determine status based on available depth
    pub fn determine_status(&self, available_depth: f64, required_depth: f64) -> (String, f64, bool) {
        let safety_margin = (available_depth - required_depth).round_to_2();
        let is_safe = safety_margin >= 0.0;
        let status = if is_safe {
            "ACCEPTABLE".to_string()
        } else {
            "NOT ACCEPTABLE".to_string()
        };
        (status, safety_margin, is_safe)
    }

    /// Validate all input parameters
    pub fn validate_inputs(&mut self, input: &UKCInput) -> ValidationResult {
        let mut errors = Vec::new();

        // Check required fields
        if input.ship_name.trim().is_empty() {
            errors.push("Ship Name is required".to_string());
        }

        // Check numeric fields
        let numeric_fields = vec![
            ("Length", input.length),
            ("Breadth", input.breadth),
            ("Static Draft", input.static_draft),
            ("Draft due to Trim", input.draft_trim),
            ("Draft due to Listing", input.draft_listing),
            ("Squat due to Ship Speed", input.squat),
            ("Wave-Induced Motion", input.wave_motion),
            ("Water Depth Available", input.water_depth_available),
        ];

        for (name, value) in numeric_fields {
            if value < 0.0 {
                errors.push(format!("{} cannot be negative", name));
            }
        }

        // Validate environment is already validated by the enum

        // Validate that draft doesn't exceed water depth
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

        self.validations.errors = errors.clone();
        self.validations.all_inputs_valid = errors.is_empty();

        self.validations.clone()
    }

    /// Main calculation function - performs all UKC calculations
    pub fn calculate(&mut self, input: &UKCInput) -> UKCResult {
        // Validate inputs
        let validation = self.validate_inputs(input);
        if !validation.all_inputs_valid {
            return UKCResult {
                is_valid: false,
                ship_name: input.ship_name.clone(),
                length: input.length,
                breadth: input.breadth,
                static_draft: input.static_draft,
                draft_trim: input.draft_trim,
                draft_listing: input.draft_listing,
                squat: input.squat,
                wave_motion: input.wave_motion,
                water_depth_available: input.water_depth_available,
                environment: input.environment.clone(),
                dynamic_draft: 0.0,
                ukc: 0.0,
                required_depth: 0.0,
                status: "INVALID".to_string(),
                safety_margin: 0.0,
                is_safe: false,
                summary: UKCSummary::new(0.0, 0.0, 0.0, 0.0),
                errors: validation.errors,
            };
        }

        // Step 1: Calculate Dynamic Draft
        let dynamic_draft_params = DynamicDraftParams {
            static_draft: input.static_draft,
            draft_trim: input.draft_trim,
            draft_listing: input.draft_listing,
            squat: input.squat,
            wave_motion: input.wave_motion,
        };
        let dynamic_draft = self.calculate_dynamic_draft(&dynamic_draft_params);

        // Step 2: Calculate UKC Requirement
        let ukc = self.calculate_ukc_requirement(
            &input.environment,
            input.static_draft,
            dynamic_draft,
        );

        // Step 3: Calculate Required Depth
        let required_depth = self.calculate_required_depth(dynamic_draft, ukc);

        // Step 4: Determine Status
        let (status, safety_margin, is_safe) = self.determine_status(
            input.water_depth_available,
            required_depth,
        );

        // Create summary
        let summary = UKCSummary::new(
            input.static_draft,
            dynamic_draft,
            input.water_depth_available,
            required_depth,
        );

        UKCResult {
            is_valid: true,
            ship_name: input.ship_name.clone(),
            length: input.length,
            breadth: input.breadth,
            static_draft: input.static_draft,
            draft_trim: input.draft_trim,
            draft_listing: input.draft_listing,
            squat: input.squat,
            wave_motion: input.wave_motion,
            water_depth_available: input.water_depth_available,
            environment: input.environment.clone(),
            dynamic_draft,
            ukc,
            required_depth,
            status,
            safety_margin,
            is_safe,
            summary,
            errors: Vec::new(),
        }
    }

    /// Generate a human-readable report
    pub fn generate_report(&self, result: &UKCResult) -> String {
        if !result.is_valid {
            return format!("❌ ERROR: {}", result.errors.join(", "));
        }

        let environment_str = result.environment.as_str();
        let status_display = &result.status;

        format!(
            r#"
╔══════════════════════════════════════════════════════════════╗
║           UNDER KEEL CLEARANCE (UKC) ANALYSIS               ║
╠══════════════════════════════════════════════════════════════╣
║ SHIP: {:<40}║
║ LENGTH: {:>6} m  BREADTH: {:>6} m ║
╠══════════════════════════════════════════════════════════════╣
║ INPUT PARAMETERS:                                           ║
║  Static Draft:           {:>8} m ║
║  Draft due to Trim:      {:>8} m ║
║  Draft due to Listing:   {:>8} m ║
║  Squat:                  {:>8} m ║
║  Wave-Induced Motion:    {:>8} m ║
╠══════════════════════════════════════════════════════════════╣
║ RESULTS:                                                    ║
║  Dynamic Draft:           {:>8} m ║
║  UKC Requirement:         {:>8} m ║
║  Required Depth:          {:>8} m ║
║  Available Depth:         {:>8} m ║
╠══════════════════════════════════════════════════════════════╣
║  STATUS: {:<40}║
║  Safety Margin:           {:>8} m ║
║  Environment: {:<46}║
╚══════════════════════════════════════════════════════════════╝
"#,
            result.ship_name,
            result.length,
            result.breadth,
            result.static_draft,
            result.draft_trim,
            result.draft_listing,
            result.squat,
            result.wave_motion,
            result.dynamic_draft,
            result.ukc,
            result.required_depth,
            result.water_depth_available,
            status_display,
            result.safety_margin,
            environment_str
        )
    }

    /// Check if a route waypoint is safe based on UKC requirements
    pub fn check_waypoint_safety(
        &mut self,
        waypoint: &Waypoint,
        ship_params: &UKCInput,
        environment: Environment,
    ) -> WaypointSafety {
        if waypoint.depth <= 0.0 {
            return WaypointSafety {
                waypoint: waypoint.clone(),
                depth: waypoint.depth,
                is_safe: false,
                status: "ERROR".to_string(),
                safety_margin: 0.0,
                required_depth: 0.0,
                dynamic_draft: 0.0,
                ukc: 0.0,
                error: Some("No depth data available for this location".to_string()),
            };
        }

        let mut input = ship_params.clone();
        input.water_depth_available = waypoint.depth;
        input.environment = environment;

        let result = self.calculate(&input);

        WaypointSafety {
            waypoint: waypoint.clone(),
            depth: waypoint.depth,
            is_safe: result.is_safe,
            status: result.status,
            safety_margin: result.safety_margin,
            required_depth: result.required_depth,
            dynamic_draft: result.dynamic_draft,
            ukc: result.ukc,
            error: None,
        }
    }

    /// Check multiple waypoints for UKC safety
    pub fn check_route_safety(
        &mut self,
        waypoints: &[Waypoint],
        ship_params: &UKCInput,
        environment: Environment,
    ) -> RouteSafety {
        let mut results = Vec::new();
        let mut unsafe_count = 0;
        let mut min_safety_margin = f64::INFINITY;
        let mut max_required_depth = 0.0;

        for (index, wp) in waypoints.iter().enumerate() {
            let safety = self.check_waypoint_safety(wp, ship_params, environment.clone());
            if !safety.is_safe {
                unsafe_count += 1;
            }
            if safety.safety_margin < min_safety_margin {
                min_safety_margin = safety.safety_margin;
            }
            if safety.required_depth > max_required_depth {
                max_required_depth = safety.required_depth;
            }

            results.push(RouteWaypointResult {
                index,
                lat: wp.latitude,
                lng: wp.longitude,
                depth: wp.depth,
                is_safe: safety.is_safe,
                result: UKCResult {
                    is_valid: true,
                    ship_name: ship_params.ship_name.clone(),
                    length: ship_params.length,
                    breadth: ship_params.breadth,
                    static_draft: ship_params.static_draft,
                    draft_trim: ship_params.draft_trim,
                    draft_listing: ship_params.draft_listing,
                    squat: ship_params.squat,
                    wave_motion: ship_params.wave_motion,
                    water_depth_available: wp.depth,
                    environment: environment.clone(),
                    dynamic_draft: safety.dynamic_draft,
                    ukc: safety.ukc,
                    required_depth: safety.required_depth,
                    status: safety.status,
                    safety_margin: safety.safety_margin,
                    is_safe: safety.is_safe,
                    summary: UKCSummary::new(0.0, 0.0, 0.0, 0.0),
                    errors: Vec::new(),
                },
            });
        }

        let overall_status = if unsafe_count == 0 {
            "SAFE".to_string()
        } else {
            "UNSAFE".to_string()
        };

        let safe_percentage = if waypoints.len() > 0 {
            ((waypoints.len() - unsafe_count) as f64 / waypoints.len() as f64 * 100.0).round_to_2()
        } else {
            0.0
        };

        RouteSafety {
            total_waypoints: waypoints.len(),
            unsafe_waypoints: unsafe_count,
            min_safety_margin: if min_safety_margin == f64::INFINITY {
                0.0
            } else {
                min_safety_margin
            },
            max_required_depth,
            results,
            overall_status,
            safe_percentage,
        }
    }
}

pub struct UKCIntegration {
    calculator: UKCCalculator,
    ship_params: UKCInput,
    environment_types: Vec<String>,
    calculation_history: Vec<UKCResult>,
}

impl UKCIntegration {
    pub fn new() -> Self {
        Self {
            calculator: UKCCalculator::new(),
            ship_params: UKCInput::default(),
            environment_types: vec![
                "Port Approach".to_string(),
                "Coastal Water".to_string(),
            ],
            calculation_history: Vec::new(),
        }
    }

    pub fn with_calculator(calculator: UKCCalculator) -> Self {
        Self {
            calculator,
            ship_params: UKCInput::default(),
            environment_types: vec![
                "Port Approach".to_string(),
                "Coastal Water".to_string(),
            ],
            calculation_history: Vec::new(),
        }
    }

    /// Update ship parameters for UKC calculations
    pub fn update_ship_params(&mut self, params: UKCInput) {
        self.ship_params = params;
    }

    /// Get depth information for a route and check UKC safety
    pub fn analyze_route_safety(
        &mut self,
        route_coords: &[(f64, f64)],
        depth_provider: Option<fn(f64, f64) -> f64>,
    ) -> RouteSafety {
        let mut results = Vec::new();
        let mut unsafe_count = 0;
        let mut min_safety_margin = f64::INFINITY;
        let mut max_required_depth = 0.0;

        for (i, &(lat, lng)) in route_coords.iter().enumerate() {
            let depth = if let Some(provider) = depth_provider {
                provider(lat, lng)
            } else {
                999.0
            };

            let mut input = self.ship_params.clone();
            input.water_depth_available = depth;
            input.environment = self.ship_params.environment.clone();

            let result = self.calculator.calculate(&input);

            results.push(RouteWaypointResult {
                index: i,
                lat,
                lng,
                depth,
                is_safe: result.is_safe,
                result: result.clone(),
            });

            if !result.is_safe {
                unsafe_count += 1;
            }
            if result.safety_margin < min_safety_margin {
                min_safety_margin = result.safety_margin;
            }
            if result.required_depth > max_required_depth {
                max_required_depth = result.required_depth;
            }
        }

        let overall_status = if unsafe_count == 0 {
            "SAFE".to_string()
        } else {
            "UNSAFE".to_string()
        };

        let safe_percentage = if route_coords.len() > 0 {
            ((route_coords.len() - unsafe_count) as f64 / route_coords.len() as f64 * 100.0).round_to_2()
        } else {
            0.0
        };

        RouteSafety {
            total_waypoints: route_coords.len(),
            unsafe_waypoints: unsafe_count,
            min_safety_margin: if min_safety_margin == f64::INFINITY {
                0.0
            } else {
                min_safety_margin
            },
            max_required_depth,
            results,
            overall_status,
            safe_percentage,
        }
    }

    /// Find the safest route among alternatives based on UKC
    pub fn find_safest_route(
        &mut self,
        routes: &mut [Route],
        depth_provider: Option<fn(f64, f64) -> f64>,
    ) -> BestRouteResult {
        let mut best_route: Option<Route> = None;
        let mut best_score = f64::NEG_INFINITY;

        for route in routes {
            let analysis = self.analyze_route_safety(&route.coordinates, depth_provider);
            // Score: prioritize safety, then safety margin, then distance
            let safety_score = analysis.safe_percentage / 100.0 * 100.0;
            let margin_score = analysis.min_safety_margin.min(20.0) * 5.0;
            let distance_score = if let Some(dist) = route.distance {
                (100.0 - dist / 10.0).max(0.0)
            } else {
                50.0
            };
            let total_score = safety_score + margin_score + distance_score;

            route.safety_analysis = Some(analysis);
            route.score = Some(total_score);

            if total_score > best_score {
                best_score = total_score;
                best_route = Some(route.clone());
            }
        }

        BestRouteResult {
            best_route,
            all_routes: routes.to_vec(),
            best_score,
        }
    }

    /// Generate a detailed UKC report for a voyage
    pub fn generate_voyage_report(&self, voyage_data: &VoyageData) -> String {
        let route_analysis = &voyage_data.route_analysis;
        let ship_params = &voyage_data.ship_params;

        format!(
            r#"
╔══════════════════════════════════════════════════════════════╗
║           VOYAGE UKC SAFETY REPORT                          ║
╠══════════════════════════════════════════════════════════════╣
║ SHIP: {:<40}║
║ DEPARTURE: {:<40}║
║ DESTINATION: {:<40}║
║ ENVIRONMENT: {:<40}║
╠══════════════════════════════════════════════════════════════╣
║ ROUTE SUMMARY:                                              ║
║  Total Waypoints: {:>6}          ║
║  Unsafe Waypoints: {:>6}          ║
║  Safety Percentage: {:>6}%           ║
║  Minimum Safety Margin: {:>6} m    ║
║  Maximum Required Depth: {:>6} m ║
╠══════════════════════════════════════════════════════════════╣
║ OVERALL STATUS: {:<40}║
╚══════════════════════════════════════════════════════════════╝
"#,
            ship_params.ship_name,
            voyage_data.departure,
            voyage_data.destination,
            ship_params.environment.as_str(),
            route_analysis.total_waypoints,
            route_analysis.unsafe_waypoints,
            route_analysis.safe_percentage,
            route_analysis.min_safety_margin,
            route_analysis.max_required_depth,
            route_analysis.overall_status
        )
    }

    /// Get recommended environment type based on location
    pub fn get_recommended_environment(&self, _lat: f64, _lng: f64, distance_from_coast: f64) -> Environment {
        // Simple heuristic: within 12 nautical miles (22.2km) = Port Approach, else Coastal Water
        if distance_from_coast <= 22.2 {
            Environment::PortApproach
        } else {
            Environment::CoastalWater
        }
    }
}

// Example usage and tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_dynamic_draft() {
        let calculator = UKCCalculator::new();
        let params = DynamicDraftParams {
            static_draft: 10.0,
            draft_trim: 0.5,
            draft_listing: 0.3,
            squat: 0.8,
            wave_motion: 1.2,
        };
        let result = calculator.calculate_dynamic_draft(&params);
        // max(0.5, 0.3) + 0.8 + 1.2 = 2.5
        assert_eq!(result, 2.5);
    }

    #[test]
    fn test_calculate_ukc_requirement_port_approach() {
        let calculator = UKCCalculator::new();
        let ukc = calculator.calculate_ukc_requirement(
            &Environment::PortApproach,
            12.0,
            3.0,
        );
        // max(1.0, 0.10 * 12.0) = 1.2
        assert_eq!(ukc, 1.2);
    }

    #[test]
    fn test_calculate_ukc_requirement_coastal_water() {
        let calculator = UKCCalculator::new();
        let ukc = calculator.calculate_ukc_requirement(
            &Environment::CoastalWater,
            12.0,
            3.0,
        );
        // 0.20 * 3.0 = 0.6
        assert_eq!(ukc, 0.6);
    }

    #[test]
    fn test_determine_status_safe() {
        let calculator = UKCCalculator::new();
        let (status, margin, is_safe) = calculator.determine_status(15.0, 12.5);
        assert_eq!(status, "ACCEPTABLE");
        assert_eq!(margin, 2.5);
        assert!(is_safe);
    }

    #[test]
    fn test_determine_status_unsafe() {
        let calculator = UKCCalculator::new();
        let (status, margin, is_safe) = calculator.determine_status(10.0, 12.5);
        assert_eq!(status, "NOT ACCEPTABLE");
        assert_eq!(margin, -2.5);
        assert!(!is_safe);
    }

    #[test]
    fn test_validate_inputs_valid() {
        let mut calculator = UKCCalculator::new();
        let input = UKCInput {
            ship_name: "Test Vessel".to_string(),
            length: 200.0,
            breadth: 30.0,
            static_draft: 10.0,
            draft_trim: 0.5,
            draft_listing: 0.3,
            squat: 0.8,
            wave_motion: 1.2,
            water_depth_available: 20.0,
            environment: Environment::PortApproach,
        };
        let validation = calculator.validate_inputs(&input);
        assert!(validation.all_inputs_valid);
        assert!(validation.errors.is_empty());
    }

    #[test]
    fn test_validate_inputs_invalid() {
        let mut calculator = UKCCalculator::new();
        let input = UKCInput {
            ship_name: "".to_string(),
            length: -10.0,
            breadth: 30.0,
            static_draft: 10.0,
            draft_trim: 0.5,
            draft_listing: 0.3,
            squat: 0.8,
            wave_motion: 1.2,
            water_depth_available: 5.0, // Too shallow
            environment: Environment::PortApproach,
        };
        let validation = calculator.validate_inputs(&input);
        assert!(!validation.all_inputs_valid);
        assert!(!validation.errors.is_empty());
        // Should have at least 2 errors: ship name empty and draft exceeds depth
        assert!(validation.errors.len() >= 2);
    }

    #[test]
    fn test_calculate_full() {
        let mut calculator = UKCCalculator::new();
        let input = UKCInput {
            ship_name: "Test Vessel".to_string(),
            length: 200.0,
            breadth: 30.0,
            static_draft: 10.0,
            draft_trim: 0.5,
            draft_listing: 0.3,
            squat: 0.8,
            wave_motion: 1.2,
            water_depth_available: 20.0,
            environment: Environment::PortApproach,
        };
        let result = calculator.calculate(&input);
        assert!(result.is_valid);
        assert!(result.is_safe);
        assert_eq!(result.dynamic_draft, 2.5);
        assert_eq!(result.ukc, 1.2); // 10% of static draft = 1.0, max with 1.0 = 1.2
        assert_eq!(result.required_depth, 3.7);
        assert!(result.safety_margin > 0.0);
    }

    #[test]
    fn test_check_waypoint_safety() {
        let mut calculator = UKCCalculator::new();
        let ship_params = UKCInput {
            ship_name: "Test Vessel".to_string(),
            length: 200.0,
            breadth: 30.0,
            static_draft: 10.0,
            draft_trim: 0.5,
            draft_listing: 0.3,
            squat: 0.8,
            wave_motion: 1.2,
            water_depth_available: 20.0,
            environment: Environment::PortApproach,
        };
        let waypoint = Waypoint {
            latitude: 0.0,
            longitude: 0.0,
            depth: 15.0,
            name: Some("Test Waypoint".to_string()),
        };
        let safety = calculator.check_waypoint_safety(
            &waypoint,
            &ship_params,
            Environment::PortApproach,
        );
        assert!(safety.is_safe);
        assert_eq!(safety.depth, 15.0);
    }

    #[test]
    fn test_route_safety() {
        let mut calculator = UKCCalculator::new();
        let ship_params = UKCInput {
            ship_name: "Test Vessel".to_string(),
            length: 200.0,
            breadth: 30.0,
            static_draft: 10.0,
            draft_trim: 0.5,
            draft_listing: 0.3,
            squat: 0.8,
            wave_motion: 1.2,
            water_depth_available: 20.0,
            environment: Environment::PortApproach,
        };
        let waypoints = vec![
            Waypoint { latitude: 0.0, longitude: 0.0, depth: 15.0, name: None },
            Waypoint { latitude: 1.0, longitude: 1.0, depth: 10.0, name: None },
            Waypoint { latitude: 2.0, longitude: 2.0, depth: 5.0, name: None }, // Unsafe
        ];
        let route_safety = calculator.check_route_safety(
            &waypoints,
            &ship_params,
            Environment::PortApproach,
        );
        assert_eq!(route_safety.total_waypoints, 3);
        assert_eq!(route_safety.unsafe_waypoints, 1);
        assert_eq!(route_safety.overall_status, "UNSAFE");
    }

    #[test]
    fn test_generate_report() {
        let mut calculator = UKCCalculator::new();
        let input = UKCInput {
            ship_name: "Test Vessel".to_string(),
            length: 200.0,
            breadth: 30.0,
            static_draft: 10.0,
            draft_trim: 0.5,
            draft_listing: 0.3,
            squat: 0.8,
            wave_motion: 1.2,
            water_depth_available: 20.0,
            environment: Environment::PortApproach,
        };
        let result = calculator.calculate(&input);
        let report = calculator.generate_report(&result);
        assert!(report.contains("Test Vessel"));
        assert!(report.contains("ACCEPTABLE"));
        assert!(report.contains("UKC"));
    }
}
