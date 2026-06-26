# ============================================
# MARINE ROUTER INDONESIA - JULIA VERSION
# Complete Navigation System with Julia
# ============================================

module MarineRouter

using LinearAlgebra
using Statistics
using Dates
using JSON
using HTTP
using Plots
using PlotlyJS
using DataFrames
using GeometryBasics
using PolygonOps
using NearestNeighbors
using Graphs
using GraphPlot

# ============================================
# CORE GEOSPATIAL TYPES
# ============================================

export Coordinate, BoundingBox, Island, Waypoint, WaypointGraph
export NavigationEngine, RouteAnalysis, MarineRouterApp

"""
    Coordinate(lat, lng)

Represents a geographic coordinate with latitude and longitude in degrees.
"""
struct Coordinate
    lat::Float64
    lng::Float64
    
    function Coordinate(lat::Float64, lng::Float64)
        @assert -90 <= lat <= 90 "Latitude must be between -90 and 90"
        @assert -180 <= lng <= 180 "Longitude must be between -180 and 180"
        new(lat, lng)
    end
end

# Constructor with keyword arguments
Coordinate(; lat::Float64, lng::Float64) = Coordinate(lat, lng)

# Conversion from array
Coordinate(arr::Vector{Float64}) = Coordinate(arr[1], arr[2])

Base.:(==)(a::Coordinate, b::Coordinate) = a.lat == b.lat && a.lng == b.lng
Base.hash(c::Coordinate, h::UInt) = hash((c.lat, c.lng), h)

"""
    distance(a::Coordinate, b::Coordinate)

Calculate Euclidean distance between two coordinates in degrees.
"""
function distance(a::Coordinate, b::Coordinate)
    dlat = a.lat - b.lat
    dlng = a.lng - b.lng
    return sqrt(dlat^2 + dlng^2)
end

"""
    distance_km(a::Coordinate, b::Coordinate)

Calculate great-circle distance between two coordinates in kilometers.
"""
function distance_km(a::Coordinate, b::Coordinate)
    R = 6371.0  # Earth's radius in km
    dlat = deg2rad(b.lat - a.lat)
    dlng = deg2rad(b.lng - a.lng)
    lat1 = deg2rad(a.lat)
    lat2 = deg2rad(b.lat)
    
    x = sin(dlat/2)^2 + cos(lat1) * cos(lat2) * sin(dlng/2)^2
    c = 2 * atan(sqrt(x), sqrt(1 - x))
    return R * c
end

"""
    to_array(c::Coordinate)

Convert coordinate to array [lat, lng].
"""
to_array(c::Coordinate) = [c.lat, c.lng]

"""
    BoundingBox(min_lat, max_lat, min_lng, max_lng)

Represents a rectangular bounding box.
"""
struct BoundingBox
    min_lat::Float64
    max_lat::Float64
    min_lng::Float64
    max_lng::Float64
end

function BoundingBox(coords::Vector{Coordinate})
    min_lat = minimum([c.lat for c in coords])
    max_lat = maximum([c.lat for c in coords])
    min_lng = minimum([c.lng for c in coords])
    max_lng = maximum([c.lng for c in coords])
    return BoundingBox(min_lat, max_lat, min_lng, max_lng)
end

contains(bbox::BoundingBox, coord::Coordinate) = 
    bbox.min_lat <= coord.lat <= bbox.max_lat &&
    bbox.min_lng <= coord.lng <= bbox.max_lng

"""
    Island(name, polygon)

Represents a landmass or obstacle with a polygon boundary.
"""
struct Island
    name::String
    polygon::Vector{Coordinate}
    bbox::BoundingBox
    
    function Island(name::String, polygon::Vector{Coordinate})
        bbox = BoundingBox(polygon)
        return new(name, polygon, bbox)
    end
end

function Island(name::String, polygon::Vector{Vector{Float64}})
    coords = [Coordinate(p) for p in polygon]
    return Island(name, coords)
end

