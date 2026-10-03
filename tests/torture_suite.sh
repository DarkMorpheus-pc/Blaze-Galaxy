#!/usr/bin/env bash
# ==============================================================================
# BlazeOS SolarUI Industrial Torture & Soak Test Suite
# Pillars:
#   1. Window Churn (Thousands of open/close/resize/workspace/fullscreen cycles)
#   2. Event Storm (PipeWire, NetworkManager, UPower, Notifications spam)
#   3. Fault Injection / Resilience (Kill/Restart Noctalia/Caelestia providers)
#   4. Telemetry (RSS, PSS, USS, VMS, FDs, Threads, Crash counts)
# Supports A/B benchmark: Vanilla Niri vs SolarUI Hybrid Shell
# ==============================================================================

set -euo pipefail

DURATION_SECS="${DURATION_SECS:-1200}" # Default 20 mins
TARGET_PROCESS="${TARGET_PROCESS:-solar-shell}"
MODE="${1:-solarui}" # "solarui" or "vanilla"
OUT_DIR="/tmp/solarui_torture_${MODE}_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$OUT_DIR"

TELEMETRY_LOG="$OUT_DIR/telemetry.csv"
EVENT_LOG="$OUT_DIR/events.log"
CRASH_LOG="$OUT_DIR/crashes.log"

echo "=== Starting BlazeOS Torture Suite [$MODE mode] ==="
echo "Duration: ${DURATION_SECS}s | Target: $TARGET_PROCESS | Logs: $OUT_DIR"

# Header for Telemetry CSV
echo "timestamp,elapsed_sec,rss_kb,pss_kb,vms_kb,threads,fds" > "$TELEMETRY_LOG"

START_TIME=$(date +%s)

# Ensure background workers are cleanly stopped on exit
cleanup() {
    echo ""
    echo "=== Stopping Torture Suite Workers ==="
    kill $(jobs -p) 2>/dev/null || true
    echo "Summary saved to $OUT_DIR"
}
trap cleanup EXIT INT TERM

# ── Telemetry Monitor Worker ─────────────────────────────────────────────────
telemetry_worker() {
    local target="$1"
    echo "[TELEMETRY] Monitoring process: $target"
    while true; do
        local pid
        pid=$(pgrep -n "$target" 2>/dev/null || true)
        local now
        now=$(date +%s)
        local elapsed=$(( now - START_TIME ))
        local iso_time
        iso_time=$(date --iso-8601=seconds)

        if [[ -n "$pid" && -d "/proc/$pid" ]]; then
            local rss=0 vms=0 thr=0 fds=0 pss=0
            
            # Read from /proc/$pid/status
            if [[ -f "/proc/$pid/status" ]]; then
                rss=$(awk '/VmRSS:/ {print $2}' "/proc/$pid/status" 2>/dev/null || echo 0)
                vms=$(awk '/VmSize:/ {print $2}' "/proc/$pid/status" 2>/dev/null || echo 0)
                thr=$(awk '/Threads:/ {print $2}' "/proc/$pid/status" 2>/dev/null || echo 0)
            fi

            # Read PSS from smaps_rollup if available (faster & safer than smaps)
            if [[ -f "/proc/$pid/smaps_rollup" ]]; then
                pss=$(awk '/^Pss:/ {print $2}' "/proc/$pid/smaps_rollup" 2>/dev/null || echo "$rss")
            else
                pss="$rss"
            fi

            # Count open file descriptors
            if [[ -d "/proc/$pid/fd" ]]; then
                fds=$(ls -1 "/proc/$pid/fd" 2>/dev/null | wc -l)
            fi

            echo "$iso_time,$elapsed,$rss,$pss,$vms,$thr,$fds" >> "$TELEMETRY_LOG"
            printf "[%4ds] %s (PID %s) -> RSS: %7s KB | PSS: %7s KB | FDs: %4s | Thr: %3s\n" \
                "$elapsed" "$target" "$pid" "$rss" "$pss" "$fds" "$thr"
        else
            echo "$iso_time,$elapsed,0,0,0,0,0" >> "$TELEMETRY_LOG"
            echo "[$iso_time] WARNING: Target process $target NOT RUNNING!" | tee -a "$CRASH_LOG"
        fi

        sleep 2
    done
}

