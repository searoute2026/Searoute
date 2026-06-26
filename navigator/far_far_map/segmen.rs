// ============================================
// 3-SEGMENT COLOR SYSTEM FOR UKC NAVIGATOR
// Red-Amber-Green (RAG) Status Indicator
// ============================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================
// CORE COLOR TYPES
// ============================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8, // Red (0-255)
    pub g: u8, // Green (0-255)
    pub b: u8, // Blue (0-255)
    pub a: u8, // Alpha (0-255)
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    pub fn rgba_str(&self) -> String {
        format!("rgba({}, {}, {}, {})", self.r, self.g, self.b, self.a as f64 / 255.0)
    }

    pub fn to_css(&self) -> String {
        if self.a == 255 {
            self.hex()
        } else {
            self.rgba_str()
        }
    }

    // Predefined colors
    pub fn red() -> Self {
        Self::rgb(231, 76, 60)  // #e74c3c
    }

    pub fn green() -> Self {
        Self::rgb(39, 174, 96)  // #27ae60
    }

    pub fn amber() -> Self {
        Self::rgb(243, 156, 18) // #f39c12
    }

    pub fn dark_red() -> Self {
        Self::rgb(192, 57, 43)  // #c0392b
    }

    pub fn dark_green() -> Self {
        Self::rgb(34, 153, 84)  // #229954
    }

    pub fn dark_amber() -> Self {
        Self::rgb(211, 84, 0)   // #d35400
    }

    pub fn white() -> Self {
        Self::rgb(255, 255, 255)
    }

    pub fn black() -> Self {
        Self::rgb(0, 0, 0)
    }

    pub fn gray() -> Self {
        Self::rgb(128, 128, 128)
    }

    pub fn light_gray() -> Self {
        Self::rgb(200, 200, 200)
    }

    pub fn dark_gray() -> Self {
        Self::rgb(64, 64, 64)
    }

    pub fn blue() -> Self {
        Self::rgb(52, 152, 219) // #3498db
    }

    pub fn dark_blue() -> Self {
        Self::rgb(41, 128, 185) // #2980b9
    }

    // Interpolate between two colors
    pub fn interpolate(&self, other: &Color, factor: f64) -> Self {
        let factor = factor.clamp(0.0, 1.0);
        let r = self.r as f64 + (other.r as f64 - self.r as f64) * factor;
        let g = self.g as f64 + (other.g as f64 - self.g as f64) * factor;
        let b = self.b as f64 + (other.b as f64 - self.b as f64) * factor;
        let a = self.a as f64 + (other.a as f64 - self.a as f64) * factor;
        
        Color {
            r: r.round() as u8,
            g: g.round() as u8,
            b: b.round() as u8,
            a: a.round() as u8,
        }
    }
}

// ============================================
// 3-SEGMENT COLOR SYSTEM
// ============================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SegmentStatus {
    Red,    // Unsafe / Danger
    Amber,  // Caution / Warning
    Green,  // Safe / Good
}

impl SegmentStatus {
    pub fn from_ukc(ukc_value: f64) -> Self {
        if ukc_value >= 1.0 {
            SegmentStatus::Green
        } else if ukc_value >= 0.0 {
            SegmentStatus::Amber
        } else {
            SegmentStatus::Red
        }
    }

    pub fn from_percentage(percentage: f64) -> Self {
        if percentage >= 70.0 {
            SegmentStatus::Green
        } else if percentage >= 40.0 {
            SegmentStatus::Amber
        } else {
            SegmentStatus::Red
        }
    }

    pub fn from_depth_ratio(ratio: f64) -> Self {
        // ratio = available_depth / required_depth
        if ratio >= 1.2 {
            SegmentStatus::Green
        } else if ratio >= 1.0 {
            SegmentStatus::Amber
        } else {
            SegmentStatus::Red
        }
    }

    pub fn to_color(&self) -> Color {
        match self {
            SegmentStatus::Red => Color::red(),
            SegmentStatus::Amber => Color::amber(),
            SegmentStatus::Green => Color::green(),
        }
    }