"""
    point_in_polygon(point, polygon)

Check if a point is inside a polygon using ray casting algorithm.
"""
function point_in_polygon(point::Coordinate, polygon::Vector{Coordinate})
    inside = false
    n = length(polygon)
    
    for i in 1:n
        j = i == 1 ? n : i - 1
        xi, yi = polygon[i].lat, polygon[i].lng
        xj, yj = polygon[j].lat, polygon[j].lng
        
        if ((yi > point.lat) != (yj > point.lat)) &&
           (point.lng < (xj - xi) * (point.lat - yi) / (yj - yi) + xi)
            inside = !inside
        end
    end
    
    return inside
end

"""
    Waypoint(lat, lng, name)

Represents a navigational waypoint.
"""
struct Waypoint
    lat::Float64
    lng::Float64
    name::String
    
    function Waypoint(lat::Float64, lng::Float64, name::String="")
        @assert -90 <= lat <= 90 "Latitude must be between -90 and 90"
        @assert -180 <= lng <= 180 "Longitude must be between -180 and 180"
        new(lat, lng, name)
    end
end

Waypoint(coord::Coordinate, name::String="") = Waypoint(coord.lat, coord.lng, name)
to_coordinate(wp::Waypoint) = Coordinate(wp.lat, wp.lng)

"""
    WaypointGraph()

Graph structure for waypoint connectivity.
"""
struct WaypointGraph
    edges::Dict{Int, Vector{Int}}
    
    function WaypointGraph()
        return new(Dict{Int, Vector{Int}}())
    end
end

function add_edge!(graph::WaypointGraph, from::Int, to::Int)
    if !haskey(graph.edges, from)
        graph.edges[from] = Int[]
    end
    if !(to in graph.edges[from])
        push!(graph.edges[from], to)
    end
end

function get_neighbors(graph::WaypointGraph, node::Int)
    return get(graph.edges, node, Int[])
end

# ============================================
# NAVIGATION ENGINE
# ============================================

"""
    NavigationEngine()

Main navigation engine with route finding capabilities.
"""
mutable struct NavigationEngine
    islands::Vector{Island}
    waypoints::Vector{Waypoint}
    graph::WaypointGraph
    wp_ready::Bool
    islands_ready::Bool
    
    # Constants
    WP_CONNECT_RADIUS::Float64
    WP_K_NEAR::Int
    MAX_ITER::Int
    
    function NavigationEngine()
        return new(
            Island[],
            Waypoint[],
            WaypointGraph(),
            false,
            false,
            3.0,   # WP_CONNECT_RADIUS
            9,     # WP_K_NEAR
            220000 # MAX_ITER
        )
    end
end

# ========== ISLAND METHODS ==========

"""
    add_island!(engine, island)

Add an island to the navigation engine.
"""
function add_island!(engine::NavigationEngine, island::Island)
    push!(engine.islands, island)
    engine.islands_ready = true
    return nothing
end

"""
    add_islands!(engine, islands)

Add multiple islands to the navigation engine.
"""
function add_islands!(engine::NavigationEngine, islands::Vector{Island})
    append!(engine.islands, islands)
    engine.islands_ready = true
    return nothing
end

"""
    is_blocked(engine, coord)

Check if a coordinate is blocked by any island.
"""
function is_blocked(engine::NavigationEngine, coord::Coordinate)
    for island in engine.islands
        # BBox quick check
        if !contains(island.bbox, coord)
            continue
        end
        if point_in_polygon(coord, island.polygon)
            return true
        end
    end
    return false
end

"""
    can_see(engine, a, b, samples)

Check if there is line of sight between two points.
"""
function can_see(engine::NavigationEngine, a::Coordinate, b::Coordinate, samples::Int=26)
    for i in 1:samples-1
        t = i / samples
        lat = a.lat + (b.lat - a.lat) * t
        lng = a.lng + (b.lng - a.lng) * t
        if is_blocked(engine, Coordinate(lat, lng))
            return false
        end
    end
    return true
end

# ========== WAYPOINT METHODS ==========

