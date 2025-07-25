use std::collections::VecDeque;
use sysinfo::{Disks, System, Networks};
use std::time::Instant;

pub struct SystemMetrics {
    cpu_history: VecDeque<f32>,
    memory_history: VecDeque<f32>,
    disk_history: VecDeque<f32>,
    
    // Network monitoring data
    network_rx_history: VecDeque<f32>,  // Download rate in Kbps
    network_tx_history: VecDeque<f32>,  // Upload rate in Kbps
    prev_rx_bytes: u64,
    prev_tx_bytes: u64,
    initial_rx_bytes: u64,  // Baseline for session totals
    initial_tx_bytes: u64,  // Baseline for session totals
    networks: Networks,
    last_network_update: Instant,
    
    // Per-core CPU data
    per_core_usage: Vec<f32>,
    per_core_temperatures: Vec<f32>,

    // GPU data (NVIDIA via nvidia-smi)
    gpu_usage: Option<f32>,
    gpu_temperature: Option<f32>,
    gpu_fan_speed: Option<f32>,       // Fan speed in percentage
    gpu_power_draw: Option<f32>,      // Power usage in watts
    gpu_memory_used: Option<f32>,     // VRAM used in MB
    gpu_memory_total: Option<f32>,    // Total VRAM in MB
    gpu_name: Option<String>,         // GPU name for display
    
    // GPU history for charts
    gpu_usage_history: VecDeque<f32>,
    gpu_memory_percent_history: VecDeque<f32>,
    
    max_history: usize,
}

