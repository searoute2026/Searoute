// UKC Calculation Module - Under Keel Clearance
// Reusable module for maritime safety analysis

class UKCCalculator {
    constructor() {
        this.validations = {
            allInputsValid: false,
            errors: []
        };
    }

    /**
     * Calculate Dynamic Draft
     * @param {Object} params - Calculation parameters
     * @param {number} params.staticDraft - Static draft in meters
     * @param {number} params.draftTrim - Draft due to trim in meters
     * @param {number} params.draftListing - Draft due to listing/rolling in meters
     * @param {number} params.squat - Squat due to ship speed in meters
     * @param {number} params.waveMotion - Wave-induced motion in meters
     * @returns {number} Dynamic draft in meters
     */
    calculateDynamicDraft(params) {
        const { staticDraft, draftTrim, draftListing, squat, waveMotion } = params;
        
        // Validate inputs
        this.validateInputs(params);
        
        // Dynamic Draft = max(Draft due to Trim, Draft due to Listing) + Squat + Wave Motion
        const maxDraft = Math.max(draftTrim || 0, draftListing || 0);
        const dynamicDraft = maxDraft + (squat || 0) + (waveMotion || 0);
        
        return Math.round(dynamicDraft * 100) / 100; // Round to 2 decimal places
    }

    /**
     * Calculate UKC Requirement based on environment
     * @param {string} environment - 'Port Approach' or 'Coastal Water'
     * @param {number} staticDraft - Static draft in meters
     * @param {number} dynamicDraft - Dynamic draft in meters
     * @returns {number} UKC requirement in meters
     */
    calculateUKCRequirement(environment, staticDraft, dynamicDraft) {
        if (environment === 'Port Approach') {
            // UKC = max(1.0 meter, 10% × Static Draft)
            return Math.max(1.0, 0.10 * staticDraft);
        } else if (environment === 'Coastal Water') {
            // UKC = 20% × Dynamic Draft
            return 0.20 * dynamicDraft;
        } else {
            throw new Error('Invalid environment type. Use "Port Approach" or "Coastal Water"');
        }
    }

    /**
     * Calculate Required Water Depth
     * @param {number} dynamicDraft - Dynamic draft in meters
     * @param {number} ukc - UKC requirement in meters
     * @returns {number} Required water depth in meters
     */
    calculateRequiredDepth(dynamicDraft, ukc) {
        return Math.round((dynamicDraft + ukc) * 100) / 100;
    }

    /**
     * Determine status based on available depth
     * @param {number} availableDepth - Available water depth in meters
     * @param {number} requiredDepth - Required water depth in meters
     * @returns {Object} Status and safety margin
     */
    determineStatus(availableDepth, requiredDepth) {
        const safetyMargin = Math.round((availableDepth - requiredDepth) * 100) / 100;
        const status = safetyMargin >= 0 ? 'ACCEPTABLE' : 'NOT ACCEPTABLE';
        return {
            status,
            safetyMargin,
            isSafe: safetyMargin >= 0
        };
    }