"""
    add_waypoint!(engine, waypoint)

Add a waypoint to the navigation engine.
"""
function add_waypoint!(engine::NavigationEngine, waypoint::Waypoint)
    push!(engine.waypoints, waypoint)
    return nothing
end

"""
    add_waypoints!(engine, waypoints)

Add multiple waypoints to the navigation engine.
"""
function add_waypoints!(engine::NavigationEngine, waypoints::Vector{Waypoint})
    append!(engine.waypoints, waypoints)
    return nothing
end

"""
    build_graph!(engine)

Build the waypoint connectivity graph.
"""
function build_graph!(engine::NavigationEngine)
    if isempty(engine.waypoints)
        return false
    end
    
    engine.graph = WaypointGraph()
    
    for i in 1:length(engine.waypoints)
        dists = Vector{Tuple{Int, Float64}}()
        a = to_coordinate(engine.waypoints[i])
        
        for j in 1:length(engine.waypoints)
            i == j && continue
            b = to_coordinate(engine.waypoints[j])
            d = distance(a, b)
            if d <= engine.WP_CONNECT_RADIUS
                push!(dists, (j, d))
            end
        end
        
        # Sort by distance
        sort!(dists, by = x -> x[2])
        
        # Take top K
        for k in 1:min(length(dists), engine.WP_K_NEAR)
            j = dists[k][1]
            a_coord = to_coordinate(engine.waypoints[i])
            b_coord = to_coordinate(engine.waypoints[j])
            
            if isempty(engine.islands) || can_see(engine, a_coord, b_coord, 18)
                add_edge!(engine.graph, i, j)
                add_edge!(engine.graph, j, i)
            end
        end
    end
    
    engine.wp_ready = true
    return true
end

"""
    find_nearest_waypoints(engine, coord, k)

Find the nearest waypoints to a coordinate.
"""
function find_nearest_waypoints(engine::NavigationEngine, coord::Coordinate, k::Int=10)
    dists = Vector{Tuple{Int, Float64}}()
    
    for (i, wp) in enumerate(engine.waypoints)
        wp_coord = to_coordinate(wp)
        d = distance(coord, wp_coord)
        push!(dists, (i, d))
    end
    
    sort!(dists, by = x -> x[2])
    return dists[1:min(length(dists), k)]
end

"""
    pick_reachable(engine, coord, candidates)

Filter candidates that are reachable from a coordinate.
"""
function pick_reachable(engine::NavigationEngine, coord::Coordinate, candidates::Vector{Tuple{Int, Float64}})
    reachable = Tuple{Int, Float64}[]
    
    for (i, _) in candidates
        wp_coord = to_coordinate(engine.waypoints[i])
        if can_see(engine, coord, wp_coord, 24)
            push!(reachable, (i, distance(coord, wp_coord)))
        end
    end
    
    if !isempty(reachable)
        return reachable[1:min(length(reachable), 4)]
    else
        return candidates[1:min(length(candidates), 2)]
    end
end

# ========== A* PATHFINDING ==========

function coord_of(engine::NavigationEngine, id::String)
    if id == "S"
        return engine.start_coord
    elseif id == "E"
        return engine.end_coord
    else
        idx = parse(Int, id)
        return to_coordinate(engine.waypoints[idx])
    end
end

