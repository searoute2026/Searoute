<!DOCTYPE html>
<html lang="id">
<head>
  <title>Marine Router Indonesia - Smart Route Laut + Simulasi Kapal (Satelit)</title>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=yes, viewport-fit=cover" />
  <!-- Leaflet CSS & JS -->
  <link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css" />
  <!-- Font Awesome 6 (free) untuk icon minimalis -->
  <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.0.0-beta3/css/all.min.css" />
  <style>
    * {
      margin: 0;
      padding: 0;
      box-sizing: border-box;
      -webkit-tap-highlight-color: transparent;
    }

    body {
      font-family: 'Inter', system-ui, -apple-system, 'Segoe UI', Roboto, sans-serif;
      background: #eef2f5;
      height: 100vh;
      width: 100vw;
      overflow: hidden;
      position: fixed;
    }

    /* Peta fullscreen */
    #map {
      height: 100%;
      width: 100%;
      z-index: 1;
    }

    /* Sembunyikan tombol zoom Leaflet bawaan (biar minimalis) */
    .leaflet-control-zoom {
      display: none !important;
    }

    /* ========= NAVBAR / PANEL MINIMALIS DENGAN SCROLL ========= */
    .marine-nav {
      position: absolute;
      top: 16px;
      left: 16px;
      right: 16px;
      max-width: 380px;
      max-height: calc(100vh - 32px);
      background: rgba(255, 255, 255, 0.98);
      backdrop-filter: blur(12px);
      -webkit-backdrop-filter: blur(12px);
      border-radius: 28px;
      z-index: 1000;
      box-shadow: 0 12px 28px rgba(0, 20, 40, 0.12);
      border: 1px solid rgba(0, 110, 150, 0.2);
      transition: all 0.25s ease;
      display: flex;
      flex-direction: column;
      overflow: hidden;
    }

    /* header navbar: sangat minimalis, hanya icon + teks & toggle (tetap di atas) */
    .nav-header {
      flex-shrink: 0;
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 12px 18px;
      cursor: pointer;
      user-select: none;
      background: rgba(255, 255, 255, 0.98);
      border-bottom: 1px solid rgba(0, 100, 130, 0.1);
    }

    .brand {
      display: flex;
      align-items: center;
      gap: 10px;
      font-weight: 600;
      font-size: 1rem;
      letter-spacing: -0.2px;
      color: #016e8c;
    }

    .brand i {
      font-size: 1.2rem;
      color: #0277a5;
    }

    .brand span {
      font-weight: 500;
      background: #e6f3fa;
      padding: 2px 10px;
      border-radius: 40px;
      font-size: 0.7rem;
      color: #006d8f;
    }

    .toggle-icon {
      background: transparent;
      border: none;
      color: #227a9e;
      font-size: 1.2rem;
      width: 32px;
      height: 32px;
      border-radius: 30px;
      display: flex;
      align-items: center;
      justify-content: center;
      cursor: pointer;
      transition: 0.2s;
    }

    .toggle-icon:hover {
      background: rgba(0, 100, 148, 0.08);
    }

    /* AREA SCROLL UNTUK KONTEN - SLIDER BAR AKAN MUNCUL OTOMATIS */
    .nav-content {
      flex: 1;
      overflow-y: auto;
      overflow-x: hidden;
      padding: 16px 18px 20px 18px;
      transition: all 0.2s;
      scrollbar-width: thin;
      scrollbar-color: #249bc7 #e0edf5;
    }
    
    .nav-content::-webkit-scrollbar {
      width: 5px;
    }
    
    .nav-content::-webkit-scrollbar-track {
      background: #e0edf5;
      border-radius: 10px;
      margin: 8px 0;
    }
    
    .nav-content::-webkit-scrollbar-thumb {
      background: #249bc7;
      border-radius: 10px;
    }
    
    .nav-content::-webkit-scrollbar-thumb:hover {
      background: #1a7a9e;
    }

    .marine-nav.collapsed .nav-content {
      display: none;
    }

    /* form group minimalis */
    .input-group {
      margin-bottom: 20px;
    }

    .input-label {
      font-size: 0.7rem;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.5px;
      color: #3e7e9e;
      margin-bottom: 8px;
      display: flex;
      align-items: center;
      gap: 6px;
    }

    .input-label i {
      font-size: 0.75rem;
      width: 18px;
    }

    .flex-row {
      display: flex;
      gap: 10px;
      flex-wrap: wrap;
      margin-bottom: 10px;
    }

    .flex-row input {
      flex: 1;
      background: #ffffff;
      border: 1px solid #d4e2ec;
      padding: 10px 14px;
      border-radius: 48px;
      font-size: 0.8rem;
      font-family: monospace;
      color: #144f68;
      transition: 0.2s;
    }

    .flex-row input:focus {
      outline: none;
      border-color: #249bc7;
      box-shadow: 0 0 0 2px rgba(36, 155, 199, 0.1);
    }

    .btn-icon {
      background: #f0f6fc;
      border: 1px solid #cde0ec;
      padding: 8px 14px;
      border-radius: 40px;
      font-size: 0.7rem;
      font-weight: 500;
      cursor: pointer;
      display: inline-flex;
      align-items: center;
      gap: 6px;
      color: #196f8f;
      transition: 0.2s;
    }

    .btn-icon i {
      font-size: 0.75rem;
    }

    .btn-icon:hover {
      background: #e2f0f8;
      border-color: #96c0d4;
    }

    .btn-primary-solid {
      background: #1f7e9e;
      color: white;
      border: none;
      padding: 12px 0;
      width: 100%;
      border-radius: 60px;
      font-weight: 600;
      font-size: 0.85rem;
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 10px;
      cursor: pointer;
      transition: 0.2s;
      margin-bottom: 10px;
    }

    .btn-primary-solid i {
      font-size: 0.9rem;
    }

    .btn-primary-solid:hover {
      background: #0f6785;
    }

    .btn-outline {
      background: #ffffffdd;
      border: 1px solid #cbdbe0;
      width: 100%;
      padding: 10px 0;
      border-radius: 60px;
      font-weight: 500;
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 8px;
      cursor: pointer;
      margin-bottom: 10px;
      color: #2d6d89;
    }

    .btn-outline i {
      font-size: 0.8rem;
    }

    .status-mini {
      background: #eef3f8;
      border-radius: 60px;
      padding: 8px 12px;
      font-size: 0.7rem;
      text-align: center;
      margin: 12px 0 8px;
      color: #296f8c;
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 6px;
    }

    .progress-bar {
      width: 100%;
      height: 3px;
      background: #dbe9f2;
      border-radius: 3px;
      margin: 6px 0 8px;
      display: none;
    }

    .progress-fill {
      height: 100%;
      background: #249bc7;
      width: 0%;
      border-radius: 3px;
    }

    .info-tip {
      font-size: 0.65rem;
      background: #f7fbfe;
      border-radius: 24px;
      padding: 8px 12px;
      margin-top: 12px;
      color: #387e9c;
      border: 1px solid #e2edf5;
    }

    .warning-tip {
      font-size: 0.65rem;
      background: #fff4e6;
      border-radius: 24px;
      padding: 6px 10px;
      text-align: center;
      margin-top: 10px;
      color: #b65c00;
    }

    button {
      font-family: inherit;
    }

    @media (max-width: 460px) {
      .marine-nav {
        max-width: calc(100% - 32px);
        top: 12px;
        left: 12px;
        right: 12px;
        max-height: calc(100vh - 24px);
      }
      .flex-row input {
        font-size: 0.75rem;
        padding: 8px 12px;
      }
    }
  </style>