    /**
     * Validate all input parameters
     * @param {Object} params - All calculation parameters
     * @returns {Object} Validation result
     */
    validateInputs(params) {
        const errors = [];
        const {
            shipName,
            length,
            breadth,
            staticDraft,
            draftTrim,
            draftListing,
            squat,
            waveMotion,
            waterDepthAvailable,
            environment
        } = params;

        // Check required fields
        if (!shipName || shipName.trim() === '') {
            errors.push('Ship Name is required');
        }
        if (!environment || (environment !== 'Port Approach' && environment !== 'Coastal Water')) {
            errors.push('Environment must be "Port Approach" or "Coastal Water"');
        }

        // Check numeric fields
        const numericFields = [
            { name: 'Length', value: length },
            { name: 'Breadth', value: breadth },
            { name: 'Static Draft', value: staticDraft },
            { name: 'Draft due to Trim', value: draftTrim },
            { name: 'Draft due to Listing', value: draftListing },
            { name: 'Squat due to Ship Speed', value: squat },
            { name: 'Wave-Induced Motion', value: waveMotion },
            { name: 'Water Depth Available', value: waterDepthAvailable }
        ];

        numericFields.forEach(field => {
            if (field.value === undefined || field.value === null || field.value === '') {
                errors.push(`${field.name} is required`);
            } else if (typeof field.value === 'number' && field.value < 0) {
                errors.push(`${field.name} cannot be negative`);
            } else if (typeof field.value === 'string' && isNaN(parseFloat(field.value))) {
                errors.push(`${field.name} must be a valid number`);
            }
        });

        // Validate that draft doesn't exceed water depth
        if (staticDraft && waterDepthAvailable) {
            const maxDraft = Math.max(draftTrim || 0, draftListing || 0);
            const totalDraft = maxDraft + (squat || 0) + (waveMotion || 0);
            if (totalDraft > waterDepthAvailable) {
                errors.push(`Total draft (${totalDraft}m) exceeds available water depth (${waterDepthAvailable}m)`);
            }
        }

        this.validations.errors = errors;
        this.validations.allInputsValid = errors.length === 0;
        
        return this.validations;
    }

    /**
     * Main calculation function - performs all UKC calculations
     * @param {Object} input - All input parameters
     * @returns {Object} Complete calculation results
     */
    calculate(input) {
        // Validate inputs
        const validation = this.validateInputs(input);
        if (!validation.allInputsValid) {
            return {
                errors: validation.errors,
                isValid: false,
                shipName: input.shipName || 'Unknown'
            };
        }

        // Extract parameters with defaults
        const {
            shipName,
            length = 0,
            breadth = 0,
            staticDraft = 0,
            draftTrim = 0,
            draftListing = 0,
            squat = 0,
            waveMotion = 0,
            waterDepthAvailable = 0,
            environment
        } = input;

        // Step 1: Calculate Dynamic Draft
        const dynamicDraft = this.calculateDynamicDraft({
            staticDraft,
            draftTrim,
            draftListing,
            squat,
            waveMotion
        });

        // Step 2: Calculate UKC Requirement
        const ukc = this.calculateUKCRequirement(environment, staticDraft, dynamicDraft);

        // Step 3: Calculate Required Depth
        const requiredDepth = this.calculateRequiredDepth(dynamicDraft, ukc);

        // Step 4: Determine Status
        const { status, safetyMargin, isSafe } = this.determineStatus(waterDepthAvailable, requiredDepth);

        // Return complete result
        return {
            isValid: true,
            shipName,
            length,
            breadth,
            staticDraft,
            draftTrim,
            draftListing,
            squat,
            waveMotion,
            waterDepthAvailable,
            environment,
            dynamicDraft,
            ukc: Math.round(ukc * 100) / 100,
            requiredDepth,
            status,
            safetyMargin,
            isSafe,
            // Additional useful information
            summary: {
                totalDraft: Math.round((dynamicDraft + (staticDraft || 0)) * 100) / 100,
                availableMargin: Math.round((waterDepthAvailable - requiredDepth) * 100) / 100,
                percentageMargin: waterDepthAvailable > 0 ? 
                    Math.round(((waterDepthAvailable - requiredDepth) / waterDepthAvailable) * 100) : 0
            }
        };
    }

