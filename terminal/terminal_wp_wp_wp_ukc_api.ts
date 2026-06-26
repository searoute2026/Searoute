// ============================================
// MARINE ROUTER - PURE ASSEMBLYSCRIPT (WASM)
// Complete Navigation System in AssemblyScript
// ============================================

// ============================================
// CORE GEOSPATIAL TYPES
// ============================================

export class Coordinate {
  lat: f64;
  lng: f64;

  constructor(lat: f64, lng: f64) {
    this.lat = lat;
    this.lng = lng;
  }

  distance(other: Coordinate): f64 {
    const dlat = this.lat - other.lat;
    const dlng = this.lng - other.lng;
    return Math.sqrt(dlat * dlat + dlng * dlng);
  }

  distanceKm(other: Coordinate): f64 {
    const R: f64 = 6371.0;
    const dlat = (other.lat - this.lat) * 0.017453292519943295;
    const dlng = (other.lng - this.lng) * 0.017453292519943295;
    const lat1 = this.lat * 0.017453292519943295;
    const lat2 = other.lat * 0.017453292519943295;
    const a = Math.sin(dlat / 2.0) * Math.sin(dlat / 2.0) +
              Math.cos(lat1) * Math.cos(lat2) *
              Math.sin(dlng / 2.0) * Math.sin(dlng / 2.0);
    const c = 2.0 * Math.atan2(Math.sqrt(a), Math.sqrt(1.0 - a));
    return R * c;
  }

  toArray(): Float64Array {
    const arr = new Float64Array(2);
    arr[0] = this.lat;
    arr[1] = this.lng;
    return arr;
  }

  isValid(): bool {
    return this.lat >= -90.0 && this.lat <= 90.0 &&
           this.lng >= -180.0 && this.lng <= 180.0;
  }
}

export class BoundingBox {
  minLat: f64;
  maxLat: f64;
  minLng: f64;
  maxLng: f64;

  constructor(minLat: f64, maxLat: f64, minLng: f64, maxLng: f64) {
    this.minLat = minLat;
    this.maxLat = maxLat;
    this.minLng = minLng;
    this.maxLng = maxLng;
  }

  contains(coord: Coordinate): bool {
    return coord.lat >= this.minLat && coord.lat <= this.maxLat &&
           coord.lng >= this.minLng && coord.lng <= this.maxLng;
  }
}

// ============================================
// ISLAND SYSTEM
// ============================================

export class Island {
  name: string;
  polygon: Coordinate[];
  bbox: BoundingBox | null;

  constructor(name: string, polygon: Coordinate[]) {
    this.name = name;
    this.polygon = polygon;
    this.bbox = this.calculateBBox();
  }

  calculateBBox(): BoundingBox | null {
    if (this.polygon.length === 0) return null;
    
    let minLat = this.polygon[0].lat;
    let maxLat = this.polygon[0].lat;
    let minLng = this.polygon[0].lng;
    let maxLng = this.polygon[0].lng;

    for (let i = 1; i < this.polygon.length; i++) {
      const p = this.polygon[i];
      if (p.lat < minLat) minLat = p.lat;
      if (p.lat > maxLat) maxLat = p.lat;
      if (p.lng < minLng) minLng = p.lng;
      if (p.lng > maxLng) maxLng = p.lng;
    }

    return new BoundingBox(
      minLat - 0.02,
      maxLat + 0.02,
      minLng - 0.02,
      maxLng + 0.02
    );
  }

  pointInPolygon(point: Coordinate): bool {
    let inside = false;
    const n = this.polygon.length;
    
    for (let i = 0, j = n - 1; i < n; j = i++) {
      const xi = this.polygon[i].lat;
      const yi = this.polygon[i].lng;
      const xj = this.polygon[j].lat;
      const yj = this.polygon[j].lng;
      
      if ((yi > point.lat) !== (yj > point.lat) &&
          (point.lng < (xj - xi) * (point.lat - yi) / (yj - yi) + xi)) {
        inside = !inside;
      }
    }
    
    return inside;
  }
}

// ============================================
// WAYPOINT SYSTEM
// ============================================

export class Waypoint {
  lat: f64;
  lng: f64;
  name: string;

  constructor(lat: f64, lng: f64, name: string = "") {
    this.lat = lat;
    this.lng = lng;
    this.name = name;
  }