</head>
<body>

<div id="map"></div>

<!-- NAVBAR / PANEL MINIMALIS -->
<div class="marine-nav collapsed" id="marineNav">
  <div class="nav-header" id="navHeader">
    <div class="brand">
      <i class="fas fa-ship"></i>
      <span>SeaRouter ID</span>
      <i class="fas fa-water" style="font-size: 0.7rem; color:#3792b0;"></i>
    </div>
    <div class="toggle-icon" id="navToggleBtn">
      <i class="fas fa-chevron-up" id="toggleIcon"></i>
    </div>
  </div>
  <div class="nav-content">
    <div class="input-group">
      <div class="input-label"><i class="fas fa-fish"></i> DATA KAPAL (AIS)</div>
      <div class="flex-row">
        <input type="text" id="shipName" placeholder="Nama Kapal" value="MV NUSANTARA">
        <input type="text" id="shipMMSI" placeholder="MMSI" value="525100123">
      </div>
      <div class="flex-row">
        <input type="number" id="shipLength" placeholder="Panjang (m)" value="120" step="1">
        <input type="number" id="shipDraft" placeholder="Draft (m)" value="6.5" step="0.1">
      </div>
    </div>

    <div class="input-group">
      <div class="input-label"><i class="fas fa-map-marker-alt"></i> TITIK AWAL</div>
      <div class="flex-row">
        <input type="text" id="startLat" placeholder="Latitude" value="">
        <input type="text" id="startLng" placeholder="Longitude" value="">
      </div>
      <div style="display: flex; gap: 8px;">
        <button class="btn-icon" id="applyStartBtn"><i class="fas fa-check-circle"></i> Terapkan</button>
        <button class="btn-icon" id="pickStartMapBtn"><i class="fas fa-hand-pointer"></i> Pilih di Peta</button>
      </div>
    </div>

    <div class="input-group">
      <div class="input-label"><i class="fas fa-flag-checkered"></i> TITIK AKHIR</div>
      <div class="flex-row">
        <input type="text" id="endLat" placeholder="Latitude" value="">
        <input type="text" id="endLng" placeholder="Longitude" value="">
      </div>
      <div style="display: flex; gap: 8px;">
        <button class="btn-icon" id="applyEndBtn"><i class="fas fa-check-circle"></i> Terapkan</button>
        <button class="btn-icon" id="pickEndMapBtn"><i class="fas fa-hand-pointer"></i> Pilih di Peta</button>
      </div>
    </div>

    <button class="btn-primary-solid" id="runRouteBtn"><i class="fas fa-route"></i> HITUNG RUTE LAUT</button>
    <button class="btn-outline" id="animateBtn"><i class="fas fa-play"></i> MULAI ANIMASI KAPAL</button>
    <button class="btn-outline" id="clearAllBtn"><i class="fas fa-trash-alt"></i> RESET SEMUA</button>

    <div class="status-mini">
      <i class="fas fa-satellite-dish"></i> <span id="status">Memuat data kelautan...</span>
    </div>
    <div class="progress-bar" id="progressBar"><div class="progress-fill" id="progressFill"></div></div>
    <div class="info-tip" id="infoBox">
      <i class="fas fa-info-circle"></i> Masukkan start & finish, lalu hitung rute.
    </div>
    <div class="warning-tip" id="reminderMsg">
      <i class="fas fa-exclamation-triangle"></i> Isi start & finish sebelum hitung rute
    </div>
  </div>
