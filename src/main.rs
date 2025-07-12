use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{
    io,
    process::Command,
    thread,
    time::{Duration, Instant},
};
use sysinfo::{Disks, System};

mod metrics;
mod ui;

use metrics::SystemMetrics;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 1)]
    interval: u64,
    
    #[arg(short, long)]
    simple: bool,
    
    #[arg(long, default_value_t = 60)]
    history: usize,
}

struct App {
    system: System,
    metrics: SystemMetrics,
    should_quit: bool,
    last_update: Instant,
    update_interval: Duration,
    simple_mode: bool,
    current_tab: usize,
    journal_logs: Vec<String>,
    journal_scroll: usize,
    processes: Vec<ProcessInfo>,
    process_scroll: usize,
    last_process_refresh: Instant,
    last_journal_refresh: Instant,
    process_refresh_interval: Duration,
    journal_refresh_interval: Duration,
    process_sort_mode: ProcessSortMode,
}

#[derive(Clone, Copy, PartialEq)]
enum ProcessSortMode {
    Cpu,
    Memory,
}

#[derive(Clone)]
struct ProcessInfo {
    pid: u32,
    name: String,
    cpu_usage: f32,
    memory_usage: u64,
    user: String,
}

impl App {
    fn new(interval: u64, history_size: usize, simple_mode: bool) -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        Self {
            system,
            metrics: SystemMetrics::new(history_size),
            should_quit: false,
            last_update: Instant::now(),
            update_interval: Duration::from_secs(interval),
            simple_mode,
            current_tab: 0,
            journal_logs: Vec::new(),
            journal_scroll: 0,
            processes: Vec::new(),
            process_scroll: 0,
            last_process_refresh: Instant::now(),
            last_journal_refresh: Instant::now(),
            process_refresh_interval: Duration::from_secs(2), // Refresh processes every 2 seconds
            journal_refresh_interval: Duration::from_secs(5), // Refresh logs every 5 seconds
            process_sort_mode: ProcessSortMode::Cpu, // Default to CPU sorting
        }
    }

    fn update(&mut self) {
        if self.last_update.elapsed() >= self.update_interval {
            // Only refresh essential system metrics for main display
            self.system.refresh_cpu_specifics(sysinfo::CpuRefreshKind::everything());
            self.system.refresh_memory();
            // Skip disk and network refresh here - they're handled separately by metrics
            
            self.metrics.update(&self.system);
            self.last_update = Instant::now();
        }
        
        // Update processes and logs based on their own intervals and current tab
        if self.current_tab == 1 && self.last_process_refresh.elapsed() >= self.process_refresh_interval {
            self.refresh_processes_cached();
        }
        
        if self.current_tab == 2 && self.last_journal_refresh.elapsed() >= self.journal_refresh_interval {
            self.refresh_journal_logs_cached();
        }
    }

    fn handle_input(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                    KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => self.should_quit = true,
                    KeyCode::Tab => {
                        self.current_tab = (self.current_tab + 1) % 3;
                        // Trigger immediate refresh for new tab if data is stale
                        match self.current_tab {
                            1 => {
                                if self.processes.is_empty() || self.last_process_refresh.elapsed() >= self.process_refresh_interval {
                                    self.refresh_processes_cached();
                                }
                            }
                            2 => {
                                if self.journal_logs.is_empty() || self.last_journal_refresh.elapsed() >= self.journal_refresh_interval {
                                    self.refresh_journal_logs_cached();
                                }
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Up => {
                        match self.current_tab {
                            1 => {
                                if !self.processes.is_empty() && self.process_scroll > 0 {
                                    self.process_scroll -= 1;
                                }
                            }
                            2 => {
                                if !self.journal_logs.is_empty() && self.journal_scroll > 0 {
                                    self.journal_scroll -= 1;
                                }
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Down => {
                        match self.current_tab {
                            1 => {
                                if !self.processes.is_empty() && self.process_scroll < self.processes.len().saturating_sub(1) {
                                    self.process_scroll += 1;
                                }
                            }
                            2 => {
                                if !self.journal_logs.is_empty() && self.journal_scroll < self.journal_logs.len().saturating_sub(1) {
                                    self.journal_scroll += 1;
                                }
                            }
                            _ => {}
                        }
                    }
                    KeyCode::PageUp => {
                        match self.current_tab {
                            1 => {
                                if !self.processes.is_empty() {
                                    self.process_scroll = self.process_scroll.saturating_sub(10);
                                }
                            }
                            2 => {
                                if !self.journal_logs.is_empty() {
                                    self.journal_scroll = self.journal_scroll.saturating_sub(10);
                                }
                            }
                            _ => {}
                        }
                    }
                    KeyCode::PageDown => {
                        match self.current_tab {
                            1 => {
                                if !self.processes.is_empty() {
                                    self.process_scroll = (self.process_scroll + 10).min(self.processes.len().saturating_sub(1));
                                }
                            }
                            2 => {
                                if !self.journal_logs.is_empty() {
                                    self.journal_scroll = (self.journal_scroll + 10).min(self.journal_logs.len().saturating_sub(1));
                                }
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Char('c') => {
                        if self.current_tab == 1 {
                            self.process_sort_mode = ProcessSortMode::Cpu;
                            self.refresh_processes_cached();
                        }
                    }
                    KeyCode::Char('m') => {
                        if self.current_tab == 1 {
                            self.process_sort_mode = ProcessSortMode::Memory;
                            self.refresh_processes_cached();
                        }
                    }
                    KeyCode::Char('k') => {
                        if self.current_tab == 1 && !self.processes.is_empty() {
                            let selected_process = &self.processes[self.process_scroll];
                            self.kill_process(selected_process.pid);
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn refresh_journal_logs_cached(&mut self) {
        // Non-blocking journal refresh with timeout
        if let Ok(output) = Command::new("timeout")
            .arg("1s") // 1 second timeout
            .arg("journalctl")
            .arg("-n")
            .arg("100")
            .arg("--no-pager")
            .arg("-o")
            .arg("short")
            .arg("-r")
            .output()
        {
            if output.status.success() {
                if let Ok(logs) = String::from_utf8(output.stdout) {
                    let new_logs: Vec<String> = logs.lines().map(|s| s.to_string()).collect();
                    if !new_logs.is_empty() {
                        self.journal_logs = new_logs;
                        // Only reset scroll if we're at the top
                        if self.journal_scroll == 0 {
                            self.journal_scroll = 0;
                        }
                    }
                }
            }
        }
        self.last_journal_refresh = Instant::now();
    }

    fn refresh_processes_cached(&mut self) {
        // Optimized process refresh - only refresh processes, not all system info
        self.system.refresh_processes(sysinfo::ProcessesToUpdate::All, false); // false = don't refresh everything
        
        let mut processes: Vec<ProcessInfo> = self.system.processes()
            .values()
            .filter(|process| {
                // More efficient filtering
                !process.name().is_empty() && process.memory() > 1024 // > 1KB to filter out tiny processes
            })
            .map(|process| ProcessInfo {
                pid: process.pid().as_u32(),
                name: process.name().to_string_lossy().to_string(),
                cpu_usage: process.cpu_usage(),
                memory_usage: process.memory(),
                user: process.user_id().map(|uid| uid.to_string()).unwrap_or_else(|| "unknown".to_string()),
            })
            .collect();
        
        // Sort based on current sort mode
        match self.process_sort_mode {
            ProcessSortMode::Cpu => {
                processes.sort_by(|a, b| {
                    b.cpu_usage.partial_cmp(&a.cpu_usage)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| b.memory_usage.cmp(&a.memory_usage))
                });
            }
            ProcessSortMode::Memory => {
                processes.sort_by(|a, b| {
                    b.memory_usage.cmp(&a.memory_usage)
                        .then_with(|| b.cpu_usage.partial_cmp(&a.cpu_usage)
                            .unwrap_or(std::cmp::Ordering::Equal))
                });
            }
        }
        
        // Limit to top 500 processes for performance
        processes.truncate(500);
        
        self.processes = processes;
        self.last_process_refresh = Instant::now();
        
        // Ensure scroll position is within bounds
        if self.process_scroll >= self.processes.len() {
            self.process_scroll = self.processes.len().saturating_sub(1);
        }
    }

    fn kill_process(&mut self, pid: u32) {
        // Use kill command to send SIGKILL to the process
        let result = Command::new("kill")
            .arg("-9") // SIGKILL
            .arg(pid.to_string())
            .output();
        
        match result {
            Ok(output) => {
                if output.status.success() {
                    // Process killed successfully, refresh the process list
                    self.refresh_processes_cached();
                }
                // Note: We don't show success/error messages to keep the UI clean
                // System administrators expect immediate feedback through the process list update
            }
            Err(_) => {
                // Kill command failed (e.g., insufficient permissions, process doesn't exist)
                // Refresh the list anyway to show current state
                self.refresh_processes_cached();
            }
        }
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    loop {
        app.update();
        
        terminal.draw(|f| ui::draw(f, &app))?;
        
        app.handle_input()?;
        
        if app.should_quit {
            break;
        }
        
        thread::sleep(Duration::from_millis(50));
    }
    
    Ok(())
}

fn run_simple_mode(mut app: App) -> Result<()> {
    loop {
        app.update();
        
        // Clear screen and move cursor to top
        print!("\x1B[2J\x1B[H");
        
        // Print current time and metrics in simple text format
        let now = chrono::Local::now();
        let clock_text = format!("{}", now.format("%H:%M:%S"));
        let header_width = 30;
        let padding = (header_width - clock_text.len()) / 2;
        println!("{:padding$}{}", "", clock_text, padding = padding);
        println!("==============================");
        
        // CPU info
        println!("\nCPU:");
        println!("  Overall Usage: {:.1}%", app.metrics.cpu_usage());
        if let Some(cpu_info) = app.system.cpus().first() {
            println!("  Brand: {}", cpu_info.brand());
            println!("  Frequency: {:.0} MHz", cpu_info.frequency());
            println!("  Cores: {}", app.system.cpus().len());
        }
        
        // Per-core CPU usage
        let per_core = app.metrics.per_core_usage();
        if !per_core.is_empty() {
            println!("  Per-core Usage:");
            let cores_per_row = 4;
            for (i, &usage) in per_core.iter().enumerate() {
                if i % cores_per_row == 0 {
                    print!("    ");
                }
                print!("C{:02}:{:5.1}%", i, usage);
                if i % cores_per_row == cores_per_row - 1 || i == per_core.len() - 1 {
                    println!();
                } else {
                    print!("  ");
                }
            }
        }
        
        // Memory info
        println!("\nMemory:");
        let total_mem = app.system.total_memory() as f64 / 1024.0 / 1024.0;
        let used_mem = app.system.used_memory() as f64 / 1024.0 / 1024.0;
        let usage_percent = (used_mem / total_mem) * 100.0;
        println!("  Usage: {:.1}%", usage_percent);
        println!("  Used: {:.1} MB", used_mem);
        println!("  Total: {:.1} MB", total_mem);
        
        // Disk info
        println!("\nDisk:");
        let disks = Disks::new_with_refreshed_list();
        for disk in &disks {
            if disk.mount_point().to_str() == Some("/") {
                let total = disk.total_space() as f64 / 1024.0 / 1024.0 / 1024.0;
                let available = disk.available_space() as f64 / 1024.0 / 1024.0 / 1024.0;
                let used = total - available;
                let usage_percent = (used / total) * 100.0;
                println!("  Usage: {:.1}%", usage_percent);
                println!("  Used: {:.1} GB", used);
                println!("  Total: {:.1} GB", total);
                break;
            }
        }
        
        
        // Network info
        println!("\nNetwork:");
        let download_rate = app.metrics.network_download_rate();
        let upload_rate = app.metrics.network_upload_rate();
        let (total_rx, total_tx) = app.metrics.total_network_bytes();
        println!("  Download: {:.1} Kbps", download_rate);
        println!("  Upload: {:.1} Kbps", upload_rate);
        println!("  Total Down: {:.1} MB", total_rx as f64 / 1024.0 / 1024.0);
        println!("  Total Up: {:.1} MB", total_tx as f64 / 1024.0 / 1024.0);
        
        // Temperature info
        println!("\nTemperature:");
        if let Some(temp) = app.metrics.cpu_temperature() {
            println!("  CPU Package: {:.1}°C", temp);
        } else {
            println!("  CPU Package: N/A");
        }
        
        // Per-core temperatures
        let per_core_temps = app.metrics.per_core_temperatures();
        if !per_core_temps.is_empty() {
            let logical_cores = app.metrics.per_core_usage().len();
            let temp_cores = per_core_temps.len();
            
            if temp_cores == logical_cores {
                println!("  Per-core Temps:");
            } else if temp_cores < logical_cores {
                println!("  Per-core Temps (physical cores mapped to logical):");
            } else {
                println!("  Core Temps:");
            }
            
            let cores_per_row = 4;
            for (i, &temp) in per_core_temps.iter().enumerate() {
                if i % cores_per_row == 0 {
                    print!("    ");
                }
                print!("C{:02}:{:5.1}°C", i, temp);
                if i % cores_per_row == cores_per_row - 1 || i == per_core_temps.len() - 1 {
                    println!();
                } else {
                    print!("  ");
                }
            }
        }

        // GPU info
        println!("\nGPU:");
        if let Some(usage) = app.metrics.gpu_usage() {
            println!("  Usage: {:.1}%", usage);
        } else {
            println!("  Usage: N/A");
        }
        if let Some(temp) = app.metrics.gpu_temperature() {
            println!("  Temp: {:.1}°C", temp);
        } else {
            println!("  Temp: N/A");
        }

        if let Some(fan) = app.metrics.gpu_fan_speed() {
            println!("  Fan: {:.0}%", fan);
        } else {
            println!("  Fan: N/A");
        }

        if let Some(power) = app.metrics.gpu_power_draw() {
            println!("  Power: {:.1} W", power);
        } else {
            println!("  Power: N/A");
        }

        match (app.metrics.gpu_memory_used(), app.metrics.gpu_memory_total()) {
            (Some(used), Some(total)) => {
                let pct = used as f32 / total as f32 * 100.0;
                println!("  VRAM: {} / {} MiB ({:.1}%)", used, total, pct);
            }
            (Some(used), None) => println!("  VRAM Used: {} MiB", used),
            _ => println!("  VRAM: N/A"),
        }
        
        // Handle Ctrl+C
        if let Ok(true) = event::poll(Duration::from_millis(100)) {
            if let Ok(Event::Key(key)) = event::read() {
                if key.code == KeyCode::Char('c') && key.modifiers.contains(event::KeyModifiers::CONTROL) {
                    break;
                }
            }
        }
        
        thread::sleep(app.update_interval);
    }
    
    Ok(())
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    format!("{:.1} {}", size, UNITS[unit_index])
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    let app = App::new(args.interval, args.history, args.simple);
    
    if args.simple {
        run_simple_mode(app)?;
    } else {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        let res = run_app(&mut terminal, app);
        
        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        
        if let Err(err) = res {
            println!("{:?}", err);
        }
    }
    
    Ok(())
}