  toCoordinate(): Coordinate {
    return new Coordinate(this.lat, this.lng);
  }
}

export class WaypointGraph {
  edges: Map<usize, usize[]> = new Map();

  addEdge(from: usize, to: usize): void {
    if (!this.edges.has(from)) {
      this.edges.set(from, []);
    }
    const fromEdges = this.edges.get(from);
    if (!fromEdges.includes(to)) {
      fromEdges.push(to);
    }
  }

  getNeighbors(node: usize): usize[] {
    return this.edges.get(node) || [];
  }
}

// ============================================
// NAVIGATION ENGINE
// ============================================

export class NavigationEngine {
  private islands: Island[] = [];
  private waypoints: Waypoint[] = [];
  private graph: WaypointGraph = new WaypointGraph();
  private wpReady: bool = false;
  private islandsReady: bool = false;

  // Constants
  private readonly WP_CONNECT_RADIUS: f64 = 3.0;
  private readonly WP_K_NEAR: i32 = 9;
  private readonly MAX_ITER: i32 = 220000;

  constructor() {}

  // ========== ISLAND METHODS ==========

  addIsland(island: Island): void {
    this.islands.push(island);
    this.islandsReady = true;
  }

  addIslands(islands: Island[]): void {
    for (let i = 0; i < islands.length; i++) {
      this.islands.push(islands[i]);
    }
    this.islandsReady = true;
  }

  isBlocked(coord: Coordinate): bool {
    for (let i = 0; i < this.islands.length; i++) {
      const island = this.islands[i];
      
      // BBox quick check
      if (island.bbox && !island.bbox.contains(coord)) {
        continue;
      }
      
      if (island.pointInPolygon(coord)) {
        return true;
      }
    }
    return false;
  }

  canSee(a: Coordinate, b: Coordinate, samples: i32 = 26): bool {
    for (let i = 1; i < samples; i++) {
      const t = i as f64 / samples as f64;
      const lat = a.lat + (b.lat - a.lat) * t;
      const lng = a.lng + (b.lng - a.lng) * t;
      if (this.isBlocked(new Coordinate(lat, lng))) {
        return false;
      }
    }
    return true;
  }

  // ========== WAYPOINT METHODS ==========

  addWaypoint(waypoint: Waypoint): void {
    this.waypoints.push(waypoint);
  }

  addWaypoints(waypoints: Waypoint[]): void {
    for (let i = 0; i < waypoints.length; i++) {
      this.waypoints.push(waypoints[i]);
    }
  }

  buildGraph(): void {
    if (this.waypoints.length === 0) return;

    this.graph = new WaypointGraph();

    for (let i = 0; i < this.waypoints.length; i++) {
      const dists: { index: usize; dist: f64 }[] = [];
      const a = new Coordinate(this.waypoints[i].lat, this.waypoints[i].lng);

      for (let j = 0; j < this.waypoints.length; j++) {
        if (i === j) continue;
        const b = new Coordinate(this.waypoints[j].lat, this.waypoints[j].lng);
        const d = a.distance(b);
        if (d <= this.WP_CONNECT_RADIUS) {
          dists.push({ index: j, dist: d });
        }
      }

      // Sort by distance
      dists.sort((a, b) => {
        if (a.dist < b.dist) return -1;
        if (a.dist > b.dist) return 1;
        return 0;
      });

      // Take top K
      const count = dists.length < this.WP_K_NEAR ? dists.length : this.WP_K_NEAR;
      for (let k = 0; k < count; k++) {
        const j = dists[k].index;
        const aCoord = new Coordinate(this.waypoints[i].lat, this.waypoints[i].lng);
        const bCoord = new Coordinate(this.waypoints[j].lat, this.waypoints[j].lng);
        
        if (this.islands.length === 0 || this.canSee(aCoord, bCoord, 18)) {
          this.graph.addEdge(i, j);
          this.graph.addEdge(j, i);
        }
      }
    }

    this.wpReady = true;
  }

  findNearestWaypoints(coord: Coordinate, k: i32 = 10): { index: usize; dist: f64 }[] {
    const results: { index: usize; dist: f64 }[] = [];
    
    for (let i = 0; i < this.waypoints.length; i++) {
      const wp = new Coordinate(this.waypoints[i].lat, this.waypoints[i].lng);
      const d = coord.distance(wp);
      results.push({ index: i, dist: d });
    }

    results.sort((a, b) => {
      if (a.dist < b.dist) return -1;
      if (a.dist > b.dist) return 1;
      return 0;
    });

    const count = results.length < k ? results.length : k;
    return results.slice(0, count as i32);
  }

