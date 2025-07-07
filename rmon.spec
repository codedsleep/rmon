Name:           rmon
Version:        0.1.0
Release:        1%{?dist}
Summary:        A lightweight CLI system monitor with real-time monitoring capabilities

License:        MIT
URL:            https://github.com/example/rmon
Source0:        %{name}-%{version}.tar.gz

BuildArch:      x86_64
Requires:       glibc

%description
rmon is a lightweight command-line system monitor that provides real-time monitoring
of CPU usage, memory consumption, disk usage, and network activity. It features both
a terminal UI mode with live graphs and gauges, and a simple text-based output mode.

Key features:
- Real-time CPU monitoring with per-core usage and temperatures
- Memory usage tracking with history graphs
- Disk usage monitoring for root filesystem
- Network activity monitoring with download/upload rates
- Session-relative network totals
- Both TUI and simple text modes
- Comprehensive temperature monitoring

%prep
%setup -q

%build
# Pre-built binary approach - build was done outside RPM
echo "Using pre-built binary"

%install
rm -rf $RPM_BUILD_ROOT
mkdir -p $RPM_BUILD_ROOT%{_bindir}
# Copy pre-built binary from build host
install -m 755 %{_builddir}/%{name}-%{version}/target/release/rmon $RPM_BUILD_ROOT%{_bindir}/rmon

%files
%{_bindir}/rmon

%changelog
* Mon Dec 30 2024 System Monitor Team - 0.1.0-1
- Initial package
- Added comprehensive system monitoring capabilities
- Includes CPU, memory, disk, and network monitoring
- Features both TUI and simple text output modes
- Added per-core CPU temperature monitoring
- Implements session-relative network tracking