# ── 1. Window Churn Worker ────────────────────────────────────────────────────
window_churn_worker() {
    echo "[CHURN] Starting Window Churn worker..."
    local niri_env_cmd="niri msg action"

    while true; do
        # Open 3 dummy/lightweight terminal windows
        for i in {1..3}; do
            alacritty --title "churn-test-$i" -e sleep 12 &>/dev/null &
            sleep 0.2
        done

        sleep 0.5
        # Move focus
        $niri_env_cmd focus-column-right 2>/dev/null || true
        sleep 0.1
        $niri_env_cmd focus-column-left 2>/dev/null || true

        # Toggle floating / tiling
        $niri_env_cmd toggle-window-floating 2>/dev/null || true
        sleep 0.1
        $niri_env_cmd toggle-window-floating 2>/dev/null || true

        # Fullscreen cycling
        $niri_env_cmd fullscreen-window 2>/dev/null || true
        sleep 0.1
        $niri_env_cmd fullscreen-window 2>/dev/null || true

        # Workspace hopping
        $niri_env_cmd focus-workspace-down 2>/dev/null || true
        sleep 0.2
        $niri_env_cmd focus-workspace-up 2>/dev/null || true

        # Resize spam
        $niri_env_cmd switch-preset-column-width 2>/dev/null || true
        sleep 0.1
        $niri_env_cmd switch-preset-column-width 2>/dev/null || true

        # Close all churn-test windows
        pkill -f "churn-test-" 2>/dev/null || true
        sleep 0.4
    done
}

# ── 2. Event Storm Worker ─────────────────────────────────────────────────────
event_storm_worker() {
    echo "[EVENT-STORM] Starting D-Bus / PipeWire / Network event storm..."
    while true; do
        # PipeWire Volume spam (wpctl)
        if command -v wpctl >/dev/null 2>&1; then
            wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+ 2>/dev/null || true
            sleep 0.05
            wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%- 2>/dev/null || true
            sleep 0.05
            wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle 2>/dev/null || true
            sleep 0.05
            wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle 2>/dev/null || true
        fi

        # Notification spam
        if command -v notify-send >/dev/null 2>&1; then
            notify-send -u low -a "TortureSuite" "Event Storm" "Testing notification event pipeline tick $(date +%N)" 2>/dev/null || true
        fi

        # NetworkManager state queries
        if command -v nmcli >/dev/null 2>&1; then
            nmcli -t -f RUNNING general 2>/dev/null || true
        fi

        sleep 0.3
    done
}

# ── 3. Fault Injection / Resilience Worker ───────────────────────────────────
fault_injection_worker() {
    if [[ "$MODE" == "vanilla" ]]; then
        return 0
    fi

    echo "[FAULT] Starting Provider Fault Injection worker..."
    while true; do
        # Sleep random interval between 25 and 60 seconds
        local wait_time=$(( 25 + RANDOM % 35 ))
        sleep "$wait_time"

        local victim
        if (( RANDOM % 2 == 0 )); then
            victim="noctalia"
        else
            victim="quickshell"
        fi

        local vpid
        vpid=$(pgrep -n "$victim" 2>/dev/null || true)
        if [[ -n "$vpid" ]]; then
            echo "[$(date --iso-8601=seconds)] FAULT INJECTION: Killing provider '$victim' (PID $vpid)..." | tee -a "$EVENT_LOG"
            kill -9 "$vpid" 2>/dev/null || true
            
            # Check if solar-shell supervisor recovers within 5 seconds
            sleep 5
            local new_pid
            new_pid=$(pgrep -n "$victim" 2>/dev/null || true)
            if [[ -n "$new_pid" ]]; then
                echo "[$(date --iso-8601=seconds)] RECOVERY SUCCESS: Provider '$victim' resurrected as PID $new_pid" | tee -a "$EVENT_LOG"
            else
                echo "[$(date --iso-8601=seconds)] RECOVERY FAILED: Provider '$victim' not restarted by supervisor" | tee -a "$CRASH_LOG"
            fi
        fi
    done
}

# ── Start Orchestration ───────────────────────────────────────────────────────
telemetry_worker "$TARGET_PROCESS" &
window_churn_worker &
event_storm_worker &
fault_injection_worker &

echo "[ORCHESTRATOR] All torture workers launched. Running for ${DURATION_SECS} seconds..."
sleep "$DURATION_SECS"

echo "=== Torture Test Completed Successfully ==="