  pickReachable(
    coord: Coordinate,
    candidates: { index: usize; dist: f64 }[]
  ): { index: usize; dist: f64 }[] {
    const reachable: { index: usize; dist: f64 }[] = [];

    for (let i = 0; i < candidates.length; i++) {
      const wp = new Coordinate(
        this.waypoints[candidates[i].index].lat,
        this.waypoints[candidates[i].index].lng
      );
      if (this.canSee(coord, wp, 24)) {
        reachable.push(candidates[i]);
      }
    }

    if (reachable.length >= 1) {
      const count = reachable.length < 4 ? reachable.length : 4;
      return reachable.slice(0, count);
    } else {
      const count = candidates.length < 2 ? candidates.length : 2;
      return candidates.slice(0, count);
    }
  }

  // ========== A* PATHFINDING ==========

  private heuristic(a: Coordinate, b: Coordinate): f64 {
    return a.distanceKm(b);
  }

  astar(
    startId: string,
    endId: string,
    extraEdges: Map<string, string[]>,
    coordOf: (id: string) => Coordinate
  ): string[] | null {
    const openSet: string[] = [startId];
    const cameFrom: Map<string, string> = new Map();
    const gScore: Map<string, f64> = new Map();
    const fScore: Map<string, f64> = new Map();
    const visited: Set<string> = new Set();

    gScore.set(startId, 0.0);
    fScore.set(startId, this.heuristic(coordOf(startId), coordOf(endId)));

    let iterations = 0;

    while (openSet.length > 0 && iterations < this.MAX_ITER) {
      iterations++;
      
      // Find node with lowest fScore
      let current = openSet[0];
      let currentIdx = 0;
      for (let i = 1; i < openSet.length; i++) {
        const fCurrent = fScore.get(openSet[i]) || Infinity;
        const fBest = fScore.get(current) || Infinity;
        if (fCurrent < fBest) {
          current = openSet[i];
          currentIdx = i;
        }
      }

      // Remove current from openSet
      openSet.splice(currentIdx, 1);

      if (current === endId) {
        // Reconstruct path
        const path: string[] = [];
        let c = current;
        while (cameFrom.has(c)) {
          path.push(c);
          c = cameFrom.get(c) || "";
        }
        path.push(startId);
        return path.reverse();
      }

      visited.add(current);

      // Get neighbors
      let neighbors: string[] = [];

      // From extra edges
      if (extraEdges.has(current)) {
        const edges = extraEdges.get(current);
        if (edges) {
          for (let i = 0; i < edges.length; i++) {
            neighbors.push(edges[i]);
          }
        }
      }

      // From graph (if current is numeric)
      const currentNum = parseInt(current);
      if (!isNaN(currentNum)) {
        const graphNeighbors = this.graph.getNeighbors(currentNum);
        for (let i = 0; i < graphNeighbors.length; i++) {
          neighbors.push(graphNeighbors[i].toString());
        }
      }

      for (let i = 0; i < neighbors.length; i++) {
        const neighbor = neighbors[i];
        if (visited.has(neighbor)) continue;

        const tentativeG = (gScore.get(current) || 0) + 
          this.heuristic(coordOf(current), coordOf(neighbor));

        if (tentativeG < (gScore.get(neighbor) || Infinity)) {
          cameFrom.set(neighbor, current);
          gScore.set(neighbor, tentativeG);
          fScore.set(neighbor, tentativeG + this.heuristic(coordOf(neighbor), coordOf(endId)));
          
          if (!openSet.includes(neighbor)) {
            openSet.push(neighbor);
          }
        }
      }
    }

    return null;
  }

  // ========== ROUTE PLANNING ==========

