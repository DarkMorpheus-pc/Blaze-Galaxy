#!/usr/bin/env bash
# BlazeOS Hardware & System Telemetry Benchmark Script
# Collects reproducible performance and system metrics in JSON / Human-readable format.
set -euo pipefail

FORMAT="${1:-text}"

get_val() {
    cat "$1" 2>/dev/null || echo "N/A"
}

TIMESTAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
KERNEL="$(uname -r)"
ARCH="$(uname -m)"
CPU_MODEL="$(lscpu | grep 'Model name:' | sed 's/Model name:[ \t]*//' || echo 'Unknown')"
CPU_CORES="$(nproc)"
MEM_TOTAL_KB="$(grep MemTotal /proc/meminfo | awk '{print $2}')"
MEM_AVAIL_KB="$(grep MemAvailable /proc/meminfo | awk '{print $2}')"
SWAPPINESS="$(cat /proc/sys/vm/swappiness 2>/dev/null || echo 'N/A')"
MAX_MAP="$(cat /proc/sys/vm/max_map_count 2>/dev/null || echo 'N/A')"
ZRAM_ACTIVE="false"
if [ -e /dev/zram0 ]; then
    ZRAM_ACTIVE="true"
fi

# Boot analyze if systemd is active
STARTUP_TIME="N/A"
if command -v systemd-analyze >/dev/null 2>&1; then
    STARTUP_TIME="$(systemd-analyze 2>/dev/null | head -n1 || echo 'N/A')"
fi

if [ "$FORMAT" = "json" ]; then
    cat <<EOF
{
  "timestamp": "$TIMESTAMP",
  "kernel": "$KERNEL",
  "arch": "$ARCH",
  "cpu_model": "$CPU_MODEL",
  "cpu_cores": $CPU_CORES,
  "mem_total_kb": $MEM_TOTAL_KB,
  "mem_avail_kb": $MEM_AVAIL_KB,
  "vm_swappiness": "$SWAPPINESS",
  "vm_max_map_count": "$MAX_MAP",
  "zram_active": $ZRAM_ACTIVE,
  "startup_time": "$STARTUP_TIME"
}
EOF
else
    echo "=========================================================="
    echo "   BlazeOS System & Performance Benchmark Telemetry"
    echo "=========================================================="
    echo "Timestamp:        $TIMESTAMP"
    echo "Kernel:           $KERNEL ($ARCH)"
    echo "CPU Model:        $CPU_MODEL ($CPU_CORES cores)"
    echo "Memory (Total):   $((MEM_TOTAL_KB / 1024)) MB"
    echo "Memory (Avail):   $((MEM_AVAIL_KB / 1024)) MB"
    echo "vm.swappiness:    $SWAPPINESS"
    echo "vm.max_map_count: $MAX_MAP"
    echo "ZRAM Enabled:     $ZRAM_ACTIVE"
    echo "Startup:          $STARTUP_TIME"
    echo "=========================================================="
fi