    /**
     * Generate a human-readable report
     * @param {Object} result - Calculation result from calculate() method
     * @returns {string} Formatted report
     */
    generateReport(result) {
        if (!result.isValid) {
            return `❌ ERROR: ${result.errors.join(', ')}`;
        }

        const report = `
╔══════════════════════════════════════════════════════════════╗
║           UNDER KEEL CLEARANCE (UKC) ANALYSIS               ║
╠══════════════════════════════════════════════════════════════╣
║ SHIP: ${result.shipName.padEnd(40)}║
║ LENGTH: ${String(result.length).padStart(6)} m  BREADTH: ${String(result.breadth).padStart(6)} m ║
╠══════════════════════════════════════════════════════════════╣
║ INPUT PARAMETERS:                                           ║
║  Static Draft:           ${String(result.staticDraft).padStart(8)} m ║
║  Draft due to Trim:      ${String(result.draftTrim).padStart(8)} m ║
║  Draft due to Listing:   ${String(result.draftListing).padStart(8)} m ║
║  Squat:                  ${String(result.squat).padStart(8)} m ║
║  Wave-Induced Motion:    ${String(result.waveMotion).padStart(8)} m ║
╠══════════════════════════════════════════════════════════════╣
║ RESULTS:                                                    ║
║  Dynamic Draft:           ${String(result.dynamicDraft).padStart(8)} m ║
║  UKC Requirement:         ${String(result.ukc).padStart(8)} m ║
║  Required Depth:          ${String(result.requiredDepth).padStart(8)} m ║
║  Available Depth:         ${String(result.waterDepthAvailable).padStart(8)} m ║
╠══════════════════════════════════════════════════════════════╣
║  STATUS: ${result.status.padEnd(40)}║
║  Safety Margin:           ${String(result.safetyMargin).padStart(8)} m ║
║  Environment: ${result.environment.padEnd(46)}║
╚══════════════════════════════════════════════════════════════╝
        `;
        return report;
    }

    /**
     * Check if a route waypoint is safe based on UKC requirements
     * @param {Object} waypoint - Waypoint with coordinates and depth info
     * @param {Object} shipParams - Ship parameters
     * @param {string} environment - 'Port Approach' or 'Coastal Water'
     * @returns {Object} Safety assessment
     */
    checkWaypointSafety(waypoint, shipParams, environment) {
        const { latitude, longitude, depth } = waypoint;
        if (!depth) {
            return { isSafe: false, error: 'No depth data available for this location' };
        }

        const result = this.calculate({
            ...shipParams,
            waterDepthAvailable: depth,
            environment: environment || 'Coastal Water'
        });

        return {
            waypoint: { latitude, longitude },
            depth,
            isSafe: result.isSafe,
            status: result.status,
            safetyMargin: result.safetyMargin,
            requiredDepth: result.requiredDepth,
            dynamicDraft: result.dynamicDraft,
            ukc: result.ukc
        };
    }

    /**
     * Check multiple waypoints for UKC safety
     * @param {Array} waypoints - Array of waypoint objects with lat/lng/depth
     * @param {Object} shipParams - Ship parameters
     * @param {string} environment - 'Port Approach' or 'Coastal Water'
     * @returns {Array} Safety assessments for each waypoint
     */
    checkRouteSafety(waypoints, shipParams, environment) {
        return waypoints.map((wp, index) => {
            const safety = this.checkWaypointSafety({
                latitude: wp.lat || wp.latitude,
                longitude: wp.lng || wp.longitude,
                depth: wp.depth || wp.waterDepth || 0
            }, shipParams, environment);
            return {
                index,
                ...safety
            };
        });
    }
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = UKCCalculator;
}

// UKC Integration for SeaRoute Pro - Extends existing functionality

/**
 * UKC Integration Module - Connects UKC Calculator with SeaRoute Pro
 * This adds UKC-aware routing and depth safety features
 */
class UKCIntegration {
    constructor(calculator) {
        this.calculator = calculator || new UKCCalculator();
        this.shipParams = {
            shipName: 'Default Vessel',
            length: 0,
            breadth: 0,
            staticDraft: 0,
            draftTrim: 0,
            draftListing: 0,
            squat: 0,
            waveMotion: 0,
            environment: 'Coastal Water'
        };
        this.environmentTypes = ['Port Approach', 'Coastal Water'];
        this.calculationHistory = [];
    }

    /**
     * Update ship parameters for UKC calculations
     */
    updateShipParams(params) {
        this.shipParams = { ...this.shipParams, ...params };
    }

