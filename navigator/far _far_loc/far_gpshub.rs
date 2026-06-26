use chrono::{DateTime, Local, Timelike};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, Write};

// Struktur untuk data koordinat GPS
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Coordinates {
    latitude: f64,
    longitude: f64,
    altitude: f64,
}

// Struktur untuk data GPS lengkap
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GPSData {
    coordinates: Coordinates,
    timestamp: DateTime<Local>,
    speed: f64,      // km/h
    heading: f64,    // derajat
    accuracy: f64,   // meter
    satellites: u8,
    fix_type: String,
}

// Sistem GPS dengan history
#[derive(Debug, Serialize, Deserialize)]
struct GPSSystem {
    current_location: GPSData,
    history: VecDeque<GPSData>,
    max_history: usize,
    is_tracking: bool,
}

// Error handling
#[derive(Debug)]
enum GPSError {
    InvalidCoordinates,
    NoFix,
    OutOfRange,
    IoError(std::io::Error),
}

impl From<std::io::Error> for GPSError {
    fn from(err: std::io::Error) -> Self {
        GPSError::IoError(err)
    }
}

impl Coordinates {
    // Validasi koordinat
    fn is_valid(&self) -> bool {
        (-90.0..=90.0).contains(&self.latitude) && 
        (-180.0..=180.0).contains(&self.longitude) &&
        self.altitude >= 0.0
    }

    // Menghitung jarak antara dua titik (Haversine formula)
    fn distance_to(&self, other: &Coordinates) -> f64 {
        let earth_radius = 6371.0; // km
        
        let lat1 = self.latitude.to_radians();
        let lon1 = self.longitude.to_radians();
        let lat2 = other.latitude.to_radians();
        let lon2 = other.longitude.to_radians();
        
        let dlat = lat2 - lat1;
        let dlon = lon2 - lon1;
        
        let a = (dlat / 2.0).sin().powi(2) + 
                lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        
        earth_radius * c
    }
}

impl GPSData {
    // Membuat data GPS baru
    fn new(lat: f64, lon: f64, alt: f64) -> Result<Self, GPSError> {
        let coords = Coordinates {
            latitude: lat,
            longitude: lon,
            altitude: alt,
        };
        
        if !coords.is_valid() {
            return Err(GPSError::InvalidCoordinates);
        }
        
        Ok(GPSData {
            coordinates: coords,
            timestamp: Local::now(),
            speed: 0.0,
            heading: 0.0,
            accuracy: 5.0,
            satellites: 8,
            fix_type: "3D".to_string(),
        })
    }

    // Simulasi pergerakan
    fn simulate_movement(&mut self, delta_lat: f64, delta_lon: f64) {
        self.coordinates.latitude += delta_lat;
        self.coordinates.longitude += delta_lon;
        self.timestamp = Local::now();
        
        // Update speed berdasarkan pergerakan
        self.speed = (delta_lat.abs() + delta_lon.abs()) * 100.0;
    }
}

impl GPSSystem {
    // Membuat sistem GPS baru
    fn new() -> Self {
        GPSSystem {
            current_location: GPSData::new(0.0, 0.0, 0.0).unwrap(),
            history: VecDeque::with_capacity(100),
            max_history: 100,
            is_tracking: false,
        }
    }

    // Set lokasi awal
    fn set_initial_location(&mut self, lat: f64, lon: f64, alt: f64) -> Result<(), GPSError> {
        let mut data = GPSData::new(lat, lon, alt)?;
        data.fix_type = "3D".to_string();
        data.satellites = 10;
        self.current_location = data;
        self.add_to_history();
        Ok(())
    }

    // Update lokasi
    fn update_location(&mut self, lat: f64, lon: f64, alt: f64) -> Result<(), GPSError> {
        let mut data = GPSData::new(lat, lon, alt)?;
        
        // Hitung speed dari pergerakan
        if let Some(last) = self.history.back() {
            let distance = last.coordinates.distance_to(&data.coordinates);
            let time_diff = data.timestamp - last.timestamp;
            let hours = time_diff.num_seconds() as f64 / 3600.0;
            
            if hours > 0.0 {
                data.speed = distance / hours;
            }
            
            // Hitung heading
            let dlat = data.coordinates.latitude - last.coordinates.latitude;
            let dlon = data.coordinates.longitude - last.coordinates.longitude;
            if dlat != 0.0 || dlon != 0.0 {
                data.heading = dlat.atan2(dlon).to_degrees();
                if data.heading < 0.0 {
                    data.heading += 360.0;
                }
            }
        }
        
        self.current_location = data;
        self.add_to_history();
        Ok(())
    }

