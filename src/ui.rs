use crate::App;
use chrono::Local;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::Line,
    widgets::{Axis, Block, Borders, Chart, Dataset, Gauge, List, ListItem, ListState, Paragraph, Tabs, Table, Row, Cell, TableState},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(3),  // Tabs
            Constraint::Min(0),     // Main content
        ])
        .split(f.area());

    // Clock
    let now = Local::now();
    let clock_text = format!("{}", now.format("%H:%M:%S"));
    let clock = Paragraph::new(clock_text)
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(clock, chunks[0]);

    // Tabs with navigation hint
    let tab_titles = vec!["System Monitor", "Processes", "Journal Logs"];
    let tabs = Tabs::new(tab_titles)
        .block(Block::default()
            .title("Press [Tab] to switch pages")
            .borders(Borders::ALL))
        .select(app.current_tab)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow));
    f.render_widget(tabs, chunks[1]);

    // Main content based on selected tab
    match app.current_tab {
        0 => draw_system_monitor(f, app, chunks[2]),
        1 => draw_processes(f, app, chunks[2]),
        2 => draw_journal_logs(f, app, chunks[2]),
        _ => {}
    }
}

fn draw_system_monitor(f: &mut Frame, app: &App, area: Rect) {
    // Main content in 5 panels layout (2x2 grid plus GPU panel)
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[0]);

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(main_chunks[1]);

    // CPU usage (top-left)
    draw_cpu_widget(f, app, top_chunks[0]);
    
    // Memory usage (top-right)
    draw_memory_widget(f, app, top_chunks[1]);
    
    // Disk usage (bottom-left)
    draw_disk_widget(f, app, bottom_chunks[0]);

    // Network usage (bottom-middle)
    draw_network_widget(f, app, bottom_chunks[1]);

    // GPU usage (bottom-right)
    draw_gpu_widget(f, app, bottom_chunks[2]);
}

fn draw_journal_logs(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Instructions
            Constraint::Min(0),     // Log content
        ])
        .split(area);

    // Instructions
    let instructions = Paragraph::new("Use ↑/↓ to scroll, PgUp/PgDn for fast scroll, Tab to switch tabs")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, chunks[0]);

    // Log content
    let log_items: Vec<ListItem> = app.journal_logs
        .iter()
        .map(|log| ListItem::new(log.as_str()))
        .collect();

    let logs_list = List::new(log_items)
        .block(Block::default()
            .title("📋 System Journal Logs (Latest 100 - Newest First)")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White));
    
    let mut list_state = ListState::default();
    list_state.select(Some(app.journal_scroll));
    f.render_stateful_widget(logs_list, chunks[1], &mut list_state);
}

fn draw_processes(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Instructions
            Constraint::Min(0),     // Process table
        ])
        .split(area);

    // Instructions with sort and kill controls
    let instructions = Paragraph::new("Use ↑/↓ to scroll, PgUp/PgDn for fast scroll, Tab to switch tabs • [C] sort by CPU • [M] sort by Memory • [K] kill process")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, chunks[0]);

    // Process table
    let header = Row::new(vec![
        Cell::from("PID"),
        Cell::from("Name"),
        Cell::from("CPU%"),
        Cell::from("Memory"),
        Cell::from("User"),
    ])
    .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = app.processes
        .iter()
        .map(|process| {
            let memory_mb = process.memory_usage as f64 / 1024.0 / 1024.0;
            
            Row::new(vec![
                Cell::from(process.pid.to_string()),
                Cell::from(process.name.clone()),
                Cell::from(format!("{:.1}", process.cpu_usage)),
                Cell::from(format!("{:.1}MB", memory_mb)),
                Cell::from(process.user.clone()),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(8),   // PID
        Constraint::Min(20),     // Name
        Constraint::Length(8),   // CPU%
        Constraint::Length(12),  // Memory
        Constraint::Length(15),  // User
    ];

    let sort_indicator = match app.process_sort_mode {
        crate::ProcessSortMode::Cpu => "CPU",
        crate::ProcessSortMode::Memory => "Memory",
    };
    
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default()
            .title(format!("🔄 Running Processes ({} total, sorted by {}) • Selected: [K] to kill", app.processes.len(), sort_indicator))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)))
        .row_highlight_style(Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD))
        .column_spacing(1);

    let mut table_state = TableState::default();
    if !app.processes.is_empty() {
        // Ensure scroll position is always valid
        let scroll_pos = app.process_scroll.min(app.processes.len().saturating_sub(1));
        table_state.select(Some(scroll_pos));
    }
    f.render_stateful_widget(table, chunks[1], &mut table_state);
}