    /**
     * Get depth information for a route and check UKC safety
     * @param {Array} routeCoords - Array of [lat, lng] coordinates
     * @param {Function} depthProvider - Function to get depth at coordinates
     * @returns {Object} Route safety analysis
     */
    analyzeRouteSafety(routeCoords, depthProvider) {
        const results = [];
        let unsafeCount = 0;
        let minSafetyMargin = Infinity;
        let maxRequiredDepth = 0;

        for (let i = 0; i < routeCoords.length; i++) {
            const [lat, lng] = routeCoords[i];
            const depth = depthProvider ? depthProvider(lat, lng) : 999;

            const result = this.calculator.calculate({
                ...this.shipParams,
                waterDepthAvailable: depth,
                environment: this.shipParams.environment
            });

            results.push({
                index: i,
                lat,
                lng,
                depth,
                ...result,
                isSafe: result.isSafe
            });

            if (!result.isSafe) unsafeCount++;
            if (result.safetyMargin < minSafetyMargin) minSafetyMargin = result.safetyMargin;
            if (result.requiredDepth > maxRequiredDepth) maxRequiredDepth = result.requiredDepth;
        }

        return {
            totalWaypoints: routeCoords.length,
            unsafeWaypoints: unsafeCount,
            minSafetyMargin: minSafetyMargin === Infinity ? 0 : minSafetyMargin,
            maxRequiredDepth,
            results,
            overallStatus: unsafeCount === 0 ? 'SAFE' : 'UNSAFE',
            safePercentage: routeCoords.length > 0 ? 
                ((routeCoords.length - unsafeCount) / routeCoords.length * 100) : 0
        };
    }

    /**
     * Find the safest route among alternatives based on UKC
     * @param {Array} routes - Array of route objects with coordinates
     * @param {Function} depthProvider - Function to get depth at coordinates
     * @returns {Object} Best route analysis
     */
    findSafestRoute(routes, depthProvider) {
        let bestRoute = null;
        let bestScore = -Infinity;

        for (const route of routes) {
            const analysis = this.analyzeRouteSafety(route.coordinates, depthProvider);
            // Score: prioritize safety, then safety margin, then distance
            const safetyScore = analysis.safePercentage / 100 * 100;
            const marginScore = Math.min(analysis.minSafetyMargin, 20) * 5;
            const distanceScore = route.distance ? Math.max(0, 100 - route.distance / 10) : 50;
            const totalScore = safetyScore + marginScore + distanceScore;

            route.safetyAnalysis = analysis;
            route.score = totalScore;

            if (totalScore > bestScore) {
                bestScore = totalScore;
                bestRoute = route;
            }
        }

        return {
            bestRoute,
            allRoutes: routes,
            bestScore
        };
    }

    /**
     * Generate a detailed UKC report for a voyage
     * @param {Object} voyageData - Complete voyage data
     * @returns {string} Formatted report
     */
    generateVoyageReport(voyageData) {
        const { routeAnalysis, shipParams, departure, destination } = voyageData;
        
        let report = `
╔══════════════════════════════════════════════════════════════╗
║           VOYAGE UKC SAFETY REPORT                          ║
╠══════════════════════════════════════════════════════════════╣
║ SHIP: ${shipParams.shipName.padEnd(40)}║
║ DEPARTURE: ${(departure || 'Unknown').padEnd(40)}║
║ DESTINATION: ${(destination || 'Unknown').padEnd(40)}║
║ ENVIRONMENT: ${(shipParams.environment || 'N/A').padEnd(40)}║
╠══════════════════════════════════════════════════════════════╣
║ ROUTE SUMMARY:                                              ║
║  Total Waypoints: ${String(routeAnalysis.totalWaypoints).padStart(6)}          ║
║  Unsafe Waypoints: ${String(routeAnalysis.unsafeWaypoints).padStart(6)}          ║
║  Safety Percentage: ${String(routeAnalysis.safePercentage.toFixed(1)).padStart(6)}%           ║
║  Minimum Safety Margin: ${String(routeAnalysis.minSafetyMargin.toFixed(2)).padStart(6)} m    ║
║  Maximum Required Depth: ${String(routeAnalysis.maxRequiredDepth.toFixed(2)).padStart(6)} m ║
╠══════════════════════════════════════════════════════════════╣
║ OVERALL STATUS: ${routeAnalysis.overallStatus.padEnd(40)}║
╚══════════════════════════════════════════════════════════════╝
        `;

        return report;
    }