  findRoute(
    startCoord: Coordinate,
    endCoord: Coordinate
  ): Coordinate[] | null {
    if (!this.wpReady || this.waypoints.length === 0) {
      // Direct line if visible
      if (this.canSee(startCoord, endCoord, 45)) {
        return [startCoord, endCoord];
      }
      return null;
    }

    if (this.isBlocked(startCoord)) return null;
    if (this.isBlocked(endCoord)) return null;

    const startCands = this.findNearestWaypoints(startCoord, 14);
    const endCands = this.findNearestWaypoints(endCoord, 14);
    const startWPs = this.pickReachable(startCoord, startCands);
    const endWPs = this.pickReachable(endCoord, endCands);

    if (startWPs.length === 0 || endWPs.length === 0) {
      // Try direct line
      if (this.canSee(startCoord, endCoord, 45)) {
        return [startCoord, endCoord];
      }
      return null;
    }

    // Build extra edges
    const extraEdges: Map<string, string[]> = new Map();
    extraEdges.set("S", []);
    extraEdges.set("E", []);

    for (let i = 0; i < startWPs.length; i++) {
      const idx = startWPs[i].index.toString();
      extraEdges.get("S").push(idx);
      if (!extraEdges.has(idx)) extraEdges.set(idx, []);
      extraEdges.get(idx).push("S");
    }

    for (let i = 0; i < endWPs.length; i++) {
      const idx = endWPs[i].index.toString();
      extraEdges.get("E").push(idx);
      if (!extraEdges.has(idx)) extraEdges.set(idx, []);
      extraEdges.get(idx).push("E");
    }

    // Check if start and end share waypoint
    for (let i = 0; i < startWPs.length; i++) {
      for (let j = 0; j < endWPs.length; j++) {
        if (startWPs[i].index === endWPs[j].index) {
          extraEdges.get("S").push("E");
          extraEdges.get("E").push("S");
        }
      }
    }

    const coordOf = (id: string): Coordinate => {
      if (id === "S") return startCoord;
      if (id === "E") return endCoord;
      const idx = parseInt(id);
      return new Coordinate(this.waypoints[idx].lat, this.waypoints[idx].lng);
    };

    const pathIds = this.astar("S", "E", extraEdges, coordOf);

    if (!pathIds || pathIds.length < 2) {
      if (this.canSee(startCoord, endCoord, 45)) {
        return [startCoord, endCoord];
      }
      return null;
    }

    const route: Coordinate[] = [];
    for (let i = 0; i < pathIds.length; i++) {
      route.push(coordOf(pathIds[i]));
    }

    return route;
  }

  // ========== ROUTE ANALYSIS ==========

  analyzeRoute(route: Coordinate[]): RouteAnalysis {
    let totalDistance = 0.0;
    let minDepth = Infinity;
    let maxDepth = -Infinity;
    let safePoints = 0;
    let unsafePoints = 0;

    for (let i = 1; i < route.length; i++) {
      const dist = route[i - 1].distanceKm(route[i]);
      totalDistance += dist;
    }

    // Simulate depth analysis
    for (let i = 0; i < route.length; i++) {
      const depth = 15.0 + 5.0 * Math.sin(route[i].lat * 0.5) + 3.0 * Math.cos(route[i].lng * 0.3);
      const safe = depth > 10.0;
      
      if (depth < minDepth) minDepth = depth;
      if (depth > maxDepth) maxDepth = depth;
      if (safe) safePoints++;
      else unsafePoints++;
    }

    return new RouteAnalysis(
      totalDistance,
      totalDistance / 1.852,
      minDepth,
      maxDepth,
      safePoints,
      unsafePoints,
      route.length
    );
  }

  // ========== UTILITY ==========

  isReady(): bool {
    return this.wpReady && this.islandsReady;
  }

  getWaypointCount(): i32 {
    return this.waypoints.length as i32;
  }

  getIslandCount(): i32 {
    return this.islands.length as i32;
  }

  clear(): void {
    this.islands = [];
    this.waypoints = [];
    this.graph = new WaypointGraph();
    this.wpReady = false;
    this.islandsReady = false;
  }
}

// ============================================
// ROUTE ANALYSIS RESULT
// ============================================

export class RouteAnalysis {
  totalDistanceKm: f64;
  totalDistanceNm: f64;
  minDepth: f64;
  maxDepth: f64;
  safePoints: i32;
  unsafePoints: i32;
  totalPoints: i32;

  constructor(
    totalDistanceKm: f64,
    totalDistanceNm: f64,
    minDepth: f64,
    maxDepth: f64,
    safePoints: i32,
    unsafePoints: i32,
    totalPoints: i32
  ) {
    this.totalDistanceKm = totalDistanceKm;
    this.totalDistanceNm = totalDistanceNm;
    this.minDepth = minDepth;
    this.maxDepth = maxDepth;
    this.safePoints = safePoints;
    this.unsafePoints = unsafePoints;
    this.totalPoints = totalPoints;
  }