function astar(engine::NavigationEngine, start_id::String, end_id::String, extra_edges::Dict{String, Vector{String}})
    engine.start_coord = coordinate_for_id(engine, start_id)
    engine.end_coord = coordinate_for_id(engine, end_id)
    
    open_set = [start_id]
    came_from = Dict{String, String}()
    g_score = Dict{String, Float64}(start_id => 0.0)
    f_score = Dict{String, Float64}(start_id => distance_km(
        coordinate_for_id(engine, start_id),
        coordinate_for_id(engine, end_id)
    ))
    visited = Set{String}()
    
    iterations = 0
    
    while !isempty(open_set) && iterations < engine.MAX_ITER
        iterations += 1
        
        # Find node with lowest f_score
        current = open_set[1]
        current_idx = 1
        for i in 2:length(open_set)
            f_current = get(f_score, open_set[i], Inf)
            f_best = get(f_score, current, Inf)
            if f_current < f_best
                current = open_set[i]
                current_idx = i
            end
        end
        
        deleteat!(open_set, current_idx)
        
        if current == end_id
            # Reconstruct path
            path = String[]
            c = current
            while haskey(came_from, c)
                push!(path, c)
                c = came_from[c]
            end
            push!(path, start_id)
            return reverse(path)
        end
        
        push!(visited, current)
        
        # Get neighbors
        neighbors = String[]
        
        # From extra edges
        if haskey(extra_edges, current)
            append!(neighbors, extra_edges[current])
        end
        
        # From graph (if current is numeric)
        current_num = parse(Int, current)
        if !isnan(current_num)
            graph_neighbors = get_neighbors(engine.graph, current_num)
            for n in graph_neighbors
                push!(neighbors, string(n))
            end
        end
        
        for neighbor in neighbors
            neighbor in visited && continue
            
            tentative_g = get(g_score, current, 0.0) + 
                distance_km(
                    coordinate_for_id(engine, current),
                    coordinate_for_id(engine, neighbor)
                )
            
            if tentative_g < get(g_score, neighbor, Inf)
                came_from[neighbor] = current
                g_score[neighbor] = tentative_g
                f_score[neighbor] = tentative_g + distance_km(
                    coordinate_for_id(engine, neighbor),
                    coordinate_for_id(engine, end_id)
                )
                
                if !(neighbor in open_set)
                    push!(open_set, neighbor)
                end
            end
        end
    end
    
    return nothing
end

function coordinate_for_id(engine::NavigationEngine, id::String)
    if id == "S"
        return engine.start_coord
    elseif id == "E"
        return engine.end_coord
    else
        idx = parse(Int, id)
        return to_coordinate(engine.waypoints[idx])
    end
end

# ========== ROUTE PLANNING ==========

"""
    find_route(engine, start, end)

Find the optimal route between two coordinates.
"""
function find_route(engine::NavigationEngine, start::Coordinate, end::Coordinate)
    if is_blocked(engine, start)
        return nothing
    end
    if is_blocked(engine, end)
        return nothing
    end
    
    # Direct line if no waypoints or can see
    if !engine.wp_ready || isempty(engine.waypoints)
        if can_see(engine, start, end, 45)
            return [start, end]
        end
        return nothing
    end
    
    start_cands = find_nearest_waypoints(engine, start, 14)
    end_cands = find_nearest_waypoints(engine, end, 14)
    start_wps = pick_reachable(engine, start, start_cands)
    end_wps = pick_reachable(engine, end, end_cands)
    
    if isempty(start_wps) || isempty(end_wps)
        if can_see(engine, start, end, 45)
            return [start, end]
        end
        return nothing
    end
    
    # Build extra edges
    extra_edges = Dict{String, Vector{String}}(
        "S" => String[],
        "E" => String[]
    )
    
    for (idx, _) in start_wps
        id = string(idx)
        push!(extra_edges["S"], id)
        if !haskey(extra_edges, id)
            extra_edges[id] = String[]
        end
        push!(extra_edges[id], "S")
    end
    
    for (idx, _) in end_wps
        id = string(idx)
        push!(extra_edges["E"], id)
        if !haskey(extra_edges, id)
            extra_edges[id] = String[]
        end
        push!(extra_edges[id], "E")
    end
    
    # Check if start and end share waypoint
    for (s_idx, _) in start_wps
        for (e_idx, _) in end_wps
            if s_idx == e_idx
                push!(extra_edges["S"], "E")
                push!(extra_edges["E"], "S")
            end
        end
    end
    
    engine.start_coord = start
    engine.end_coord = end
    
    path_ids = astar(engine, "S", "E", extra_edges)
    
    if path_ids === nothing
        if can_see(engine, start, end, 45)
            return [start, end]
        end
        return nothing
    end
    
    # Build route from path
    route = Coordinate[]
    for id in path_ids
        push!(route, coordinate_for_id(engine, id))
    end
    
    return route