    pub fn to_dark_color(&self) -> Color {
        match self {
            SegmentStatus::Red => Color::dark_red(),
            SegmentStatus::Amber => Color::dark_amber(),
            SegmentStatus::Green => Color::dark_green(),
        }
    }

    pub fn to_icon(&self) -> &'static str {
        match self {
            SegmentStatus::Red => "🔴",
            SegmentStatus::Amber => "🟡",
            SegmentStatus::Green => "🟢",
        }
    }

    pub fn to_label(&self) -> &'static str {
        match self {
            SegmentStatus::Red => "UNSAFE",
            SegmentStatus::Amber => "CAUTION",
            SegmentStatus::Green => "SAFE",
        }
    }

    pub fn to_css_class(&self) -> &'static str {
        match self {
            SegmentStatus::Red => "status-unsafe",
            SegmentStatus::Amber => "status-caution",
            SegmentStatus::Green => "status-safe",
        }
    }

    pub fn priority(&self) -> u8 {
        match self {
            SegmentStatus::Red => 3,
            SegmentStatus::Amber => 2,
            SegmentStatus::Green => 1,
        }
    }
}

// ============================================
// SEGMENTED COLOR PALETTE
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentedPalette {
    pub red: Color,
    pub amber: Color,
    pub green: Color,
    pub red_dark: Color,
    pub amber_dark: Color,
    pub green_dark: Color,
    pub red_light: Color,
    pub amber_light: Color,
    pub green_light: Color,
    pub background: Color,
    pub text_light: Color,
    pub text_dark: Color,
}

impl SegmentedPalette {
    pub fn new() -> Self {
        Self {
            red: Color::rgb(231, 76, 60),
            amber: Color::rgb(243, 156, 18),
            green: Color::rgb(39, 174, 96),
            red_dark: Color::rgb(192, 57, 43),
            amber_dark: Color::rgb(211, 84, 0),
            green_dark: Color::rgb(34, 153, 84),
            red_light: Color::rgb(245, 183, 177),
            amber_light: Color::rgb(250, 220, 170),
            green_light: Color::rgb(170, 220, 190),
            background: Color::rgb(248, 249, 250),
            text_light: Color::rgb(255, 255, 255),
            text_dark: Color::rgb(44, 62, 80),
        }
    }