fn draw_cpu_widget(f: &mut Frame, app: &App, area: Rect) {
    let cpu_usage = app.metrics.cpu_usage();
    
    // Split into gauge and info areas (no chart)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Gauge
            Constraint::Min(0),     // Info (expanded to fill space)
        ])
        .split(area);

    // CPU Gauge
    let cpu_color = if cpu_usage < 50.0 {
        Color::Green
    } else if cpu_usage < 80.0 {
        Color::Yellow
    } else {
        Color::Red
    };
    
    let gauge = Gauge::default()
        .block(Block::default()
            .title("🖥️ CPU Usage")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)))
        .gauge_style(Style::default().fg(cpu_color))
        .percent(cpu_usage as u16)
        .label(format!("{:.1}%", cpu_usage));
    f.render_widget(gauge, chunks[0]);

    // CPU Info with per-core data
    let mut cpu_info = if let Some(cpu) = app.system.cpus().first() {
        vec![
            Line::from(format!("┌─ CPU Info ─────────────────")),
            Line::from(format!("│ Brand: {}", cpu.brand())),
            Line::from(format!("│ Cores: {}    Freq: {:.0} MHz", app.system.cpus().len(), cpu.frequency())),
            Line::from("└───────────────────────────"),
            Line::from(""),  // Empty line for spacing
        ]
    } else {
        vec![Line::from("CPU info unavailable")]
    };

    // Add per-core usage and temperature info side by side
    let per_core = app.metrics.per_core_usage();
    let per_core_temps = app.metrics.per_core_temperatures();
    
    if !per_core.is_empty() {
        if per_core.len() <= 8 {
            // For systems with 8 cores or fewer, show detailed per-core info
            cpu_info.push(Line::from("┌─ Core Usage & Temperature ─"));
            
            for (i, &usage) in per_core.iter().enumerate() {
                // Get temperature for this core if available
                let temp_str = if i < per_core_temps.len() {
                    format!("{:5.1}°C", per_core_temps[i])
                } else {
                    "  N/A ".to_string()
                };
                
                let usage_bar = if usage < 25.0 {
                    "▁"
                } else if usage < 50.0 {
                    "▃"
                } else if usage < 75.0 {
                    "▅"
                } else {
                    "▇"
                };
                
                cpu_info.push(Line::from(format!("│ Core {:2}: {:5.1}% {} │ {}", i, usage, usage_bar, temp_str)));
            }
            cpu_info.push(Line::from("└─────────────────────────────"));
        } else {
            // For systems with many cores, show summary stats first
            let avg_usage = per_core.iter().sum::<f32>() / per_core.len() as f32;
            let max_usage = per_core.iter().fold(0.0f32, |a, &b| a.max(b));
            let min_usage = per_core.iter().fold(100.0f32, |a, &b| a.min(b));
            
            cpu_info.push(Line::from("┌─ Usage Summary ─────────────"));
            cpu_info.push(Line::from(format!("│ Avg: {:5.1}%  Max: {:5.1}%", avg_usage, max_usage)));
            cpu_info.push(Line::from(format!("│ Min: {:5.1}%  Cores: {:3}", min_usage, per_core.len())));
            
            // Show temperature stats if available
            if !per_core_temps.is_empty() {
                let avg_temp = per_core_temps.iter().sum::<f32>() / per_core_temps.len() as f32;
                let max_temp = per_core_temps.iter().fold(0.0f32, |a, &b| a.max(b));
                let min_temp = per_core_temps.iter().fold(200.0f32, |a, &b| a.min(b));
                cpu_info.push(Line::from(format!("│ Temp: {:.1}°C  Max: {:.1}°C", avg_temp, max_temp)));
            }
            cpu_info.push(Line::from("└─────────────────────────────"));
            cpu_info.push(Line::from(""));  // Empty line for spacing
            
            // Show all cores in a more compact but readable format
            cpu_info.push(Line::from("┌─ Individual Cores ─────────"));
            
            let cores_per_line = 4;
            for chunk in per_core.chunks(cores_per_line) {
                let mut line = String::from("│ ");
                for (local_i, &usage) in chunk.iter().enumerate() {
                    let core_idx = (chunk.as_ptr() as usize - per_core.as_ptr() as usize) / std::mem::size_of::<f32>() + local_i;
                    
                    // Get temperature for this core if available
                    let temp_str = if core_idx < per_core_temps.len() {
                        format!("{:.0}°", per_core_temps[core_idx])
                    } else {
                        "N/A".to_string()
                    };
                    
                    line += &format!("C{:2}:{:4.0}%/{:>3} ", core_idx, usage, temp_str);
                }
                cpu_info.push(Line::from(line));
            }
            cpu_info.push(Line::from("└─────────────────────────────"));
        }
    }


    let info_paragraph = Paragraph::new(cpu_info)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    f.render_widget(info_paragraph, chunks[1]);
}