    // Tambahkan ke history
    fn add_to_history(&mut self) {
        self.history.push_back(self.current_location.clone());
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }
    }

    // Mulai tracking
    fn start_tracking(&mut self) {
        self.is_tracking = true;
        println!("🟢 GPS Tracking Started");
    }

    // Stop tracking
    fn stop_tracking(&mut self) {
        self.is_tracking = false;
        println!("🔴 GPS Tracking Stopped");
    }

    // Simulasi tracking berjalan
    fn simulate_tracking(&mut self, steps: usize, interval_seconds: u64) {
        if !self.is_tracking {
            println!("⚠️  Start tracking first!");
            return;
        }

        let mut rng = rand::thread_rng();
        
        for i in 0..steps {
            // Simulasi pergerakan acak
            let delta_lat = rng.gen_range(-0.001..0.001);
            let delta_lon = rng.gen_range(-0.001..0.001);
            let alt_change = rng.gen_range(-1.0..1.0);
            
            let new_lat = self.current_location.coordinates.latitude + delta_lat;
            let new_lon = self.current_location.coordinates.longitude + delta_lon;
            let new_alt = (self.current_location.coordinates.altitude + alt_change).max(0.0);
            
            match self.update_location(new_lat, new_lon, new_alt) {
                Ok(_) => {
                    self.display_current_location();
                    println!("Step {}/{} completed", i + 1, steps);
                }
                Err(e) => println!("Error: {:?}", e),
            }
            
            // Tunggu sebentar
            if i < steps - 1 {
                std::thread::sleep(std::time::Duration::from_secs(interval_seconds));
            }
        }
    }

    // Tampilkan lokasi saat ini
    fn display_current_location(&self) {
        let data = &self.current_location;
        let time = data.timestamp.format("%H:%M:%S");
        
        println!("\n📍 CURRENT LOCATION");
        println!("─────────────────────");
        println!("Latitude:  {:.6}°", data.coordinates.latitude);
        println!("Longitude: {:.6}°", data.coordinates.longitude);
        println!("Altitude:  {:.1} m", data.coordinates.altitude);
        println!("Speed:     {:.1} km/h", data.speed);
        println!("Heading:   {:.1}°", data.heading);
        println!("Accuracy:  ±{:.1} m", data.accuracy);
        println!("Satellites: {}", data.satellites);
        println!("Fix Type:  {}", data.fix_type);
        println!("Time:      {}", time);
        println!("Tracking:  {}", if self.is_tracking { "ON" } else { "OFF" });
        println!("─────────────────────");
    }

    // Tampilkan history
    fn display_history(&self) {
        println!("\n📜 GPS HISTORY (Last {} entries)", self.history.len());
        println!("─────────────────────────────────────");
        
        for (i, data) in self.history.iter().enumerate() {
            let time = data.timestamp.format("%H:%M:%S");
            println!(
                "{:2}. [{:8}] Lat: {:.6}°, Lon: {:.6}°, Alt: {:.1}m, Speed: {:.1} km/h",
                i + 1,
                time,
                data.coordinates.latitude,
                data.coordinates.longitude,
                data.coordinates.altitude,
                data.speed
            );
        }
        println!("─────────────────────────────────────");
    }

    // Simpan data ke file
    fn save_to_file(&self, filename: &str) -> Result<(), GPSError> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(filename)?;
        file.write_all(json.as_bytes())?;
        println!("✅ Data saved to {}", filename);
        Ok(())
    }

    // Load data dari file
    fn load_from_file(filename: &str) -> Result<Self, GPSError> {
        let file = File::open(filename)?;
        let system: GPSSystem = serde_json::from_reader(file)?;
        println!("✅ Data loaded from {}", filename);
        Ok(system)
    }

    // Dapatkan statistik perjalanan
    fn get_trip_stats(&self) -> TripStats {
        let mut total_distance = 0.0;
        let mut max_speed = 0.0;
        
        if self.history.len() < 2 {
            return TripStats::default();
        }
        
        for i in 1..self.history.len() {
            let prev = &self.history[i - 1];
            let curr = &self.history[i];
            let distance = prev.coordinates.distance_to(&curr.coordinates);
            total_distance += distance;
            
            if curr.speed > max_speed {
                max_speed = curr.speed;
            }
        }
        
        TripStats {
            total_distance,
            max_speed,
            avg_speed: total_distance / (self.history.len() - 1) as f64,
            total_time: if let (Some(first), Some(last)) = (self.history.front(), self.history.back()) {
                last.timestamp - first.timestamp
            } else {
                chrono::Duration::zero()
            },
        }
    }
}

// Statistik perjalanan
#[derive(Debug, Default)]
struct TripStats {
    total_distance: f64,
    max_speed: f64,
    avg_speed: f64,
    total_time: chrono::Duration,
}

impl TripStats {
    fn display(&self) {
        println!("\n📊 TRIP STATISTICS");
        println!("─────────────────────");
        println!("Total Distance: {:.2} km", self.total_distance);
        println!("Max Speed:      {:.1} km/h", self.max_speed);
        println!("Average Speed:  {:.1} km/h", self.avg_speed);
        println!("Total Time:     {} seconds", self.total_time.num_seconds());
        println!("─────────────────────");
    }
}