    pub fn get_color(&self, status: SegmentStatus, variant: ColorVariant) -> Color {
        match (status, variant) {
            (SegmentStatus::Red, ColorVariant::Normal) => self.red,
            (SegmentStatus::Red, ColorVariant::Dark) => self.red_dark,
            (SegmentStatus::Red, ColorVariant::Light) => self.red_light,
            (SegmentStatus::Amber, ColorVariant::Normal) => self.amber,
            (SegmentStatus::Amber, ColorVariant::Dark) => self.amber_dark,
            (SegmentStatus::Amber, ColorVariant::Light) => self.amber_light,
            (SegmentStatus::Green, ColorVariant::Normal) => self.green,
            (SegmentStatus::Green, ColorVariant::Dark) => self.green_dark,
            (SegmentStatus::Green, ColorVariant::Light) => self.green_light,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorVariant {
    Normal,
    Dark,
    Light,
}

// ============================================
// SEGMENTED UKC ANALYZER
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentedUKCAnalysis {
    pub status: SegmentStatus,
    pub color: Color,
    pub ukc_value: f64,
    pub segment: String,
    pub label: String,
    pub icon: String,
    pub css_class: String,
    pub recommendations: Vec<String>,
    pub metrics: SegmentedMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentedMetrics {
    pub safety_margin: f64,
    pub depth_ratio: f64,
    pub risk_level: String,
    pub confidence: f64,
    pub trend: TrendDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Deteriorating,
}

impl TrendDirection {
    pub fn to_color(&self) -> Color {
        match self {
            TrendDirection::Improving => Color::green(),
            TrendDirection::Stable => Color::amber(),
            TrendDirection::Deteriorating => Color::red(),
        }
    }

    pub fn to_icon(&self) -> &'static str {
        match self {
            TrendDirection::Improving => "📈",
            TrendDirection::Stable => "➡️",
            TrendDirection::Deteriorating => "📉",
        }
    }
}

pub struct SegmentedUKCAnalyzer {
    palette: SegmentedPalette,
    thresholds: SegmentedThresholds,
}

#[derive(Debug, Clone)]
pub struct SegmentedThresholds {
    pub green_min: f64,
    pub amber_min: f64,
    pub red_max: f64,
    pub green_depth_ratio: f64,
    pub amber_depth_ratio: f64,
}

impl Default for SegmentedThresholds {
    fn default() -> Self {
        Self {
            green_min: 1.0,      // UKC >= 1.0m = Green
            amber_min: 0.0,      // UKC >= 0.0m = Amber
            red_max: -0.1,       // UKC < 0.0m = Red
            green_depth_ratio: 1.2,  // Depth >= 120% required = Green
            amber_depth_ratio: 1.0,  // Depth >= 100% required = Amber
        }
    }
}

impl SegmentedUKCAnalyzer {
    pub fn new() -> Self {
        Self {
            palette: SegmentedPalette::new(),
            thresholds: SegmentedThresholds::default(),
        }
    }

    pub fn with_thresholds(thresholds: SegmentedThresholds) -> Self {
        Self {
            palette: SegmentedPalette::new(),
            thresholds,
        }
    }

    pub fn analyze(&self, ukc_value: f64, depth: f64, required_depth: f64) -> SegmentedUKCAnalysis {
        let status = SegmentStatus::from_ukc(ukc_value);
        let color = status.to_color();
        let depth_ratio = if required_depth > 0.0 {
            depth / required_depth
        } else {
            1.0
        };

        let risk_level = match status {
            SegmentStatus::Red => "HIGH RISK",
            SegmentStatus::Amber => "MODERATE RISK",
            SegmentStatus::Green => "LOW RISK",
        };

        let mut recommendations = Vec::new();
        match status {
            SegmentStatus::Red => {
                recommendations.push("🚨 UKC is insufficient - DO NOT PROCEED".to_string());
                recommendations.push("📋 Find alternative route with deeper water".to_string());
                recommendations.push("⏰ Wait for higher tide if possible".to_string());
                recommendations.push("⚓ Reduce draft by discharging cargo".to_string());
                recommendations.push("📊 Consult with port authority immediately".to_string());
            }
            SegmentStatus::Amber => {
                recommendations.push("⚠️ UKC is marginal - proceed with caution".to_string());
                recommendations.push("🔄 Reduce speed to minimize squat".to_string());
                recommendations.push("📈 Monitor depth continuously".to_string());
                recommendations.push("🛑 Have contingency plan ready".to_string());
                recommendations.push("📞 Keep VHF channel open for emergencies".to_string());
            }
            SegmentStatus::Green => {
                recommendations.push("✅ UKC is sufficient - safe to proceed".to_string());
                recommendations.push("📊 Continue monitoring depth".to_string());
                recommendations.push("🚢 Maintain safe speed".to_string());
                recommendations.push("📝 Log UKC readings regularly".to_string());
                recommendations.push("🔄 Update navigation plan as needed".to_string());
            }
        }

        // Determine trend based on UKC value
        let trend = if ukc_value >= 2.0 {
            TrendDirection::Improving
        } else if ukc_value >= 1.0 {
            TrendDirection::Stable
        } else {
            TrendDirection::Deteriorating
        };

        SegmentedUKCAnalysis {
            status,
            color,
            ukc_value,
            segment: format!("UKC: {:.2}m", ukc_value),
            label: status.to_label().to_string(),
            icon: status.to_icon().to_string(),
            css_class: status.to_css_class().to_string(),
            recommendations,
            metrics: SegmentedMetrics {
                safety_margin: ukc_value,
                depth_ratio,
                risk_level: risk_level.to_string(),
                confidence: self.calculate_confidence(ukc_value, depth_ratio),
                trend,
            },
        }
    }

    pub fn analyze_waypoint(&self, ukc_value: f64, depth: f64, required_depth: f64, waypoint_name: &str) -> SegmentedWaypointAnalysis {
        let analysis = self.analyze(ukc_value, depth, required_depth);
        
        SegmentedWaypointAnalysis {
            waypoint_name: waypoint_name.to_string(),
            analysis,
            formatted_report: self.format_waypoint_report(waypoint_name, &analysis),
        }
    }

    pub fn analyze_route(&self, waypoints: &[(f64, f64, f64, f64)]) -> SegmentedRouteAnalysis {
        let mut analyses = Vec::new();
        let mut unsafe_count = 0;
        let mut caution_count = 0;
        let mut safe_count = 0;
        let mut min_ukc = f64::INFINITY;
        let mut max_ukc = f64::NEG_INFINITY;
        let mut total_ukc = 0.0;

        for (i, (ukc, depth, req_depth, _)) in waypoints.iter().enumerate() {
            let analysis = self.analyze(*ukc, *depth, *req_depth);
            
            match analysis.status {
                SegmentStatus::Red => unsafe_count += 1,
                SegmentStatus::Amber => caution_count += 1,
                SegmentStatus::Green => safe_count += 1,
            }
            
            min_ukc = min_ukc.min(*ukc);
            max_ukc = max_ukc.max(*ukc);
            total_ukc += *ukc;
            
            analyses.push(analysis);
        }

        let total_waypoints = waypoints.len();
        let avg_ukc = if total_waypoints > 0 { total_ukc / total_waypoints as f64 } else { 0.0 };
        let safe_percentage = if total_waypoints > 0 {
            (safe_count as f64 / total_waypoints as f64) * 100.0
        } else {
            0.0
        };

        let overall_status = if unsafe_count > 0 {
            SegmentStatus::Red
        } else if caution_count > 0 {
            SegmentStatus::Amber
        } else {
            SegmentStatus::Green
        };

        SegmentedRouteAnalysis {
            waypoints: analyses,
            overall_status,
            unsafe_count,
            caution_count,
            safe_count,
            total_waypoints,
            min_ukc,
            max_ukc,
            avg_ukc,
            safe_percentage,
            recommendations: self.generate_route_recommendations(unsafe_count, caution_count, safe_count),
        }
    }

    fn calculate_confidence(&self, ukc_value: f64, depth_ratio: f64) -> f64 {
        // Confidence based on UKC margin and depth ratio
        let ukc_confidence = if ukc_value >= 2.0 {
            0.95
        } else if ukc_value >= 1.0 {
            0.80
        } else if ukc_value >= 0.0 {
            0.60
        } else {
            0.30
        };
        
        let depth_confidence = if depth_ratio >= 1.5 {
            0.95
        } else if depth_ratio >= 1.2 {
            0.85
        } else if depth_ratio >= 1.0 {
            0.70
        } else {
            0.40
        };
        
        (ukc_confidence + depth_confidence) / 2.0
    }

    fn generate_route_recommendations(&self, unsafe: usize, caution: usize, safe: usize) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if unsafe > 0 {
            recommendations.push(format!("🚨 {} waypoint(s) UNSAFE - route is not recommended", unsafe));
            recommendations.push("📍 Identify and avoid unsafe waypoints".to_string());
            recommendations.push("🔄 Find alternative route with better depth".to_string());
        }
        
        if caution > 0 {
            recommendations.push(format!("⚠️ {} waypoint(s) require CAUTION", caution));
            recommendations.push("📊 Monitor UKC closely at these waypoints".to_string());
            recommendations.push("🐌 Reduce speed through caution zones".to_string());
        }
        
        if safe > 0 && unsafe == 0 {
            recommendations.push(format!("✅ {} waypoint(s) are SAFE", safe));
            if caution > 0 {
                recommendations.push("🚶 Proceed with caution through amber zones".to_string());
            } else {
                recommendations.push("🎯 Route is safe - proceed as planned".to_string());
            }
        }
        
        if recommendations.is_empty() {
            recommendations.push("📋 No waypoints analyzed".to_string());
        }
        
        recommendations
    }

    fn format_waypoint_report(&self, name: &str, analysis: &SegmentedUKCAnalysis) -> String {
        format!(
            r#"
╔═══════════════════════════════════════════════════════════╗
║                WAYPOINT STATUS REPORT                    ║
╠═══════════════════════════════════════════════════════════╣
║ Name: {:<42}║
║ Status: {:<38}║
║ UKC: {:.2}m {:<30}║
║ Risk Level: {:<36}║
║ Confidence: {:.0}% {:<30}║
║ Trend: {:<40}║
╠═══════════════════════════════════════════════════════════╣
║ RECOMMENDATIONS:                                         ║
{}
╚═══════════════════════════════════════════════════════════╝
"#,
            name,
            format!("{} {} {}", analysis.icon, analysis.label, analysis.css_class),
            analysis.ukc_value,
            analysis.segment,
            analysis.metrics.risk_level,
            analysis.metrics.confidence * 100.0,
            "",
            format!("{} {}", analysis.metrics.trend.to_icon(), format!("{:?}", analysis.metrics.trend)),
            analysis.recommendations.iter()
                .map(|r| format!("  {}", r))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

// ============================================
// SEGMENTED ANALYSIS RESULT TYPES
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentedWaypointAnalysis {
    pub waypoint_name: String,
    pub analysis: SegmentedUKCAnalysis,
    pub formatted_report: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentedRouteAnalysis {
    pub waypoints: Vec<SegmentedUKCAnalysis>,
    pub overall_status: SegmentStatus,
    pub unsafe_count: usize,
    pub caution_count: usize,
    pub safe_count: usize,
    pub total_waypoints: usize,
    pub min_ukc: f64,
    pub max_ukc: f64,
    pub avg_ukc: f64,
    pub safe_percentage: f64,
    pub recommendations: Vec<String>,
}

impl SegmentedRouteAnalysis {
    pub fn to_html(&self) -> String {
        let status_color = self.overall_status.to_color().hex();
        let status_label = self.overall_status.to_label();
        let status_icon = self.overall_status.to_icon();
        let status_class = self.overall_status.to_css_class();
        
        let mut html = String::new();
        html.push_str(&format!(
            r#"
            <div class="route-analysis">
                <div class="route-status {0}">
                    <span class="status-icon">{1}</span>
                    <span class="status-label">{2}</span>
                    <span class="status-detail">{3:.1}% safe</span>
                </div>
                <div class="route-metrics">
                    <div class="metric">
                        <span class="metric-label">Waypoints</span>
                        <span class="metric-value">{4}</span>
                    </div>
                    <div class="metric">
                        <span class="metric-label">🟢 Safe</span>
                        <span class="metric-value">{5}</span>
                    </div>
                    <div class="metric">
                        <span class="metric-label">🟡 Caution</span>
                        <span class="metric-value">{6}</span>
                    </div>
                    <div class="metric">
                        <span class="metric-label">🔴 Unsafe</span>
                        <span class="metric-value">{7}</span>
                    </div>
                    <div class="metric">
                        <span class="metric-label">Min UKC</span>
                        <span class="metric-value">{8:.2}m</span>
                    </div>
                    <div class="metric">
                        <span class="metric-label">Avg UKC</span>
                        <span class="metric-value">{9:.2}m</span>
                    </div>
                </div>
                <div class="route-recommendations">
                    <h4>Recommendations</h4>
                    <ul>
                        {10}
                    </ul>
                </div>
            </div>
            "#,
            status_class,
            status_icon,
            status_label,
            self.safe_percentage,
            self.total_waypoints,
            self.safe_count,
            self.caution_count,
            self.unsafe_count,
            self.min_ukc,
            self.avg_ukc,
            self.recommendations.iter()
                .map(|r| format!("<li>{}</li>", r))
                .collect::<Vec<_>>()
                .join("\n")
        ));
        
        html
    }

    pub fn to_svg(&self) -> String {
        let width = 800;
        let height = 400;
        let bar_width = (width - 100) / self.total_waypoints.max(1) as f64;
        
        let mut svg = String::new();
        svg.push_str(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
            width, height, width, height
        ));
        
        // Background
        svg.push_str(&format!(
            r#"<rect width="{}" height="{}" fill="{}"/>"#,
            width, height, "#f8f9fa"
        ));
        
        // Title
        svg.push_str(&format!(
            r#"<text x="{}" y="30" font-family="Arial" font-size="20" font-weight="bold" text-anchor="middle" fill="#2c3e50">Route UKC Analysis - {} ({:.1}% Safe)</text>"#,
            width / 2,
            self.overall_status.to_label(),
            self.safe_percentage
        ));
        
        // Draw bars
        let mut x = 50;
        for (i, waypoint) in self.waypoints.iter().enumerate() {
            let color = waypoint.status.to_color().hex();
            let height_value = 50.0 + (waypoint.ukc_value / 5.0).min(300.0);
            let y = height - 50 - height_value as i32;
            
            svg.push_str(&format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" rx="2" ry="2">
                    <title>WP{}: {:.2}m - {}</title>
                </rect>"#,
                x, y, bar_width - 4, height_value, color,
                i + 1, waypoint.ukc_value, waypoint.status.to_label()
            ));
            
            // Label
            svg.push_str(&format!(
                r#"<text x="{}" y="{}" font-family="Arial" font-size="8" text-anchor="middle" fill="#666" transform="rotate(-45, {}, {})">WP{}</text>"#,
                x + bar_width / 2.0,
                height - 10,
                x + bar_width / 2.0,
                height - 10,
                i + 1
            ));
            
            x += bar_width as i32;
        }
        
        // Legend
        svg.push_str(r#"
            <g transform="translate(600, 350)">
                <rect x="0" y="0" width="12" height="12" fill="#e74c3c" rx="2"/>
                <text x="16" y="10" font-family="Arial" font-size="10" fill="#2c3e50">Unsafe</text>
                <rect x="80" y="0" width="12" height="12" fill="#f39c12" rx="2"/>
                <text x="96" y="10" font-family="Arial" font-size="10" fill="#2c3e50">Caution</text>
                <rect x="160" y="0" width="12" height="12" fill="#27ae60" rx="2"/>
                <text x="176" y="10" font-family="Arial" font-size="10" fill="#2c3e50">Safe</text>
            </g>
        "#);
        
        svg.push_str("</svg>");
        svg
    }
}

// ============================================
// TESTING
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_creation() {
        let red = Color::red();
        assert_eq!(red.r, 231);
        assert_eq!(red.g, 76);
        assert_eq!(red.b, 60);
        assert_eq!(red.a, 255);
        assert_eq!(red.hex(), "#e74c3c");
    }

    #[test]
    fn test_segment_status_from_ukc() {
        assert_eq!(SegmentStatus::from_ukc(2.0), SegmentStatus::Green);
        assert_eq!(SegmentStatus::from_ukc(0.5), SegmentStatus::Amber);
        assert_eq!(SegmentStatus::from_ukc(-0.5), SegmentStatus::Red);
    }

    #[test]
    fn test_segment_status_to_color() {
        assert_eq!(SegmentStatus::Red.to_color().hex(), "#e74c3c");
        assert_eq!(SegmentStatus::Amber.to_color().hex(), "#f39c12");
        assert_eq!(SegmentStatus::Green.to_color().hex(), "#27ae60");
    }

    #[test]
    fn test_analyzer() {
        let analyzer = SegmentedUKCAnalyzer::new();
        let analysis = analyzer.analyze(0.5, 12.0, 10.0);
        
        assert_eq!(analysis.status, SegmentStatus::Amber);
        assert_eq!(analysis.label, "CAUTION");
        assert_eq!(analysis.ukc_value, 0.5);
        assert!(!analysis.recommendations.is_empty());
    }

    #[test]
    fn test_route_analysis() {
        let analyzer = SegmentedUKCAnalyzer::new();
        let waypoints = vec![
            (2.5, 15.0, 10.0, 0.0),
            (0.5, 12.0, 10.0, 0.0),
            (-0.5, 8.0, 10.0, 0.0),
            (1.5, 14.0, 10.0, 0.0),
        ];
        
        let route_analysis = analyzer.analyze_route(&waypoints);
        assert_eq!(route_analysis.total_waypoints, 4);
        assert_eq!(route_analysis.unsafe_count, 1);
        assert_eq!(route_analysis.caution_count, 1);
        assert_eq!(route_analysis.safe_count, 2);
        assert_eq!(route_analysis.overall_status, SegmentStatus::Red);
        assert_eq!(route_analysis.min_ukc, -0.5);
    }

    #[test]
    fn test_color_interpolation() {
        let red = Color::red();
        let green = Color::green();
        let mixed = red.interpolate(&green, 0.5);
        
        assert_eq!(mixed.r, (231.0 + 39.0) / 2.0 as u8);
        assert_eq!(mixed.g, (76.0 + 174.0) / 2.0 as u8);
        assert_eq!(mixed.b, (60.0 + 96.0) / 2.0 as u8);
    }

    #[test]
    fn test_segmented_palette() {
        let palette = SegmentedPalette::new();
        let color = palette.get_color(SegmentStatus::Red, ColorVariant::Normal);
        assert_eq!(color.hex(), "#e74c3c");
        
        let color = palette.get_color(SegmentStatus::Green, ColorVariant::Dark);
        assert_eq!(color.hex(), "#229954");
    }

    #[test]
    fn test_waypoint_report() {
        let analyzer = SegmentedUKCAnalyzer::new();
        let analysis = analyzer.analyze(1.2, 15.0, 10.0);
        let waypoint_analysis = SegmentedWaypointAnalysis {
            waypoint_name: "Test WP".to_string(),
            analysis,
            formatted_report: "".to_string(),
        };
        
        let report = analyzer.format_waypoint_report(&waypoint_analysis.waypoint_name, &waypoint_analysis.analysis);
        assert!(report.contains("Test WP"));
        assert!(report.contains("SAFE"));
        assert!(report.contains("UKC: 1.20m"));
    }
}

// ============================================
// INTEGRATION WITH UKC CALCULATOR
// ============================================

pub trait UKCAnalyzerExt {
    fn analyze_with_segments(&self, ukc_value: f64, depth: f64, required_depth: f64) -> SegmentedUKCAnalysis;
    fn get_status_color(&self, status: SegmentStatus) -> Color;
    fn get_route_summary(&self, analyses: &[SegmentedUKCAnalysis]) -> SegmentedRouteAnalysis;
}

impl UKCAnalyzerExt for SegmentedUKCAnalyzer {
    fn analyze_with_segments(&self, ukc_value: f64, depth: f64, required_depth: f64) -> SegmentedUKCAnalysis {
        self.analyze(ukc_value, depth, required_depth)
    }

    fn get_status_color(&self, status: SegmentStatus) -> Color {
        status.to_color()
    }

    fn get_route_summary(&self, analyses: &[SegmentedUKCAnalysis]) -> SegmentedRouteAnalysis {
        let waypoints: Vec<(f64, f64, f64, f64)> = analyses.iter()
            .map(|a| (a.ukc_value, 0.0, 0.0, 0.0))
            .collect();
        
        self.analyze_route(&waypoints)
    }
}

// ============================================
// CSS STYLES FOR 3-SEGMENT SYSTEM
// ============================================

pub fn get_css_styles() -> String {
    r#"
    /* 3-Segment Color System Styles */
    
    .status-safe {
        color: #27ae60;
        background-color: rgba(39, 174, 96, 0.1);
        border-color: #27ae60;
    }
    
    .status-caution {
        color: #f39c12;
        background-color: rgba(243, 156, 18, 0.1);
        border-color: #f39c12;
    }
    
    .status-unsafe {
        color: #e74c3c;
        background-color: rgba(231, 76, 60, 0.1);
        border-color: #e74c3c;
    }
    
    .status-badge {
        display: inline-block;
        padding: 4px 12px;
        border-radius: 12px;
        font-weight: bold;
        font-size: 12px;
        border: 2px solid;
        text-transform: uppercase;
    }
    
    .status-indicator {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 10px 15px;
        border-radius: 8px;
        margin: 5px 0;
    }
    
    .status-dot {
        width: 12px;
        height: 12px;
        border-radius: 50%;
        display: inline-block;
        flex-shrink: 0;
    }
    
    .status-dot.green { background-color: #27ae60; }
    .status-dot.amber { background-color: #f39c12; }
    .status-dot.red { background-color: #e74c3c; }
    
    .route-analysis {
        background: white;
        border-radius: 12px;
        padding: 20px;
        box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        margin: 10px 0;
    }
    
    .route-status {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 10px 15px;
        border-radius: 8px;
        margin-bottom: 15px;
        font-weight: bold;
    }
    
    .route-status.status-safe {
        background: rgba(39, 174, 96, 0.15);
        border-left: 4px solid #27ae60;
    }
    
    .route-status.status-caution {
        background: rgba(243, 156, 18, 0.15);
        border-left: 4px solid #f39c12;
    }
    
    .route-status.status-unsafe {
        background: rgba(231, 76, 60, 0.15);
        border-left: 4px solid #e74c3c;
    }
    
    .route-metrics {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
        gap: 10px;
        margin: 15px 0;
    }
    
    .metric {
        background: #f8f9fa;
        padding: 8px 12px;
        border-radius: 6px;
        text-align: center;
    }
    
    .metric-label {
        display: block;
        font-size: 11px;
        color: #7f8c8d;
        margin-bottom: 2px;
    }
    
    .metric-value {
        display: block;
        font-size: 16px;
        font-weight: bold;
        color: #2c3e50;
    }
    
    .route-recommendations {
        margin-top: 15px;
        padding-top: 15px;
        border-top: 1px solid #eee;
    }
    
    .route-recommendations h4 {
        margin: 0 0 10px 0;
        color: #2c3e50;
        font-size: 14px;
    }
    
    .route-recommendations ul {
        margin: 0;
        padding-left: 20px;
    }
    
    .route-recommendations li {
        margin: 4px 0;
        font-size: 13px;
        color: #34495e;
    }
    
    .waypoint-status {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 6px 10px;
        border-radius: 4px;
        font-size: 12px;
        font-weight: 500;
    }
    
    /* Progress bar with segments */
    .segmented-progress {
        display: flex;
        height: 8px;
        border-radius: 4px;
        overflow: hidden;
        background: #ecf0f1;
        margin: 5px 0;
    }
    
    .segmented-progress .segment {
        height: 100%;
        transition: width 0.3s ease;
    }
    
    .segmented-progress .segment-safe {
        background: #27ae60;
    }
    
    .segmented-progress .segment-caution {
        background: #f39c12;
    }
    
    .segmented-progress .segment-unsafe {
        background: #e74c3c;
    }
    
    /* Responsive adjustments */
    @media (max-width: 768px) {
        .route-metrics {
            grid-template-columns: repeat(2, 1fr);
        }
        
        .route-status {
            flex-direction: column;
            align-items: flex-start;
            gap: 5px;
        }
    }
    "#.to_string()
}

// ============================================
// EXPORT
// ============================================

pub use {
    Color,
    SegmentStatus,
    SegmentedPalette,
    SegmentedUKCAnalyzer,
    SegmentedUKCAnalysis,
    SegmentedWaypointAnalysis,
    SegmentedRouteAnalysis,
    SegmentedThresholds,
    ColorVariant,
    TrendDirection,
    SegmentedMetrics,
    UKCAnalyzerExt,
};