fn draw_memory_widget(f: &mut Frame, app: &App, area: Rect) {
    let memory_usage = app.metrics.memory_usage();
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Gauge
            Constraint::Length(4),  // Info
            Constraint::Min(0),     // Chart
        ])
        .split(area);

    // Memory Gauge
    let memory_color = if memory_usage < 60.0 {
        Color::Cyan
    } else if memory_usage < 85.0 {
        Color::Yellow
    } else {
        Color::Red
    };
    
    let gauge = Gauge::default()
        .block(Block::default()
            .title("💾 Memory Usage")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue)))
        .gauge_style(Style::default().fg(memory_color))
        .percent(memory_usage as u16)
        .label(format!("{:.1}%", memory_usage));
    f.render_widget(gauge, chunks[0]);

    // Memory Info
    let total_mem = app.system.total_memory() as f64 / 1024.0 / 1024.0;
    let used_mem = app.system.used_memory() as f64 / 1024.0 / 1024.0;
    let free_mem = total_mem - used_mem;

    let memory_info = vec![
        Line::from(format!("Total: {:.1} MB", total_mem)),
        Line::from(format!("Used: {:.1} MB", used_mem)),
        Line::from(format!("Free: {:.1} MB", free_mem)),
    ];

    let info_paragraph = Paragraph::new(memory_info)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    f.render_widget(info_paragraph, chunks[1]);

    // Memory Chart
    let memory_data: Vec<(f64, f64)> = app.metrics.memory_history()
        .iter()
        .enumerate()
        .map(|(i, &value)| (i as f64, value as f64))
        .collect();

    if !memory_data.is_empty() {
        let datasets = vec![Dataset::default()
            .name("Memory Usage")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Cyan))
            .data(&memory_data)];

        let chart = Chart::new(datasets)
            .block(Block::default()
                .title("Memory Usage History")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)))
            .x_axis(
                Axis::default()
                    .title("Time")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.0, app.metrics.memory_history().len() as f64])
                    .labels(vec!["Past", "Now"]),
            )
            .y_axis(
                Axis::default()
                    .title("Usage %")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.0, 100.0])
                    .labels(vec!["0%", "25%", "50%", "75%", "100%"]),
            );
        f.render_widget(chart, chunks[2]);
    }
}

fn draw_disk_widget(f: &mut Frame, app: &App, area: Rect) {
    let disk_usage = app.metrics.disk_usage();
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Gauge
            Constraint::Min(0),     // Info (expanded to fill space)
        ])
        .split(area);

    // Disk Gauge
    let disk_color = if disk_usage < 70.0 {
        Color::Green
    } else if disk_usage < 90.0 {
        Color::Yellow
    } else {
        Color::Red
    };
    
    let gauge = Gauge::default()
        .block(Block::default()
            .title("💽 Disk Usage")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)))
        .gauge_style(Style::default().fg(disk_color))
        .percent(disk_usage as u16)
        .label(format!("{:.1}%", disk_usage));
    f.render_widget(gauge, chunks[0]);

    // Disk Info
    let mut disk_info = vec![Line::from("Root filesystem:")];
    let disks = sysinfo::Disks::new_with_refreshed_list();
    for disk in &disks {
        if disk.mount_point().to_str() == Some("/") {
            let total = disk.total_space() as f64 / 1024.0 / 1024.0 / 1024.0;
            let available = disk.available_space() as f64 / 1024.0 / 1024.0 / 1024.0;
            let used = total - available;
            
            disk_info.push(Line::from(format!("Total: {:.1} GB", total)));
            disk_info.push(Line::from(format!("Used: {:.1} GB", used)));
            break;
        }
    }

    let info_paragraph = Paragraph::new(disk_info)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    f.render_widget(info_paragraph, chunks[1]);
}