</div>

<script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"></script>
<script>
  // --------------------------------------------------------------
  // KONFIGURASI MAPTILER SATELLITE
  // --------------------------------------------------------------
  const MAPTILER_API_KEY = 'mP5EMYL63473c6VT5cg6';
  
  // Membuat peta dengan tileset satelit dari MapTiler
  const map = L.map('map', { zoomControl: false }).setView([-2.5, 118], 5);
  
  // Layer Satelit MapTiler (Hybrid Satellite)
  L.tileLayer(`https://api.maptiler.com/maps/hybrid/{z}/{x}/{y}.jpg?key=${MAPTILER_API_KEY}`, {
    attribution: '&copy; <a href="https://www.maptiler.com/copyright/" target="_blank">MapTiler</a> &copy; <a href="https://www.openstreetmap.org/copyright" target="_blank">OpenStreetMap</a> contributors',
    maxZoom: 20,
    tileSize: 512,
    zoomOffset: -1,
    crossOrigin: true
  }).addTo(map);

  // --------------------------------------------------------------
  // GLOBAL STATE & LOGIC (Sama seperti sebelumnya)
  // --------------------------------------------------------------
  const WP_CONNECT_RADIUS = 3.0;
  const WP_K_NEAR         = 9;
  const MAX_ITER          = 220000;
  
  let islands   = [];
  let waypoints = [];
  let wpGraph   = {};
  let wpReady   = false;
  let islandsReady = false;
  
  let start = null;
  let end   = null;
  let mode  = null;
  let routeLine, startMarker, endMarker;
  let isRouting = false;
  let currentRouteCoords = [];
  let animationInterval = null;
  let shipMarker = null;
  let currentAnimIndex = 0;
  
  // helper geometri
  function dist(a, b) { return Math.hypot(a[0] - b[0], a[1] - b[1]); }
  function distKm(a, b) {
    const R = 6371;
    const dLat = (b[0]-a[0]) * Math.PI/180;
    const dLng = (b[1]-a[1]) * Math.PI/180;
    const lat1 = a[0]*Math.PI/180, lat2 = b[0]*Math.PI/180;
    const s = Math.sin(dLat/2)**2 + Math.cos(lat1)*Math.cos(lat2)*Math.sin(dLng/2)**2;
    return R * 2 * Math.atan2(Math.sqrt(s), Math.sqrt(1-s));
  }
  
  function pointInPolygon(p, vs) {
    let inside = false;
    for (let i = 0, j = vs.length-1; i < vs.length; j = i++) {
      const xi=vs[i][0], yi=vs[i][1], xj=vs[j][0], yj=vs[j][1];
      if (((yi > p[1]) !== (yj > p[1])) && (p[0] < (xj-xi)*(p[1]-yi)/(yj-yi)+xi))
        inside = !inside;
    }
    return inside;
  }
  
  function isBlocked(lat, lng) {
    if (!islands.length) return false;
    for (const isl of islands) {
      if (!isl.bbox) continue;
      if (lat < isl.bbox.minLat || lat > isl.bbox.maxLat) continue;
      if (lng < isl.bbox.minLng || lng > isl.bbox.maxLng) continue;
      if (pointInPolygon([lat, lng], isl.polygon)) return true;
    }
    return false;
  }
  
  function canSee(a, b, samples = 26) {
    for (let i = 1; i < samples; i++) {
      const t = i / samples;
      const lat = a[0] + (b[0]-a[0])*t;
      const lng = a[1] + (b[1]-a[1])*t;
      if (isBlocked(lat, lng)) return false;
    }
    return true;
  }
  
  function setStatus(txt, color="#2c7a4d") {
    const el = document.getElementById('status');
    el.innerText = txt;
    el.style.color = color;
  }
  function setInfo(txt) { document.getElementById('infoBox').innerHTML = `<i class="fas fa-info-circle"></i> ${txt}`; }
  function showProgress(p) {
    const pb = document.getElementById('progressBar');
    pb.style.display = 'block';
    document.getElementById('progressFill').style.width = Math.round(p*100)+'%';
  }
  function hideProgress() { document.getElementById('progressBar').style.display = 'none'; }
  const tick = () => new Promise(r => setTimeout(r, 12));
  
  // LOAD islands.json & waypoint.json
  fetch('islands.json')
    .then(r => r.json())
    .then(data => {
      islands = data.islands || [];
      for (const isl of islands) {
        if (!isl.polygon || isl.polygon.length < 3) { isl.skip = true; continue; }
        if (isl.polygon[0] !== isl.polygon[isl.polygon.length-1])
          isl.polygon.push(isl.polygon[0]);
        const lats = isl.polygon.map(p=>p[0]);
        const lngs = isl.polygon.map(p=>p[1]);
        isl.bbox = {
          minLat: Math.min(...lats)-0.02, maxLat: Math.max(...lats)+0.02,
          minLng: Math.min(...lngs)-0.02, maxLng: Math.max(...lngs)+0.02
        };
        L.polygon(isl.polygon, { color:'#e67e22', weight:1.2, fillOpacity:0.15, fillColor:'#c0392b', interactive:false }).addTo(map);
      }
      islandsReady = true;
      tryBuildGraph();
      setStatus("✅ Pulau & batasan siap", "#2c7a4d");
    })
    .catch(() => {
      islandsReady = true;
      tryBuildGraph();
      setStatus("⚠️ islands.json tidak ada → asumsi laut bebas", "#f39c12");
    });
  
  fetch('waypoint.json')
    .then(r => r.json())
    .then(data => {
      waypoints = (data.waypoints || data || []).filter(w => w.lat && w.lng);
      setStatus(`📍 ${waypoints.length} waypoint laut dimuat`, "#2f7f5f");
      tryBuildGraph();
    })
    .catch(() => setStatus("⚠️ waypoint.json gagal", "#e67e22"));
  
  function tryBuildGraph() {
    if (!islandsReady || waypoints.length === 0 || (wpGraph && Object.keys(wpGraph).length > 0)) return;
    buildWaypointGraph();
  }
  
  function buildWaypointGraph() {
    setStatus("🔗 Membangun graf waypoint...", "#f39c12");
    wpGraph = {};
    for (let i = 0; i < waypoints.length; i++) {
      wpGraph[i] = [];
      const dists = [];
      for (let j = 0; j < waypoints.length; j++) {
        if (i === j) continue;
        const d = dist([waypoints[i].lat, waypoints[i].lng], [waypoints[j].lat, waypoints[j].lng]);
        if (d <= WP_CONNECT_RADIUS) dists.push({ j, d });
      }
      dists.sort((a,b)=>a.d-b.d);
      for (const { j } of dists.slice(0, WP_K_NEAR)) {
        const a = [waypoints[i].lat, waypoints[i].lng];
        const b = [waypoints[j].lat, waypoints[j].lng];
        if (!islands.length || canSee(a, b, 18)) {
          if (!wpGraph[i].includes(j)) wpGraph[i].push(j);
          if (!wpGraph[j]) wpGraph[j] = [];
          if (!wpGraph[j].includes(i)) wpGraph[j].push(i);
        }
      }
    }
    wpReady = true;
    setStatus(`✅ Graf siap · ${waypoints.length} titik`, "#2e7d64");
  }
  
  function findNearestWaypoints(coord, k = 10) {
    return waypoints.map((wp, i) => ({ i, d: dist(coord, [wp.lat, wp.lng]) })).sort((a,b)=>a.d-b.d).slice(0,k);
  }
  function pickReachable(coord, candidates) {
    const reachable = candidates.filter(({ i }) => canSee(coord, [waypoints[i].lat, waypoints[i].lng], 24));
    if (reachable.length >= 1) return reachable.slice(0, 4);
    return candidates.slice(0, 2);
  }
  
  class MinHeap {
    constructor() { this.h = []; }
    push(item) { this.h.push(item); this._up(this.h.length-1); }
    pop() { const top = this.h[0], last = this.h.pop(); if (this.h.length) { this.h[0] = last; this._down(0); } return top; }
    _up(i) { while(i>0) { const p=(i-1)>>1; if(this.h[p][0]<=this.h[i][0]) break; [this.h[p],this.h[i]]=[this.h[i],this.h[p]]; i=p; } }
    _down(i) { const n=this.h.length; while(true) { let s=i, l=2*i+1, r=2*i+2; if(l<n && this.h[l][0]<this.h[s][0]) s=l; if(r<n && this.h[r][0]<this.h[s][0]) s=r; if(s===i) break; [this.h[i],this.h[s]]=[this.h[s],this.h[i]]; i=s; } }
    get size() { return this.h.length; }
  }
  
  function astarWithVirtual(srcId, dstId, extraEdges, coordOf) {
    const heap = new MinHeap();
    heap.push([0, srcId]);
    const g = new Map([[srcId, 0]]);
    const came = new Map();
    const vis = new Set();
    let iter=0;
    while(heap.size && iter++<MAX_ITER) {
      const [,cur] = heap.pop();
      if(vis.has(cur)) continue;
      vis.add(cur);
      if(cur === dstId) {
        const path = [];
        let c = cur;
        while(came.has(c)) { path.push(c); c = came.get(c); }
        path.push(srcId);
        return path.reverse();
      }
      const neighbors = [...(extraEdges[cur]||[]), ...(typeof cur === 'number' ? (wpGraph[cur]||[]) : [])];
      for(const nb of neighbors) {
        if(vis.has(nb)) continue;
        const ng = (g.get(cur)||0) + dist(coordOf(cur), coordOf(nb));
        if(ng < (g.get(nb) ?? Infinity)) {
          g.set(nb, ng);
          came.set(nb, cur);
          const h = dist(coordOf(nb), coordOf(dstId));
          heap.push([ng + h, nb]);
        }
      }
    }
    return null;
  }
  
  async function runRoute() {
    if (!start || !end) { alert("⚠️ Tentukan START dan FINISH dulu!"); return; }
    if (isRouting) return;
    if (!wpReady || waypoints.length===0) { alert("Data waypoint belum siap."); return; }
    isRouting = true;
    showProgress(0.1);
    setStatus("🔍 Mencari waypoint...", "#f39c12");
    await tick();
    if (isBlocked(start[0], start[1])) { alert("START di daratan!"); setStatus("❌ START daratan","red"); isRouting=false; hideProgress(); return; }
    if (isBlocked(end[0], end[1])) { alert("FINISH di daratan!"); setStatus("❌ FINISH daratan","red"); isRouting=false; hideProgress(); return; }
    
    const startCands = findNearestWaypoints(start, 14);
    const endCands = findNearestWaypoints(end, 14);
    const startWPs = pickReachable(start, startCands);
    const endWPs = pickReachable(end, endCands);
    if (!startWPs.length || !endWPs.length) {
      setStatus("❌ Tidak terhubung ke waypoint", "red");
      isRouting=false; hideProgress(); return;
    }
    showProgress(0.35);
    const extraEdges = { 'S': [], 'E': [] };
    for (const {i} of startWPs) { extraEdges['S'].push(i); if(!extraEdges[i]) extraEdges[i]=[]; extraEdges[i].push('S'); }
    for (const {i} of endWPs) { extraEdges['E'].push(i); if(!extraEdges[i]) extraEdges[i]=[]; extraEdges[i].push('E'); }
    if(startWPs.some(s=>endWPs.some(e=>e.i===s.i))){ extraEdges['S'].push('E'); extraEdges['E'].push('S'); }
    const coordOf = id => {
      if(id==='S') return start;
      if(id==='E') return end;
      return [waypoints[id].lat, waypoints[id].lng];
    };
    const pathIds = astarWithVirtual('S','E', extraEdges, coordOf);
    showProgress(0.85);
    if (!pathIds || pathIds.length<2) {
      if(canSee(start, end, 45)) {
        const coords = [start,end];
        drawRoute(coords);
        const km = distKm(start,end).toFixed(1);
        setStatus(`✅ Jalur Langsung: ${km} km`, "#2ecc71");
        setInfo("Rute LOS langsung tanpa waypoint.");
      } else {
        setStatus("❌ Rute tidak ditemukan", "red");
      }
      showProgress(1); setTimeout(hideProgress,500); isRouting=false; return;
    }
    const coords = pathIds.map(id=>coordOf(id));
    drawRoute(coords);
    let totalKm=0;
    for(let i=1;i<coords.length;i++) totalKm += distKm(coords[i-1],coords[i]);
    const wpCount = pathIds.filter(id=>id!=='S'&&id!=='E').length;
    setStatus(`✅ ${totalKm.toFixed(1)} km · ${(totalKm/1.852).toFixed(0)} nm`, "#2c8c6e");
    setInfo(`🎯 Rute via ${wpCount} waypoint. Jarak ${totalKm.toFixed(1)} km. Klik animasi.`);
    showProgress(1);
    setTimeout(hideProgress,600);
    isRouting=false;
  }
  
  function drawRoute(coords) {
    currentRouteCoords = coords;
    if(routeLine) map.removeLayer(routeLine);
    routeLine = L.polyline(coords, { color: '#1f85c7', weight: 5, opacity: 0.9, lineJoin: 'round' }).addTo(map);
    map.fitBounds(routeLine.getBounds(), { padding: [50,50] });
  }
  
  function stopAnimation() {
    if(animationInterval) { clearInterval(animationInterval); animationInterval = null; }
  }
  
  // ========== FUNGSI IKON KAPAL DENGAN KETERANGAN KECIL ==========
  function createShipIcon() {
    const shipName = document.getElementById('shipName').value || "Kapal";
    const shipMMSI = document.getElementById('shipMMSI').value || "-";
    const shipLength = document.getElementById('shipLength').value || "?";
    const shipDraft = document.getElementById('shipDraft').value || "?";
    
    // Ikon div dengan styling lebih kecil dan ringkas
    return L.divIcon({
      html: `
        <div style="
          background: #1a5276; 
          border-radius: 20px; 
          padding: 3px 10px; 
          color: white; 
          font-weight: 500; 
          font-size: 9px; 
          white-space: nowrap; 
          box-shadow: 0 2px 6px rgba(0,0,0,0.3); 
          border: 1px solid #f1c40f;
          display: flex;
          align-items: center;
          gap: 4px;
          line-height: 1.2;
        ">
          <i class="fas fa-ship" style="font-size: 10px; margin-right: 2px;"></i>
          <div style="display: flex; flex-direction: column;">
            <span style="font-weight: 700; font-size: 8.5px;">${shipName.substring(0, 15)}</span>
            <span style="font-size: 7px; opacity: 0.9; display: flex; gap: 6px;">
              <span><i class="fas fa-id-card" style="font-size:6px;"></i> ${shipMMSI.substring(0,9)}</span>
              <span>📏 ${shipLength}m</span>
              <span>⚓ ${shipDraft}m</span>
            </span>
          </div>
        </div>
      `,
      iconSize: [150, 36],
      iconAnchor: [75, 18],
      className: 'ship-label'
    });
  }
  
  function startAnimation() {
    if(!currentRouteCoords || currentRouteCoords.length < 2) {
      alert("Tidak ada rute. Hitung rute terlebih dahulu!");
      return;
    }
    stopAnimation();
    if(shipMarker) map.removeLayer(shipMarker);
    currentAnimIndex = 0;
    shipMarker = L.marker(currentRouteCoords[0], { icon: createShipIcon() }).addTo(map);
    map.setView(currentRouteCoords[0], 12);
    
    let segmentIndex = 0;
    let t = 0;
    const speed = 0.008;
    animationInterval = setInterval(() => {
      if(segmentIndex >= currentRouteCoords.length-1) {
        stopAnimation();
        setStatus("✅ Simulasi selesai. Kapal tiba.", "#2c8c6e");
        return;
      }
      const p1 = currentRouteCoords[segmentIndex];
      const p2 = currentRouteCoords[segmentIndex+1];
      t += speed;
      if(t >= 1) {
        t = 0;
        segmentIndex++;
        if(segmentIndex < currentRouteCoords.length) shipMarker.setLatLng(currentRouteCoords[segmentIndex]);
        shipMarker.setIcon(createShipIcon());
      } else {
        const lat = p1[0] + (p2[0]-p1[0])*t;
        const lng = p1[1] + (p2[1]-p1[1])*t;
        shipMarker.setLatLng([lat, lng]);
        shipMarker.setIcon(createShipIcon());
      }
    }, 80);
    setStatus("🚢 Animasi kapal berjalan...", "#2c9b6e");
  }
  
  function updateStartMarker(lat,lng){
    if(startMarker) map.removeLayer(startMarker);
    startMarker = L.marker([lat,lng], { icon: L.divIcon({ html: `<i class="fas fa-anchor" style="font-size:28px; color:#1f7e9e;"></i>`, iconSize: [28,28], iconAnchor: [14,28] }) }).addTo(map).bindPopup("START").openPopup();
  }
  function updateEndMarker(lat,lng){
    if(endMarker) map.removeLayer(endMarker);
    endMarker = L.marker([lat,lng], { icon: L.divIcon({ html: `<i class="fas fa-flag-checkered" style="font-size:28px; color:#d97706;"></i>`, iconSize: [28,28], iconAnchor: [14,28] }) }).addTo(map).bindPopup("FINISH");
  }
  
  function applyStartFromInput(){
    let lat = parseFloat(document.getElementById('startLat').value);
    let lng = parseFloat(document.getElementById('startLng').value);
    if(isNaN(lat)||isNaN(lng)){ alert("Masukkan angka valid untuk START"); return; }
    start=[lat,lng];
    updateStartMarker(lat,lng);
    setStatus(`✅ Start: ${lat.toFixed(5)}, ${lng.toFixed(5)}`, "#1f7a5c");
    if(routeLine){ map.removeLayer(routeLine); routeLine=null; currentRouteCoords=[]; stopAnimation(); if(shipMarker) map.removeLayer(shipMarker); }
  }
  function applyEndFromInput(){
    let lat = parseFloat(document.getElementById('endLat').value);
    let lng = parseFloat(document.getElementById('endLng').value);
    if(isNaN(lat)||isNaN(lng)){ alert("Masukkan angka valid untuk FINISH"); return; }
    end=[lat,lng];
    updateEndMarker(lat,lng);
    setStatus(`✅ Finish: ${lat.toFixed(5)}, ${lng.toFixed(5)}`, "#4a7c9c");
    if(routeLine){ map.removeLayer(routeLine); routeLine=null; currentRouteCoords=[]; stopAnimation(); if(shipMarker) map.removeLayer(shipMarker); }
  }
  
  function setModeSelection(m) {
    if(isRouting) return;
    mode = m;
    setStatus(`🖱️ Klik di peta untuk ${mode==='start'?'START':'FINISH'}`, "#2266aa");
  }
  
  function clearAll(){
    if(isRouting) return;
    if(routeLine) map.removeLayer(routeLine);
    if(startMarker) map.removeLayer(startMarker);
    if(endMarker) map.removeLayer(endMarker);
    if(shipMarker) map.removeLayer(shipMarker);
    stopAnimation();
    start=end=null; routeLine=startMarker=endMarker=null; mode=null; currentRouteCoords=[];
    document.getElementById('startLat').value='';
    document.getElementById('startLng').value='';
    document.getElementById('endLat').value='';
    document.getElementById('endLng').value='';
    setStatus("Reset total.","#447799");
    setInfo("Data dihapus. Masukkan koordinat baru.");
  }
  
  map.on('click', e=>{
    if(!mode || isRouting) return;
    const lat=e.latlng.lat, lng=e.latlng.lng;
    if(islands.length && isBlocked(lat,lng)){ alert("❌ Titik di daratan! Pilih di laut."); return; }
    if(mode==='start'){
      start=[lat,lng];
      updateStartMarker(lat,lng);
      document.getElementById('startLat').value=lat.toFixed(6);
      document.getElementById('startLng').value=lng.toFixed(6);
      setStatus(`📍 Start dari peta: ${lat.toFixed(5)}, ${lng.toFixed(5)}`,"#368f6b");
    } else {
      end=[lat,lng];
      updateEndMarker(lat,lng);
      document.getElementById('endLat').value=lat.toFixed(6);
      document.getElementById('endLng').value=lng.toFixed(6);
      setStatus(`🏁 Finish dari peta: ${lat.toFixed(5)}, ${lng.toFixed(5)}`,"#667fa0");
    }
    if(routeLine){ map.removeLayer(routeLine); routeLine=null; currentRouteCoords=[]; stopAnimation(); if(shipMarker) map.removeLayer(shipMarker); }
    mode=null;
  });
  
  // ========== NAVBAR COLLAPSE ==========
  const nav = document.getElementById('marineNav');
  const toggleNavBtn = document.getElementById('navToggleBtn');
  const toggleIconSpan = document.getElementById('toggleIcon');
  
  function toggleNavPanel() {
    nav.classList.toggle('collapsed');
    if(nav.classList.contains('collapsed')) {
      toggleIconSpan.className = 'fas fa-chevron-down';
    } else {
      toggleIconSpan.className = 'fas fa-chevron-up';
    }
  }
  
  toggleNavBtn.addEventListener('click', (e) => { 
    e.stopPropagation(); 
    toggleNavPanel(); 
  });
  
  document.getElementById('navHeader').addEventListener('click', (e) => { 
    if(e.target !== toggleNavBtn && !toggleNavBtn.contains(e.target)) {
      toggleNavPanel(); 
    }
  });
  
  if (nav.classList.contains('collapsed')) {
    toggleIconSpan.className = 'fas fa-chevron-down';
  } else {
    toggleIconSpan.className = 'fas fa-chevron-up';
  }
  
  // EVENT LISTENERS
  document.getElementById('applyStartBtn').addEventListener('click', applyStartFromInput);
  document.getElementById('applyEndBtn').addEventListener('click', applyEndFromInput);
  document.getElementById('pickStartMapBtn').addEventListener('click', ()=>setModeSelection('start'));
  document.getElementById('pickEndMapBtn').addEventListener('click', ()=>setModeSelection('end'));
  document.getElementById('runRouteBtn').addEventListener('click', runRoute);
  document.getElementById('animateBtn').addEventListener('click', startAnimation);
  document.getElementById('clearAllBtn').addEventListener('click', clearAll);
  
  window.addEventListener('load', ()=>{
    setStatus("🌊 Marine Router + Animasi Kapal siap (Mode Satelit).", "#2f9e6b");
  });
</script>
</body>
</html>
