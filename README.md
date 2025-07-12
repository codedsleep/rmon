# rmon - Resource Monitor
<img width="3804" height="2092" alt="swappy-20250712_152104" src="https://github.com/user-attachments/assets/b7a0d8d5-afff-48fb-b704-938bbe3339f1" />


A lightweight CLI system monitor with real-time monitoring capabilities, built in Rust.

## Features

- **Real-time CPU monitoring** with per-core usage and temperatures
- **Memory usage tracking** with history graphs
- **Process display**
- **Disk usage monitoring** for root filesystem
- **Network activity monitoring** with download/upload rates
- **GPU usage and temperature monitoring** (NVIDIA)
- **Journal listing**
- **Session-relative network totals**
- **Both TUI and simple text modes**
- **Comprehensive temperature monitoring**

![swappy-20250708_102056](https://github.com/user-attachments/assets/6b847023-c80a-4da4-9bd1-51228acf682c)

![swappy-20250708_102125](https://github.com/user-attachments/assets/694e6d10-cbe9-4c06-88d2-ad2f949a858c)

## Installation

### From Packages

#### DEB Package (Debian/Ubuntu)
#### RPM Package (RHEL/CentOS/Fedora)
- Download from releases section

### From Source

#### Prerequisites
- Rust 1.70+ 
- Cargo

#### Build and Install
```bash
git clone https://github.com/codedsleep/rmon
cd rmon
cargo build --release
sudo cp target/release/rmon /usr/local/bin/
```

## Usage

### TUI Mode (default)
```bash
rmon
```

### Simple Text Mode
```bash
rmon --simple
```

### Navigation (TUI Mode)
- **Tab**: Switch between panels (System Monitor, Processes, Journal Logs)
- **↑/↓**: Scroll in lists
- **PgUp/PgDn**: Fast scroll
- **C**: Sort processes by CPU usage
- **M**: Sort processes by Memory usage
- **K**: Kill selected process
- **q/Ctrl+C**: Quit

````