fn draw_network_widget(f: &mut Frame, app: &App, area: Rect) {
    let download_rate = app.metrics.network_download_rate();
    let upload_rate = app.metrics.network_upload_rate();
    let (total_rx, total_tx) = app.metrics.total_network_bytes();
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Download gauge
            Constraint::Length(3),  // Upload gauge
            Constraint::Min(0),     // Info section
        ])
        .split(area);

    // Calculate percentage for gauges (scale to a reasonable max speed)
    let max_speed_kbps = 10000.0; // 10 Mbps max for gauge scale
    let download_percent = ((download_rate / max_speed_kbps) * 100.0).min(100.0) as u16;
    let upload_percent = ((upload_rate / max_speed_kbps) * 100.0).min(100.0) as u16;

    // Download Gauge
    let download_color = if download_rate < 1000.0 {
        Color::Green
    } else if download_rate < 5000.0 {
        Color::Yellow
    } else {
        Color::Red
    };
    
    let download_gauge = Gauge::default()
        .block(Block::default()
            .title("⬇️ Download")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)))
        .gauge_style(Style::default().fg(download_color))
        .percent(download_percent)
        .label(format!("{:.1} Kbps", download_rate));
    f.render_widget(download_gauge, chunks[0]);

    // Upload Gauge
    let upload_color = if upload_rate < 1000.0 {
        Color::Green
    } else if upload_rate < 5000.0 {
        Color::Yellow
    } else {
        Color::Red
    };
    
    let upload_gauge = Gauge::default()
        .block(Block::default()
            .title("⬆️ Upload")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red)))
        .gauge_style(Style::default().fg(upload_color))
        .percent(upload_percent)
        .label(format!("{:.1} Kbps", upload_rate));
    f.render_widget(upload_gauge, chunks[1]);

    // Network Info
    let network_info = vec![
        Line::from(format!("Total Down: {:.1} MB", total_rx as f64 / 1024.0 / 1024.0)),
        Line::from(format!("Total Up: {:.1} MB", total_tx as f64 / 1024.0 / 1024.0)),
        Line::from(format!("Max Scale: {:.0} Mbps", max_speed_kbps / 1000.0)),
    ];

    let info_paragraph = Paragraph::new(network_info)
        .block(Block::default()
            .title("🌐 Network Stats")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta)))
        .style(Style::default().fg(Color::White));
    f.render_widget(info_paragraph, chunks[2]);
}