end

# ============================================
# ROUTE ANALYSIS
# ============================================

"""
    RouteAnalysis

Analysis results for a route.
"""
struct RouteAnalysis
    total_distance_km::Float64
    total_distance_nm::Float64
    min_depth::Float64
    max_depth::Float64
    safe_points::Int
    unsafe_points::Int
    total_points::Int
end

function RouteAnalysis(route::Vector{Coordinate})
    total_distance = 0.0
    min_depth = Inf
    max_depth = -Inf
    safe_points = 0
    unsafe_points = 0
    
    for i in 2:length(route)
        total_distance += distance_km(route[i-1], route[i])
    end
    
    for coord in route
        # Simulate depth data
        depth = 15.0 + 5.0 * sin(coord.lat * 0.5) + 3.0 * cos(coord.lng * 0.3)
        is_safe = depth > 10.0
        
        min_depth = min(min_depth, depth)
        max_depth = max(max_depth, depth)
        if is_safe
            safe_points += 1
        else
            unsafe_points += 1
        end
    end
    
    return RouteAnalysis(
        total_distance,
        total_distance / 1.852,
        min_depth,
        max_depth,
        safe_points,
        unsafe_points,
        length(route)
    )
end

function safety_percentage(analysis::RouteAnalysis)
    if analysis.total_points == 0
        return 0.0
    end
    return analysis.safe_points / analysis.total_points * 100.0
end

function is_safe(analysis::RouteAnalysis)
    return analysis.unsafe_points == 0
end

function status(analysis::RouteAnalysis)
    if analysis.unsafe_points == 0
        return "SAFE"
    elseif analysis.unsafe_points < analysis.total_points / 2
        return "CAUTION"
    else
        return "UNSAFE"
    end
end

# ============================================
# VISUALIZATION
# ============================================

"""
    plot_route(engine, route; kwargs...)

Plot the route with islands and waypoints.
"""
function plot_route(engine::NavigationEngine, route::Vector{Coordinate}; 
                    title="Marine Route", figsize=(1200, 800))
    
    # Collect coordinates
    route_lats = [c.lat for c in route]
    route_lngs = [c.lng for c in route]
    
    # Waypoints
    wp_lats = [wp.lat for wp in engine.waypoints]
    wp_lngs = [wp.lng for wp in engine.waypoints]
    
    # Create plot
    p = PlotlyJS.plot(
        PlotlyJS.scatter(
            x=route_lngs,
            y=route_lats,
            mode="lines+markers",
            name="Route",
            line=Dict("color" => "blue", "width" => 3),
            marker=Dict("size" => 8)
        ),
        PlotlyJS.scatter(
            x=wp_lngs,
            y=wp_lats,
            mode="markers",
            name="Waypoints",
            marker=Dict("size" => 6, "color" => "green", "symbol" => "star")
        ),
        Layout(
            title=title,
            xaxis_title="Longitude",
            yaxis_title="Latitude",
            width=figsize[1],
            height=figsize[2],
            hovermode="closest"
        )
    )
    
    # Add islands
    for island in engine.islands
        island_lats = [c.lat for c in island.polygon]
        island_lngs = [c.lng for c in island.polygon]
        PlotlyJS.add_trace!(
            p,
            PlotlyJS.scatter(
                x=island_lngs,
                y=island_lats,
                mode="lines",
                name=island.name,
                fill="toself",
                fillcolor="rgba(200, 100, 50, 0.3)",
                line=Dict("color" => "red", "width" => 2)
            )
        )
    end
    
    return p
end

"""
    plot_route_analysis(analysis::RouteAnalysis)

Plot route analysis metrics.
"""
function plot_route_analysis(analysis::RouteAnalysis)
    labels = ["Safe", "Unsafe"]
    values = [analysis.safe_points, analysis.unsafe_points]
    colors = ["#27ae60", "#e74c3c"]
    
    p = PlotlyJS.plot(
        PlotlyJS.pie(
            labels=labels,
            values=values,
            marker=Dict("colors" => colors),
            hole=0.3,
            textinfo="label+percent"
        ),
        Layout(
            title="Route Safety Analysis",
            width=500,
            height=500
        )
    )
    
    return p