  getSafetyPercentage(): f64 {
    if (this.totalPoints === 0) return 0.0;
    return (this.safePoints as f64 / this.totalPoints as f64) * 100.0;
  }

  isSafe(): bool {
    return this.unsafePoints === 0;
  }

  getStatus(): string {
    if (this.unsafePoints === 0) return "SAFE";
    if (this.unsafePoints < this.totalPoints / 2) return "CAUTION";
    return "UNSAFE";
  }
}

// ============================================
// EXPORTED FUNCTIONS FOR JAVASCRIPT
// ============================================

// Global engine instance
let engine: NavigationEngine | null = null;

export function createEngine(): void {
  engine = new NavigationEngine();
}

export function destroyEngine(): void {
  engine = null;
}

// Island functions
export function addIslandFromArray(
  name: string,
  latArray: Float64Array,
  lngArray: Float64Array
): void {
  if (!engine) return;
  
  const polygon: Coordinate[] = [];
  const len = latArray.length < lngArray.length ? latArray.length : lngArray.length;
  
  for (let i = 0; i < len; i++) {
    polygon.push(new Coordinate(latArray[i], lngArray[i]));
  }
  
  const island = new Island(name, polygon);
  engine.addIsland(island);
}

export function addIslandsFromJSON(json: string): void {
  if (!engine) return;
  // Parse JSON and add islands
  // This would be handled by JavaScript side
}

// Waypoint functions
export function addWaypoint(lat: f64, lng: f64, name: string): void {
  if (!engine) return;
  engine.addWaypoint(new Waypoint(lat, lng, name));
}

export function addWaypointsFromArray(
  latArray: Float64Array,
  lngArray: Float64Array,
  nameArray: string[] | null
): void {
  if (!engine) return;
  
  const len = latArray.length < lngArray.length ? latArray.length : lngArray.length;
  
  for (let i = 0; i < len; i++) {
    const name = nameArray && i < nameArray.length ? nameArray[i] : `WP-${i}`;
    engine.addWaypoint(new Waypoint(latArray[i], lngArray[i], name));
  }
}

export function buildGraph(): bool {
  if (!engine) return false;
  engine.buildGraph();
  return engine.isReady();
}

// Navigation functions
export function findRoute(
  startLat: f64,
  startLng: f64,
  endLat: f64,
  endLng: f64
): Float64Array | null {
  if (!engine) return null;
  
  const start = new Coordinate(startLat, startLng);
  const end = new Coordinate(endLat, endLng);
  
  const route = engine.findRoute(start, end);
  if (!route) return null;
  
  const result = new Float64Array(route.length * 2);
  for (let i = 0; i < route.length; i++) {
    result[i * 2] = route[i].lat;
    result[i * 2 + 1] = route[i].lng;
  }
  
  return result;
}

export function analyzeRoute(routeArray: Float64Array): RouteAnalysis | null {
  if (!engine) return null;
  
  const route: Coordinate[] = [];
  for (let i = 0; i < routeArray.length; i += 2) {
    route.push(new Coordinate(routeArray[i], routeArray[i + 1]));
  }
  
  return engine.analyzeRoute(route);
}

// Utility functions
export function isBlocked(lat: f64, lng: f64): bool {
  if (!engine) return false;
  return engine.isBlocked(new Coordinate(lat, lng));
}

export function getStatus(): string {
  if (!engine) return "NOT_INITIALIZED";
  if (engine.isReady()) return "READY";
  return "LOADING";
}

export function getWaypointCount(): i32 {
  if (!engine) return 0;
  return engine.getWaypointCount();
}

export function getIslandCount(): i32 {
  if (!engine) return 0;
  return engine.getIslandCount();
}

export function clearAll(): void {
  if (!engine) return;
  engine.clear();
}

// Distance calculation
export function distanceKm(lat1: f64, lng1: f64, lat2: f64, lng2: f64): f64 {
  const a = new Coordinate(lat1, lng1);
  const b = new Coordinate(lat2, lng2);
  return a.distanceKm(b);
}

// ============================================
// MEMORY MANAGEMENT
// ============================================

export function memory(): WebAssembly.Memory {
  return new WebAssembly.Memory({ initial: 256, maximum: 1024 });
}