fn draw_gpu_widget(f: &mut Frame, app: &App, area: Rect) {
    let usage = app.metrics.gpu_usage().unwrap_or(0.0);
    let temp = app.metrics.gpu_temperature();
    let fan_speed = app.metrics.gpu_fan_speed();
    let power_draw = app.metrics.gpu_power_draw();
    let memory_used = app.metrics.gpu_memory_used();
    let memory_total = app.metrics.gpu_memory_total();
    let memory_percent = app.metrics.gpu_memory_usage_percent();
    let gpu_name = app.metrics.gpu_name();

    // Create a more detailed layout for comprehensive GPU info
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // GPU Usage gauge
            Constraint::Length(3),  // VRAM Usage gauge
            Constraint::Min(0),     // Detailed info section
        ])
        .split(area);

    // GPU Usage gauge with dynamic coloring
    let usage_color = if usage < 50.0 {
        Color::Green
    } else if usage < 80.0 {
        Color::Yellow
    } else {
        Color::Red
    };

    // Create title with GPU name if available
    let gpu_title = if let Some(name) = gpu_name {
        format!("🎮 GPU Usage - {}", name)
    } else {
        "🎮 GPU Usage - NVIDIA GPU".to_string()
    };

    let usage_gauge = Gauge::default()
        .block(Block::default()
            .title(gpu_title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta)))
        .gauge_style(Style::default().fg(usage_color))
        .percent(usage as u16)
        .label(format!("{:.1}%", usage));
    f.render_widget(usage_gauge, chunks[0]);

    // VRAM Usage gauge
    if let Some(mem_percent) = memory_percent {
        let memory_color = if mem_percent < 60.0 {
            Color::Cyan
        } else if mem_percent < 85.0 {
            Color::Yellow
        } else {
            Color::Red
        };

        let vram_label = if let (Some(used), Some(total)) = (memory_used, memory_total) {
            format!("{:.0}MB / {:.0}MB ({:.1}%)", used, total, mem_percent)
        } else {
            format!("{:.1}%", mem_percent)
        };

        let memory_gauge = Gauge::default()
            .block(Block::default()
                .title("💾 VRAM Usage")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)))
            .gauge_style(Style::default().fg(memory_color))
            .percent(mem_percent as u16)
            .label(vram_label);
        f.render_widget(memory_gauge, chunks[1]);
    } else {
        // Show placeholder if VRAM info not available
        let memory_gauge = Gauge::default()
            .block(Block::default()
                .title("💾 VRAM Usage")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)))
            .gauge_style(Style::default().fg(Color::DarkGray))
            .percent(0)
            .label("N/A");
        f.render_widget(memory_gauge, chunks[1]);
    }

    // Comprehensive GPU information panel
    let mut gpu_info = vec![
        Line::from("┌─ GPU Metrics ───────────────"),
    ];

    // Temperature with color coding
    if let Some(t) = temp {
        let temp_icon = if t < 60.0 {
            "🌡️"
        } else if t < 80.0 {
            "🔥"
        } else {
            "🚨"
        };
        gpu_info.push(Line::from(format!("│ {} Temperature: {:.1}°C", temp_icon, t)));
    } else {
        gpu_info.push(Line::from("│ 🌡️ Temperature: N/A"));
    }

    // Fan speed with visual indicator
    if let Some(fan) = fan_speed {
        let fan_icon = if fan < 30.0 {
            "💨"
        } else if fan < 70.0 {
            "🌪️"
        } else {
            "🚁"
        };
        gpu_info.push(Line::from(format!("│ {} Fan Speed: {:.0}%", fan_icon, fan)));
    } else {
        gpu_info.push(Line::from("│ 💨 Fan Speed: N/A"));
    }

    // Power draw with efficiency indicator
    if let Some(power) = power_draw {
        let power_icon = if power < 150.0 {
            "⚡"
        } else if power < 250.0 {
            "🔌"
        } else {
            "🔋"
        };
        gpu_info.push(Line::from(format!("│ {} Power Draw: {:.1}W", power_icon, power)));
    } else {
        gpu_info.push(Line::from("│ ⚡ Power Draw: N/A"));
    }

    // Memory details if available
    if let (Some(used), Some(total)) = (memory_used, memory_total) {
        let free_memory = total - used;
        gpu_info.push(Line::from("├─ VRAM Details ─────────────"));
        gpu_info.push(Line::from(format!("│ 📊 Used: {:.0} MB", used)));
        gpu_info.push(Line::from(format!("│ 📋 Free: {:.0} MB", free_memory)));
        gpu_info.push(Line::from(format!("│ 💽 Total: {:.0} MB", total)));
    }

    gpu_info.push(Line::from("└─────────────────────────────"));

    // Add performance status indicator
    let performance_status = if usage > 80.0 {
        "🔴 High Load"
    } else if usage > 50.0 {
        "🟡 Medium Load"
    } else if usage > 10.0 {
        "🟢 Light Load"
    } else {
        "💤 Idle"
    };
    
    gpu_info.push(Line::from(""));
    gpu_info.push(Line::from(format!("Status: {}", performance_status)));

    // Thermal status if temperature available
    if let Some(t) = temp {
        let thermal_status = if t > 80.0 {
            "🚨 Hot"
        } else if t > 70.0 {
            "🔥 Warm"
        } else if t > 50.0 {
            "🌡️ Normal"
        } else {
            "❄️ Cool"
        };
        gpu_info.push(Line::from(format!("Thermal: {}", thermal_status)));
    }

    let info_paragraph = Paragraph::new(gpu_info)
        .block(Block::default()
            .title("📈 GPU Analytics")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta)))
        .style(Style::default().fg(Color::White));
    f.render_widget(info_paragraph, chunks[2]);
}