end

"""
    plot_route_with_matplotlib(engine, route; kwargs...)

Plot route using Plots.jl (Matplotlib backend).
"""
function plot_route_with_matplotlib(engine::NavigationEngine, route::Vector{Coordinate};
                                    title="Marine Route", figsize=(12, 8))
    
    # Convert to arrays for plotting
    route_lats = [c.lat for c in route]
    route_lngs = [c.lng for c in route]
    
    wp_lats = [wp.lat for wp in engine.waypoints]
    wp_lngs = [wp.lng for wp in engine.waypoints]
    
    # Create plot
    p = plot(route_lngs, route_lats, 
             linewidth=3, 
             label="Route",
             title=title,
             xlabel="Longitude",
             ylabel="Latitude",
             legend=:topright,
             figsize=figsize,
             linecolor=:blue,
             marker=:circle,
             markersize=6)
    
    # Add waypoints
    scatter!(p, wp_lngs, wp_lats, 
             label="Waypoints", 
             markersize=8, 
             marker=:star5, 
             markercolor=:green)
    
    # Add islands
    for island in engine.islands
        island_lats = [c.lat for c in island.polygon]
        island_lngs = [c.lng for c in island.polygon]
        plot!(p, island_lngs, island_lats,
              label=island.name,
              linewidth=2,
              linecolor=:red,
              fill=(0, 0.2, :orange))
    end
    
    return p
end

# ============================================
# DATA LOADING
# ============================================

"""
    load_islands_from_json(filename)

Load islands from a JSON file.
"""
function load_islands_from_json(filename::String)
    data = JSON.parsefile(filename)
    islands = Island[]
    
    for item in data["islands"]
        polygon = [[p[1], p[2]] for p in item["polygon"]]
        island = Island(item["name"], polygon)
        push!(islands, island)
    end
    
    return islands
end

"""
    load_waypoints_from_json(filename)

Load waypoints from a JSON file.
"""
function load_waypoints_from_json(filename::String)
    data = JSON.parsefile(filename)
    waypoints = Waypoint[]
    
    for item in data["waypoints"]
        wp = Waypoint(item["lat"], item["lng"], get(item, "name", ""))
        push!(waypoints, wp)
    end
    
    return waypoints
end

"""
    load_from_url(url)

Load data from a URL.
"""
function load_from_url(url::String)
    response = HTTP.get(url)
    return JSON.parse(String(response.body))
end

# ============================================
# MAIN APPLICATION
# ============================================

"""
    MarineRouterApp()

Complete marine routing application.
"""
mutable struct MarineRouterApp
    engine::NavigationEngine
    routes::Vector{Vector{Coordinate}}
    analyses::Vector{RouteAnalysis}
    
    function MarineRouterApp()
        return new(
            NavigationEngine(),
            Vector{Coordinate}[],
            RouteAnalysis[]
        )
    end
end

function setup_app!(app::MarineRouterApp)
    # Sample islands
    islands = [
        Island("Java", [
            [-8.0, 105.0], [-7.5, 106.0], [-7.0, 107.0], 
            [-6.5, 108.0], [-6.0, 109.0], [-6.5, 110.0],
            [-7.0, 111.0], [-7.5, 112.0], [-8.0, 113.0],
            [-8.5, 114.0], [-8.0, 105.0]
        ]),
        Island("Bali", [
            [-8.5, 114.5], [-8.2, 115.0], [-8.0, 115.5],
            [-8.2, 115.8], [-8.5, 115.5], [-8.7, 115.0],
            [-8.5, 114.5]
        ])
    ]
    
    add_islands!(app.engine, islands)
    
    # Sample waypoints
    waypoints = [
        Waypoint(-6.125, 106.655, "Jakarta"),
        Waypoint(-7.189, 112.730, "Surabaya"),
        Waypoint(-8.340, 115.092, "Bali"),
        Waypoint(-5.148, 119.432, "Makassar"),
        Waypoint(-6.0, 108.0, "WP-1"),
        Waypoint(-6.5, 109.0, "WP-2"),
        Waypoint(-7.0, 110.0, "WP-3"),
        Waypoint(-7.5, 111.0, "WP-4"),
        Waypoint(-8.0, 114.0, "WP-5"),
        Waypoint(-7.0, 113.0, "WP-6")
    ]
    
    add_waypoints!(app.engine, waypoints)
    build_graph!(app.engine)
    
    return nothing