// Fungsi menu utama
fn main_menu() {
    println!("\n┌─────────────────────────────────┐");
    println!("│       GPS SYSTEM MENU          │");
    println!("├─────────────────────────────────┤");
    println!("│ 1. Set Initial Location        │");
    println!("│ 2. Update Location             │");
    println!("│ 3. Start Tracking              │");
    println!("│ 4. Stop Tracking               │");
    println!("│ 5. Simulate Tracking           │");
    println!("│ 6. Display Current Location    │");
    println!("│ 7. Display History             │");
    println!("│ 8. Show Trip Statistics        │");
    println!("│ 9. Save Data                   │");
    println!("│ 10. Load Data                  │");
    println!("│ 11. Exit                       │");
    println!("└─────────────────────────────────┘");
}

fn main() {
    println!("🛰️  GPS SYSTEM IN RUST 🛰️");
    println!("===========================\n");
    
    let mut gps = GPSSystem::new();
    
    // Set lokasi awal (Jakarta)
    match gps.set_initial_location(-6.2088, 106.8456, 10.0) {
        Ok(_) => println!("✅ Initial location set to Jakarta"),
        Err(e) => println!("❌ Error: {:?}", e),
    }
    
    loop {
        main_menu();
        
        print!("Enter your choice: ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        
        match input.trim() {
            "1" => {
                println!("Enter coordinates (lat lon alt):");
                let mut coords = String::new();
                io::stdin().read_line(&mut coords).unwrap();
                let parts: Vec<f64> = coords
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();
                
                if parts.len() == 3 {
                    match gps.set_initial_location(parts[0], parts[1], parts[2]) {
                        Ok(_) => println!("✅ Location updated"),
                        Err(e) => println!("❌ Error: {:?}", e),
                    }
                } else {
                    println!("❌ Invalid input. Format: lat lon alt");
                }
            }
            "2" => {
                println!("Enter new coordinates (lat lon alt):");
                let mut coords = String::new();
                io::stdin().read_line(&mut coords).unwrap();
                let parts: Vec<f64> = coords
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();
                
                if parts.len() == 3 {
                    match gps.update_location(parts[0], parts[1], parts[2]) {
                        Ok(_) => println!("✅ Location updated"),
                        Err(e) => println!("❌ Error: {:?}", e),
                    }
                } else {
                    println!("❌ Invalid input. Format: lat lon alt");
                }
            }
            "3" => gps.start_tracking(),
            "4" => gps.stop_tracking(),
            "5" => {
                println!("How many steps to simulate?");
                let mut steps = String::new();
                io::stdin().read_line(&mut steps).unwrap();
                let steps: usize = steps.trim().parse().unwrap_or(10);
                
                println!("Interval between steps (seconds)?");
                let mut interval = String::new();
                io::stdin().read_line(&mut interval).unwrap();
                let interval: u64 = interval.trim().parse().unwrap_or(1);
                
                gps.simulate_tracking(steps, interval);
            }
            "6" => gps.display_current_location(),
            "7" => gps.display_history(),
            "8" => {
                let stats = gps.get_trip_stats();
                stats.display();
            }
            "9" => {
                println!("Enter filename to save (e.g., gps_data.json):");
                let mut filename = String::new();
                io::stdin().read_line(&mut filename).unwrap();
                let filename = filename.trim();
                
                match gps.save_to_file(filename) {
                    Ok(_) => println!("✅ Data saved successfully"),
                    Err(e) => println!("❌ Error saving: {:?}", e),
                }
            }
            "10" => {
                println!("Enter filename to load (e.g., gps_data.json):");
                let mut filename = String::new();
                io::stdin().read_line(&mut filename).unwrap();
                let filename = filename.trim();
                
                match GPSSystem::load_from_file(filename) {
                    Ok(loaded) => {
                        gps = loaded;
                        println!("✅ Data loaded successfully");
                    }
                    Err(e) => println!("❌ Error loading: {:?}", e),
                }
            }
            "11" => {
                println!("👋 Goodbye!");
                break;
            }
            _ => println!("❌ Invalid option. Please try again."),
        }
    }
}

// Test modul
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinates_validation() {
        let valid = Coordinates {
            latitude: 0.0,
            longitude: 0.0,
            altitude: 0.0,
        };
        assert!(valid.is_valid());

        let invalid = Coordinates {
            latitude: 100.0,
            longitude: 0.0,
            altitude: 0.0,
        };
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_distance_calculation() {
        let jakarta = Coordinates {
            latitude: -6.2088,
            longitude: 106.8456,
            altitude: 10.0,
        };
        
        let bandung = Coordinates {
            latitude: -6.9175,
            longitude: 107.6191,
            altitude: 768.0,
        };
        
        let distance = jakarta.distance_to(&bandung);
        assert!(distance > 100.0 && distance < 150.0);
    }
}