    /**
     * Get recommended environment type based on location
     * @param {number} lat - Latitude
     * @param {number} lng - Longitude
     * @param {number} distanceFromCoast - Distance from coast in km
     * @returns {string} Recommended environment
     */
    getRecommendedEnvironment(lat, lng, distanceFromCoast) {
        // Simple heuristic: within 12 nautical miles (22.2km) = Port Approach, else Coastal Water
        if (distanceFromCoast <= 22.2) {
            return 'Port Approach';
        } else {
            return 'Coastal Water';
        }
    }
}

// ============================================
// INTEGRATION WITH EXISTING SEAROUTE PRO CODE
// ============================================

// Extend the existing route generation with UKC awareness
function extendRouteWithUKC() {
    // Initialize UKC Calculator
    const ukcCalculator = new UKCCalculator();
    const ukcIntegration = new UKCIntegration(ukcCalculator);

    // Store UKC data for routes
    let currentUKCAnalysis = null;

    // Override or extend the existing route generation function
    const originalGenerateRoute = window.generateRoute || function() { console.warn('Original generateRoute not found'); };
    
    // New function that includes UKC analysis
    window.generateRouteWithUKC = async function() {
        // Check if we have ship parameters
        const length = parseFloat(document.getElementById('shipLength')?.value) || 0;
        const draft = parseFloat(document.getElementById('shipDraft')?.value) || 0;
        
        if (length === 0 || draft === 0) {
            setStatus('⚠️ Mohon isi panjang dan draft kapal terlebih dahulu', '#c0392b');
            return;
        }

        // Update ship parameters for UKC
        ukcIntegration.updateShipParams({
            shipName: 'Active Vessel',
            length: length,
            breadth: 0, // We'll get from AIS if available
            staticDraft: draft,
            draftTrim: 0,
            draftListing: 0,
            squat: 0,
            waveMotion: 0,
            environment: 'Coastal Water' // Will be determined based on location
        });

        // Get route coordinates from existing route generation
        // This would be the route generated by the existing algorithm
        const routeCoords = currentRouteCoords || [];
        
        if (routeCoords.length < 2) {
            setStatus('⚠️ Rute belum digenerate atau tidak valid', '#c0392b');
            return;
        }

        // Analyze route safety with UKC
        const analysis = ukcIntegration.analyzeRouteSafety(routeCoords, getDepthAt);
        currentUKCAnalysis = analysis;

        // Display UKC results
        displayUKCResults(analysis);

        // Show warning if unsafe waypoints found
        if (analysis.unsafeWaypoints > 0) {
            setStatus(`⚠️ ${analysis.unsafeWaypoints} waypoint tidak aman untuk dilintasi!`, '#c0392b');
            // Highlight unsafe waypoints on map
            highlightUnsafeWaypoints(analysis.results);
        } else {
            setStatus(`✅ Rute aman! Safety margin minimum: ${analysis.minSafetyMargin.toFixed(2)}m`, '#2c7a4d');
        }

        // Store for later use
        window.currentUKCAnalysis = analysis;
        
        return analysis;
    };

    // Display UKC results in the UI
    function displayUKCResults(analysis) {
        // Create or update UKC info panel
        let ukcPanel = document.getElementById('ukcPanel');
        if (!ukcPanel) {
            ukcPanel = document.createElement('div');
            ukcPanel.id = 'ukcPanel';
            ukcPanel.className = 'telemetry-card';
            ukcPanel.style.marginTop = '12px';
            const navContent = document.querySelector('.nav-content');
            const runBtn = document.getElementById('runRouteBtn');
            if (runBtn) {
                runBtn.parentNode.insertBefore(ukcPanel, runBtn.nextSibling);
            }
        }

        const statusColor = analysis.overallStatus === 'SAFE' ? '#27ae60' : '#e74c3c';
        ukcPanel.innerHTML = `
            <div class="telemetry-title">
                <span><i class="fas fa-water"></i> UKC ANALYSIS</span>
                <span class="status-badge ${analysis.overallStatus === 'SAFE' ? '' : 'danger'}">
                    ${analysis.overallStatus}
                </span>
            </div>
            <div class="telemetry-grid">
                <div class="telemetry-item">
                    <i class="fas fa-check-circle"></i>
                    <span>Safety: ${analysis.safePercentage.toFixed(1)}%</span>
                </div>
                <div class="telemetry-item">
                    <i class="fas fa-arrow-up"></i>
                    <span>Min Margin: ${analysis.minSafetyMargin.toFixed(2)}m</span>
                </div>
                <div class="telemetry-item">
                    <i class="fas fa-exclamation-triangle"></i>
                    <span>Unsafe: ${analysis.unsafeWaypoints}</span>
                </div>
                <div class="telemetry-item">
                    <i class="fas fa-ruler"></i>
                    <span>Max Req Depth: ${analysis.maxRequiredDepth.toFixed(2)}m</span>
                </div>
            </div>
        `;
        ukcPanel.style.display = 'block';
    }

    // Highlight unsafe waypoints on the map
    function highlightUnsafeWaypoints(results) {
        // Remove old highlights
        document.querySelectorAll('.ukc-unsafe-marker').forEach(el => el.remove());

        results.forEach((result, index) => {
            if (!result.isSafe) {
                // Add a warning marker on the map at the unsafe waypoint
                const marker = L.marker([result.lat, result.lng], {
                    icon: L.divIcon({
                        className: 'ukc-unsafe-marker',
                        html: `<div style="background:#e74c3c;color:white;padding:2px 8px;border-radius:12px;font-size:10px;font-weight:bold;border:2px solid #fff;">⚠️ ${result.depth.toFixed(1)}m</div>`,
                        iconSize: [60, 20],
                        iconAnchor: [30, 10]
                    })
                }).addTo(map);
                // Store for later cleanup
                if (!window._ukcWarningMarkers) window._ukcWarningMarkers = [];
                window._ukcWarningMarkers.push(marker);
            }
        });
    }

    // Add UKC check to existing waypoint hover info
    const originalWaypointInfo = window.showWaypointInfo || function(wp) {};
    window.showWaypointInfoWithUKC = function(wp) {
        const depth = getDepthAt(wp.lat, wp.lng);
        const result = ukcCalculator.calculate({
            ...ukcIntegration.shipParams,
            waterDepthAvailable: depth,
            environment: ukcIntegration.shipParams.environment
        });

        const info = `
            <b>${wp.name || 'Waypoint'}</b><br>
            📍 ${wp.lat.toFixed(4)}, ${wp.lng.toFixed(4)}<br>
            📏 Depth: ${depth.toFixed(1)}m<br>
            ⚓ Required: ${result.requiredDepth.toFixed(2)}m<br>
            ${result.isSafe ? '✅ SAFE' : '❌ UNSAFE'}<br>
            Margin: ${result.safetyMargin.toFixed(2)}m
        `;
        
        // Display this info in a tooltip or info box
        setInfo(info);
    };

    // Add function to clear UKC markers
    window.clearUKCMarkers = function() {
        if (window._ukcWarningMarkers) {
            window._ukcWarningMarkers.forEach(m => map.removeLayer(m));
            window._ukcWarningMarkers = [];
        }
        const panel = document.getElementById('ukcPanel');
        if (panel) panel.style.display = 'none';
    };

    // Expose UKC functions globally
    window.UKCCalculator = UKCCalculator;
    window.UKCIntegration = UKCIntegration;
    window.ukcIntegration = ukcIntegration;

    console.log('✅ UKC Module integrated successfully');
}

// Initialize UKC integration when the page loads
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', function() {
        // Wait for existing code to load
        setTimeout(extendRouteWithUKC, 500);
    });
} else {
    setTimeout(extendRouteWithUKC, 500);
}