end

function find_and_analyze!(app::MarineRouterApp, start::Coordinate, finish::Coordinate)
    route = find_route(app.engine, start, finish)
    
    if route === nothing
        @warn "Route not found!"
        return nothing
    end
    
    analysis = RouteAnalysis(route)
    push!(app.routes, route)
    push!(app.analyses, analysis)
    
    return (route, analysis)
end

# ============================================
# EXPORT FUNCTIONS FOR INTEROPERABILITY
# ============================================

"""
    export_route_to_geojson(route, filename)

Export route to GeoJSON format.
"""
function export_route_to_geojson(route::Vector{Coordinate}, filename::String)
    coords = [[c.lng, c.lat] for c in route]
    
    geojson = Dict(
        "type" => "FeatureCollection",
        "features" => [
            Dict(
                "type" => "Feature",
                "geometry" => Dict(
                    "type" => "LineString",
                    "coordinates" => coords
                ),
                "properties" => Dict(
                    "type" => "Route",
                    "waypoint_count" => length(route)
                )
            )
        ]
    )
    
    open(filename, "w") do f
        JSON.print(f, geojson, 2)
    end
end

"""
    export_route_to_kml(route, filename)

Export route to KML format.
"""
function export_route_to_kml(route::Vector{Coordinate}, filename::String)
    kml = """
    <?xml version="1.0" encoding="UTF-8"?>
    <kml xmlns="http://www.opengis.net/kml/2.2">
    <Document>
        <name>Marine Route</name>
        <Placemark>
            <name>Route</name>
            <LineString>
                <coordinates>
    """
    
    for coord in route
        kml *= "        $(coord.lng),$(coord.lat)\n"
    end
    
    kml *= """
                </coordinates>
            </LineString>
        </Placemark>
    </Document>
    </kml>
    """
    
    write(filename, kml)
end

# ============================================
# EXAMPLES AND TESTS
# ============================================

function run_example()
    println("🚢 Marine Router Indonesia - Julia Version")
    println("=" * "^" * 50)
    
    # Create application
    app = MarineRouterApp()
    setup_app!(app)
    
    println("✅ Islands loaded: $(length(app.engine.islands))")
    println("✅ Waypoints loaded: $(length(app.engine.waypoints))")
    println("✅ Graph built: $(app.engine.wp_ready)")
    
    # Find route from Jakarta to Surabaya
    start = Coordinate(-6.125, 106.655)
    finish = Coordinate(-7.189, 112.730)
    
    println("\n🗺️ Finding route from Jakarta to Surabaya...")
    
    result = find_and_analyze!(app, start, finish)
    
    if result !== nothing
        route, analysis = result
        println("✅ Route found!")
        println("   Waypoints: $(length(route))")
        println("   Distance: $(round(analysis.total_distance_km, digits=2)) km")
        println("   Distance: $(round(analysis.total_distance_nm, digits=2)) nm")
        println("   Status: $(status(analysis))")
        println("   Safety: $(round(safety_percentage(analysis), digits=2))%")
        println("   Min Depth: $(round(analysis.min_depth, digits=2)) m")
        println("   Max Depth: $(round(analysis.max_depth, digits=2)) m")
        
        # Export route
        export_route_to_geojson(route, "route.geojson")
        export_route_to_kml(route, "route.kml")
        println("📁 Route exported to route.geojson and route.kml")
        
        # Create visualization
        p = plot_route(app.engine, route)
        PlotlyJS.savehtml(p, "route_plot.html")
        println("📊 Interactive plot saved to route_plot.html")
        
        # Return plot for display
        return p
    else
        println("❌ Route not found!")
        return nothing
    end
