# rmon - Resource Monitor
<img width="3811" height="2092" alt="swappy-20250712_093415" src="https://github.com/user-attachments/assets/160301a4-a8bc-4079-b6ce-1b82c3fd2fea" />

`rmon` is a lightweight terminal application for keeping an eye on system resources in real time.
It offers a text user interface as well as a simple text output mode.

## Features

### CPU Monitoring
- Per-core usage graphs
- Package and per-core temperature readings using the system sensors
- Historical charts so spikes and trends are easy to spot

### GPU Monitoring
- NVIDIA GPU support via **nvidia-smi**
- Usage percentage and temperature
- Fan speed, power draw and VRAM usage where available
- Graphs for GPU load and memory utilisation

### Additional Monitoring
- Memory usage with history graphs
- Disk usage for the root filesystem
- Network activity and session totals
- Process list with sorting and kill options
- Journal log viewing

![swappy-20250708_102056](https://github.com/user-attachments/assets/6b847023-c80a-4da4-9bd1-51228acf682c)

![swappy-20250708_102125](https://github.com/user-attachments/assets/694e6d10-cbe9-4c06-88d2-ad2f949a858c)

## Installation

### From Packages

#### DEB Package (Debian/Ubuntu)
#### RPM Package (RHEL/CentOS/Fedora)
- Download from the releases page

### From Source

#### Prerequisites
- Rust 1.70+
- Cargo
- For GPU metrics, ensure `nvidia-smi` is available in your PATH

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