impl SystemMetrics {
    pub fn new(max_history: usize) -> Self {
        let mut networks = Networks::new();
        networks.refresh_list();
        
        // Get initial network byte counts to use as baseline (reset point)
        let mut initial_rx_bytes = 0;
        let mut initial_tx_bytes = 0;
        
        for (interface_name, network) in &networks {
            if interface_name != "lo" && !interface_name.starts_with("virbr") && !interface_name.starts_with("docker") && !interface_name.starts_with("veth") {
                initial_rx_bytes += network.total_received();
                initial_tx_bytes += network.total_transmitted();
            }
        }
        
        Self {
            cpu_history: VecDeque::with_capacity(max_history),
            memory_history: VecDeque::with_capacity(max_history),
            disk_history: VecDeque::with_capacity(max_history),
            network_rx_history: VecDeque::with_capacity(max_history),
            network_tx_history: VecDeque::with_capacity(max_history),
            prev_rx_bytes: initial_rx_bytes,
            prev_tx_bytes: initial_tx_bytes,
            initial_rx_bytes,
            initial_tx_bytes,
            networks,
            last_network_update: Instant::now(),
            per_core_usage: Vec::new(),
            per_core_temperatures: Vec::new(),
            gpu_usage: None,
            gpu_temperature: None,
            gpu_fan_speed: None,
            gpu_power_draw: None,
            gpu_memory_used: None,
            gpu_memory_total: None,
            gpu_name: None,
            gpu_usage_history: VecDeque::with_capacity(max_history),
            gpu_memory_percent_history: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    pub fn update(&mut self, system: &System) {
        // Update CPU usage
        let cpu_usage = system.global_cpu_usage();
        if self.cpu_history.len() >= self.max_history {
            self.cpu_history.pop_front();
        }
        self.cpu_history.push_back(cpu_usage);

        // Update per-core CPU usage
        self.per_core_usage.clear();
        for cpu in system.cpus() {
            self.per_core_usage.push(cpu.cpu_usage());
        }

        // Update per-core temperatures
        self.update_per_core_temperatures();

        // Update memory usage
        let memory_usage = (system.used_memory() as f32 / system.total_memory() as f32) * 100.0;
        if self.memory_history.len() >= self.max_history {
            self.memory_history.pop_front();
        }
        self.memory_history.push_back(memory_usage);

        // Update disk usage (root filesystem)
        let mut disk_usage = 0.0;
        let disks = Disks::new_with_refreshed_list();
        for disk in &disks {
            if disk.mount_point().to_str() == Some("/") {
                let total = disk.total_space() as f32;
                let available = disk.available_space() as f32;
                disk_usage = ((total - available) / total) * 100.0;
                break;
            }
        }
        if self.disk_history.len() >= self.max_history {
            self.disk_history.pop_front();
        }
        self.disk_history.push_back(disk_usage);

        // Update network usage
        self.update_network_stats();

        // Update GPU usage/temperature if available
        self.update_gpu_stats();
        
        // Update GPU history
        self.update_gpu_history();
    }


    pub fn cpu_usage(&self) -> f32 {
        self.cpu_history.back().copied().unwrap_or(0.0)
    }

    pub fn memory_usage(&self) -> f32 {
        self.memory_history.back().copied().unwrap_or(0.0)
    }

    pub fn disk_usage(&self) -> f32 {
        self.disk_history.back().copied().unwrap_or(0.0)
    }

    pub fn cpu_history(&self) -> &VecDeque<f32> {
        &self.cpu_history
    }

    pub fn memory_history(&self) -> &VecDeque<f32> {
        &self.memory_history
    }

    pub fn disk_history(&self) -> &VecDeque<f32> {
        &self.disk_history
    }

    pub fn network_download_rate(&self) -> f32 {
        self.network_rx_history.back().copied().unwrap_or(0.0)
    }

    pub fn network_upload_rate(&self) -> f32 {
        self.network_tx_history.back().copied().unwrap_or(0.0)
    }

    pub fn network_rx_history(&self) -> &VecDeque<f32> {
        &self.network_rx_history
    }

    pub fn network_tx_history(&self) -> &VecDeque<f32> {
        &self.network_tx_history
    }


    pub fn per_core_usage(&self) -> &[f32] {
        &self.per_core_usage
    }

    pub fn per_core_temperatures(&self) -> &[f32] {
        &self.per_core_temperatures
    }

    pub fn gpu_usage(&self) -> Option<f32> {
        self.gpu_usage
    }

    pub fn gpu_temperature(&self) -> Option<f32> {
        self.gpu_temperature
    }

    pub fn gpu_fan_speed(&self) -> Option<f32> {
        self.gpu_fan_speed
    }

    pub fn gpu_power_draw(&self) -> Option<f32> {
        self.gpu_power_draw
    }

    pub fn gpu_memory_used(&self) -> Option<f32> {
        self.gpu_memory_used
    }

    pub fn gpu_memory_total(&self) -> Option<f32> {
        self.gpu_memory_total
    }

    pub fn gpu_memory_usage_percent(&self) -> Option<f32> {
        if let (Some(used), Some(total)) = (self.gpu_memory_used, self.gpu_memory_total) {
            if total > 0.0 {
                Some((used / total) * 100.0)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn gpu_name(&self) -> Option<&String> {
        self.gpu_name.as_ref()
    }

    pub fn gpu_usage_history(&self) -> &VecDeque<f32> {
        &self.gpu_usage_history
    }

    pub fn gpu_memory_percent_history(&self) -> &VecDeque<f32> {
        &self.gpu_memory_percent_history
    }

    fn update_network_stats(&mut self) {
        // Refresh network data
        self.networks.refresh();
        
        let mut total_rx_bytes = 0;
        let mut total_tx_bytes = 0;
        
        // Sum up bytes from all network interfaces (excluding loopback)
        for (interface_name, network) in &self.networks {
            if interface_name != "lo" && !interface_name.starts_with("virbr") && !interface_name.starts_with("docker") && !interface_name.starts_with("veth") {
                total_rx_bytes += network.total_received();
                total_tx_bytes += network.total_transmitted();
                // Debug: uncomment to see which interfaces are being monitored
                // eprintln!("Interface: {}, RX: {}, TX: {}", interface_name, network.total_received(), network.total_transmitted());
            }
        }
        
        // Calculate time elapsed since last update
        let now = Instant::now();
        let time_diff = now.duration_since(self.last_network_update).as_secs_f32();
        self.last_network_update = now;
        
        // Calculate rates (bytes per second, converted to Kbps)
        let rx_rate = if self.prev_rx_bytes > 0 && time_diff > 0.0 {
            let bytes_diff = total_rx_bytes.saturating_sub(self.prev_rx_bytes);
            (bytes_diff as f32) / time_diff * 8.0 / 1000.0 // Convert to Kbps (bits per second / 1000)
        } else {
            0.0
        };
        
        let tx_rate = if self.prev_tx_bytes > 0 && time_diff > 0.0 {
            let bytes_diff = total_tx_bytes.saturating_sub(self.prev_tx_bytes);
            (bytes_diff as f32) / time_diff * 8.0 / 1000.0 // Convert to Kbps (bits per second / 1000)
        } else {
            0.0
        };
        
        // Update history
        if self.network_rx_history.len() >= self.max_history {
            self.network_rx_history.pop_front();
        }
        self.network_rx_history.push_back(rx_rate);
        
        if self.network_tx_history.len() >= self.max_history {
            self.network_tx_history.pop_front();
        }
        self.network_tx_history.push_back(tx_rate);
        
        // Store current values for next calculation
        self.prev_rx_bytes = total_rx_bytes;
        self.prev_tx_bytes = total_tx_bytes;
    }

    pub fn total_network_bytes(&self) -> (u64, u64) {
        // Return session-relative totals (current - initial)
        let session_rx = self.prev_rx_bytes.saturating_sub(self.initial_rx_bytes);
        let session_tx = self.prev_tx_bytes.saturating_sub(self.initial_tx_bytes);
        (session_rx, session_tx)
    }

    pub fn cpu_temperature(&self) -> Option<f32> {
        // First try hwmon sensors (more reliable for package temp)
        if let Some(temp) = self.read_hwmon_temperature() {
            return Some(temp);
        }
        // Fallback to thermal zones
        self.read_thermal_zone()
    }
    
    fn read_thermal_zone(&self) -> Option<f32> {
        use std::fs;
        
        // Common thermal zone paths for CPU temperature
        let thermal_paths = [
            "/sys/class/thermal/thermal_zone0/temp",
            "/sys/class/thermal/thermal_zone1/temp",
            "/sys/class/thermal/thermal_zone2/temp",
        ];
        
        for path in &thermal_paths {
            if let Ok(temp_str) = fs::read_to_string(path) {
                if let Ok(temp_milli) = temp_str.trim().parse::<i32>() {
                    // Temperature is in millidegrees Celsius
                    let temp_celsius = temp_milli as f32 / 1000.0;
                    // Return reasonable CPU temperatures (typically 20-90°C)
                    if temp_celsius > 10.0 && temp_celsius < 150.0 {
                        return Some(temp_celsius);
                    }
                }
            }
        }
        
        // Try hwmon sensors
        self.read_hwmon_temperature()
    }
    
    fn read_hwmon_temperature(&self) -> Option<f32> {
        use std::fs;
        
        // Look for coretemp or CPU-related hwmon sensors
        let hwmon_base = "/sys/class/hwmon";
        let mut package_candidates = Vec::new();
        let mut fallback_candidates = Vec::new();
        
        if let Ok(entries) = fs::read_dir(hwmon_base) {
            for entry in entries.flatten() {
                let hwmon_path = entry.path();
                
                // Check if this is a CPU temperature sensor
                if let Ok(name) = fs::read_to_string(hwmon_path.join("name")) {
                    let name = name.trim().to_lowercase();
                    if name.contains("coretemp") || name.contains("cpu") || name.contains("k10temp") {
                        // Look through all temp sensors in this hwmon device
                        for temp_num in 1..=10 {
                            let temp_file = hwmon_path.join(format!("temp{}_input", temp_num));
                            let label_file = hwmon_path.join(format!("temp{}_label", temp_num));
                            
                            if let Ok(temp_str) = fs::read_to_string(&temp_file) {
                                if let Ok(temp_milli) = temp_str.trim().parse::<i32>() {
                                    let temp_celsius = temp_milli as f32 / 1000.0;
                                    
                                    if temp_celsius > 10.0 && temp_celsius < 150.0 {
                                        // Check if this has a package/pkg label (highest priority)
                                        if let Ok(label_data) = fs::read_to_string(&label_file) {
                                            let label = label_data.trim().to_lowercase();
                                            if label.contains("package") || label.contains("pkg") {
                                                package_candidates.push(temp_celsius);
                                            } else if temp_num == 1 {
                                                // temp1_input is often the package temp even without explicit label
                                                fallback_candidates.push(temp_celsius);
                                            }
                                        } else if temp_num == 1 {
                                            // No label file, but temp1_input might be package temp
                                            fallback_candidates.push(temp_celsius);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Return the highest package temperature found (most accurate)
        if let Some(&max_pkg_temp) = package_candidates.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
            return Some(max_pkg_temp);
        }
        
        // Fallback to highest temp1_input reading
        if let Some(&max_fallback_temp) = fallback_candidates.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
            return Some(max_fallback_temp);
        }
        
        None
    }

    fn update_per_core_temperatures(&mut self) {
        self.per_core_temperatures.clear();
        
        // Try to read per-core temperatures from hwmon
        if let Some(physical_temps) = self.read_hwmon_core_temperatures() {
            let logical_cores = self.per_core_usage.len();
            let physical_cores = physical_temps.len();
            
            if physical_cores > 0 && logical_cores > physical_cores {
                // Map physical core temps to all logical cores
                // For Intel 13900KF: 24 physical cores (8 P-cores + 16 E-cores) -> 32 logical cores
                // P-cores have hyperthreading (2 threads per core), E-cores don't
                // Mapping strategy: distribute available physical temps across all logical cores
                for logical_core in 0..logical_cores {
                    let physical_core = if logical_core < physical_cores {
                        // Direct mapping for cores within physical range
                        logical_core
                    } else {
                        // For additional logical cores, map back to physical cores cyclically
                        logical_core % physical_cores
                    };
                    
                    if physical_core < physical_cores {
                        self.per_core_temperatures.push(physical_temps[physical_core]);
                    }
                }
            } else if physical_cores > 0 {
                // Direct mapping or pad with the last available temperature
                for logical_core in 0..logical_cores {
                    if logical_core < physical_cores {
                        self.per_core_temperatures.push(physical_temps[logical_core]);
                    } else {
                        // Use the last available physical core temp for additional logical cores
                        self.per_core_temperatures.push(physical_temps[physical_cores - 1]);
                    }
                }
            }
        } else {
            // Fallback: try to estimate from thermal zones
            if let Some(temps) = self.read_thermal_zone_core_temperatures() {
                let logical_cores = self.per_core_usage.len();
                // Ensure we have temps for all logical cores
                if temps.len() < logical_cores {
                    self.per_core_temperatures = temps.clone();
                    // Pad remaining cores with the average temperature if we have some temps
                    if !temps.is_empty() {
                        let avg_temp = temps.iter().sum::<f32>() / temps.len() as f32;
                        while self.per_core_temperatures.len() < logical_cores {
                            self.per_core_temperatures.push(avg_temp);
                        }
                    }
                } else {
                    self.per_core_temperatures = temps;
                }
            }
        }
    }

    fn read_hwmon_core_temperatures(&self) -> Option<Vec<f32>> {
        use std::fs;
        
        let hwmon_base = "/sys/class/hwmon";
        
        if let Ok(entries) = fs::read_dir(hwmon_base) {
            for entry in entries.flatten() {
                let hwmon_path = entry.path();
                
                // Check if this is a CPU temperature sensor
                if let Ok(name) = fs::read_to_string(hwmon_path.join("name")) {
                    let name = name.trim().to_lowercase();
                    if name.contains("coretemp") || name.contains("k10temp") {
                        // Collect core temperatures in order
                        let mut temp_map = Vec::new();
                        
                        // Look for all temp*_input files with "Core" labels
                        // Check a wider range since core sensors might be at non-consecutive numbers
                        for i in 1..=64 { // Expanded range to cover more possible sensor locations
                            let temp_file = hwmon_path.join(format!("temp{}_input", i));
                            let label_file = hwmon_path.join(format!("temp{}_label", i));
                            
                            if let Ok(temp_str) = fs::read_to_string(&temp_file) {
                                if let Ok(temp_milli) = temp_str.trim().parse::<i32>() {
                                    let temp_celsius = temp_milli as f32 / 1000.0;
                                    
                                    // Check if this is a core temperature and get core number
                                    if let Ok(label_data) = fs::read_to_string(&label_file) {
                                        let label = label_data.trim().to_lowercase();
                                        if label.contains("core") && temp_celsius > 10.0 && temp_celsius < 150.0 {
                                            // Extract core number from label like "Core 0", "Core 8", etc.
                                            if let Some(core_num_str) = label.split_whitespace().nth(1) {
                                                if let Ok(core_num) = core_num_str.parse::<usize>() {
                                                    temp_map.push((core_num, temp_celsius));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        
                        if !temp_map.is_empty() {
                            // Sort by core number to ensure correct order
                            temp_map.sort_by_key(|&(core_num, _)| core_num);
                            // Map sparse core numbers to continuous array
                            let core_temps = temp_map.into_iter().map(|(_, temp)| temp).collect();
                            return Some(core_temps);
                        }
                    }
                }
            }
        }
        
        None
    }

    fn read_thermal_zone_core_temperatures(&self) -> Option<Vec<f32>> {
        use std::fs;
        
        let mut core_temps = Vec::new();
        
        // Try multiple thermal zones
        for i in 0..16 { // Check first 16 thermal zones
            let zone_path = format!("/sys/class/thermal/thermal_zone{}", i);
            let temp_file = format!("{}/temp", zone_path);
            let type_file = format!("{}/type", zone_path);
            
            if let Ok(temp_str) = fs::read_to_string(&temp_file) {
                if let Ok(temp_milli) = temp_str.trim().parse::<i32>() {
                    let temp_celsius = temp_milli as f32 / 1000.0;
                    
                    // Check if this zone relates to CPU cores
                    let is_cpu_related = if let Ok(type_data) = fs::read_to_string(&type_file) {
                        let zone_type = type_data.trim().to_lowercase();
                        zone_type.contains("cpu") || zone_type.contains("core") || 
                        zone_type.contains("x86_pkg_temp") || zone_type.contains("coretemp")
                    } else {
                        // If no type info, include reasonable temperatures
                        temp_celsius > 20.0 && temp_celsius < 100.0
                    };
                    
                    if is_cpu_related && temp_celsius > 10.0 && temp_celsius < 150.0 {
                        core_temps.push(temp_celsius);
                    }
                }
            }
        }
        
        if !core_temps.is_empty() {
            Some(core_temps)
        } else {
            None
        }
    }

    fn update_gpu_stats(&mut self) {
        use std::process::Command;

        // Enhanced nvidia-smi query for comprehensive GPU information
        let output = Command::new("nvidia-smi")
            .args([
                "--query-gpu=name,utilization.gpu,temperature.gpu,fan.speed,power.draw,memory.used,memory.total",
                "--format=csv,noheader,nounits",
            ])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                if let Ok(out_str) = String::from_utf8(output.stdout) {
                    if let Some(line) = out_str.lines().next() {
                        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                        if parts.len() >= 7 {
                            // Parse all GPU metrics
                            self.gpu_name = if !parts[0].is_empty() && parts[0] != "[Not Supported]" {
                                Some(parts[0].to_string())
                            } else {
                                None
                            };
                            
                            self.gpu_usage = parts[1].parse::<f32>().ok();
                            self.gpu_temperature = parts[2].parse::<f32>().ok();
                            
                            // Fan speed (percentage)
                            self.gpu_fan_speed = if parts[3] != "[Not Supported]" {
                                parts[3].parse::<f32>().ok()
                            } else {
                                None
                            };
                            
                            // Power draw (watts)
                            self.gpu_power_draw = if parts[4] != "[Not Supported]" {
                                parts[4].parse::<f32>().ok()
                            } else {
                                None
                            };
                            
                            // Memory usage (convert to MB)
                            self.gpu_memory_used = if parts[5] != "[Not Supported]" {
                                parts[5].parse::<f32>().ok()
                            } else {
                                None
                            };
                            
                            self.gpu_memory_total = if parts[6] != "[Not Supported]" {
                                parts[6].parse::<f32>().ok()
                            } else {
                                None
                            };
                            
                            return;
                        }
                    }
                }
            }
        }

        // Fallback: try basic query if comprehensive query fails
        let fallback_output = Command::new("nvidia-smi")
            .args([
                "--query-gpu=utilization.gpu,temperature.gpu",
                "--format=csv,noheader,nounits",
            ])
            .output();

        if let Ok(output) = fallback_output {
            if output.status.success() {
                if let Ok(out_str) = String::from_utf8(output.stdout) {
                    if let Some(line) = out_str.lines().next() {
                        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                        if parts.len() >= 2 {
                            self.gpu_usage = parts[0].parse::<f32>().ok();
                            self.gpu_temperature = parts[1].parse::<f32>().ok();
                            
                            // Clear advanced metrics since they weren't available
                            self.gpu_fan_speed = None;
                            self.gpu_power_draw = None;
                            self.gpu_memory_used = None;
                            self.gpu_memory_total = None;
                            self.gpu_name = None;
                            return;
                        }
                    }
                }
            }
        }

        // Clear all GPU data if nvidia-smi is not available or failed
        self.gpu_usage = None;
        self.gpu_temperature = None;
        self.gpu_fan_speed = None;
        self.gpu_power_draw = None;
        self.gpu_memory_used = None;
        self.gpu_memory_total = None;
        self.gpu_name = None;
    }

    fn update_gpu_history(&mut self) {
        // Update GPU usage history
        let gpu_usage = self.gpu_usage.unwrap_or(0.0);
        if self.gpu_usage_history.len() >= self.max_history {
            self.gpu_usage_history.pop_front();
        }
        self.gpu_usage_history.push_back(gpu_usage);

        // Update GPU memory percentage history
        let gpu_memory_percent = self.gpu_memory_usage_percent().unwrap_or(0.0);
        if self.gpu_memory_percent_history.len() >= self.max_history {
            self.gpu_memory_percent_history.pop_front();
        }
        self.gpu_memory_percent_history.push_back(gpu_memory_percent);
    }
}