end

# ============================================
# WEB SERVER (for API)
# ============================================

using Sockets, HTTP

function start_server(port::Int=8080)
    app = MarineRouterApp()
    setup_app!(app)
    
    println("🚀 Starting Marine Router API on port $port")
    
    @async HTTP.listen("0.0.0.0", port) do http
        path = HTTP.URIs.URI(http.message.target).path
        
        if path == "/"
            response = """
            <!DOCTYPE html>
            <html>
            <head>
                <title>Marine Router API</title>
                <style>
                    body { font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }
                    h1 { color: #1f7e9e; }
                    .endpoint { background: #f0f6fc; padding: 15px; margin: 10px 0; border-radius: 8px; }
                    code { background: #e8ecf0; padding: 2px 6px; border-radius: 4px; }
                </style>
            </head>
            <body>
                <h1>🚢 Marine Router API</h1>
                <p>REST API for marine navigation routing.</p>
                
                <h2>Endpoints:</h2>
                <div class="endpoint">
                    <strong>GET /route</strong><br>
                    Parameters: <code>start_lat</code>, <code>start_lng</code>, <code>end_lat</code>, <code>end_lng</code>
                </div>
                <div class="endpoint">
                    <strong>GET /status</strong><br>
                    Get system status and waypoint count.
                </div>
                <div class="endpoint">
                    <strong>GET /health</strong><br>
                    Health check endpoint.
                </div>
                
                <h2>Example:</h2>
                <code>/route?start_lat=-6.125&start_lng=106.655&end_lat=-7.189&end_lng=112.730</code>
            </body>
            </html>
            """
            return HTTP.Response(200, response)
            
        elseif path == "/health"
            return HTTP.Response(200, JSON.json(Dict("status" => "healthy", "timestamp" => now())))
            
        elseif path == "/status"
            return HTTP.Response(200, JSON.json(Dict(
                "status" => "ready",
                "waypoints" => length(app.engine.waypoints),
                "islands" => length(app.engine.islands),
                "graph_built" => app.engine.wp_ready
            )))
            
        elseif path == "/route"
            params = HTTP.URIs.URI(http.message.target).query_params
            start_lat = parse(Float64, get(params, "start_lat", "0"))
            start_lng = parse(Float64, get(params, "start_lng", "0"))
            end_lat = parse(Float64, get(params, "end_lat", "0"))
            end_lng = parse(Float64, get(params, "end_lng", "0"))
            
            start = Coordinate(start_lat, start_lng)
            finish = Coordinate(end_lat, end_lng)
            
            route = find_route(app.engine, start, finish)
            
            if route === nothing
                return HTTP.Response(404, JSON.json(Dict("error" => "Route not found")))
            end
            
            analysis = RouteAnalysis(route)
            route_json = Dict(
                "route" => [[c.lat, c.lng] for c in route],
                "analysis" => Dict(
                    "distance_km" => round(analysis.total_distance_km, digits=2),
                    "distance_nm" => round(analysis.total_distance_nm, digits=2),
                    "safe_points" => analysis.safe_points,
                    "unsafe_points" => analysis.unsafe_points,
                    "status" => status(analysis),
                    "safety_percentage" => round(safety_percentage(analysis), digits=2)
                )
            )
            
            return HTTP.Response(200, JSON.json(route_json))
        else
            return HTTP.Response(404, "Not Found")
        end
    end
    
    # Keep server running
    while true
        sleep(1)
    end
end

# ============================================
# MAIN ENTRY POINT
# ============================================

function main()
    println("""
  
    """)
    
    # Run example
    run_example()
    
    # Start server
    println("\n🌐 Starting web server...")
    println("   API available at: http://localhost:8080")
    println("   Press Ctrl+C to stop\n")
    
    start_server(8080)
end

# Run if executed directly
if abspath(PROGRAM_FILE) == @__FILE__
    main()
end

end # module MarineRouter
