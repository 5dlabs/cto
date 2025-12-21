#!/bin/bash
# WiFi Diagnostics Script
set -euo pipefail

WIFI_INTERFACE="en0"
PRIMARY_SSID="${PRIMARY_SSID:-TELUS2090}"
BACKUP_SSID="${BACKUP_SSID:-Johns iPhone}"

get_current_network() {
    networksetup -getairportnetwork "$WIFI_INTERFACE" 2>/dev/null | sed 's/Current Wi-Fi Network: //' || echo "Not connected"
}

check_internet() {
    ping -c 1 -W 2 8.8.8.8 >/dev/null 2>&1
}

get_signal_strength() {
    /System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport -I 2>/dev/null | grep agrCtlRSSI | awk '{print $2}' || echo ""
}

diagnose() {
    echo "=== WiFi Diagnostics Report ==="
    echo "Time: $(date)"
    echo ""
    
    echo "=== Current Connection ==="
    CURRENT=$(get_current_network)
    echo "Network: $CURRENT"
    
    RSSI=$(get_signal_strength)
    if [ -n "$RSSI" ]; then
        echo "Signal: $RSSI dBm"
    fi
    echo ""
    
    echo "=== Internet Connectivity ==="
    if check_internet; then
        echo "Internet: OK"
    else
        echo "Internet: FAIL"
    fi
    
    if host google.com >/dev/null 2>&1; then
        echo "DNS: OK"
    else
        echo "DNS: FAIL"
    fi
    echo ""
    
    echo "=== Interface Details ==="
    IP=$(ifconfig "$WIFI_INTERFACE" 2>/dev/null | grep "inet " | awk '{print $2}')
    echo "Interface: $WIFI_INTERFACE"
    echo "IP: ${IP:-N/A}"
    echo ""
    
    echo "=== WiFi Channel Info ==="
    /System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport -I 2>/dev/null | grep -E "channel|BSSID|lastTxRate" || echo "N/A"
    echo ""
    
    echo "=== Preferred Networks ==="
    networksetup -listpreferredwirelessnetworks "$WIFI_INTERFACE" 2>/dev/null | head -10
}

monitor() {
    echo "=== WiFi Monitor Mode ==="
    echo "Press Ctrl+C to stop"
    echo ""
    
    while true; do
        CURRENT=$(get_current_network)
        RSSI=$(get_signal_strength)
        
        if check_internet; then
            STATUS="OK"
        else
            STATUS="FAIL"
        fi
        
        printf "\r[%s] %s | %s dBm | Internet: %s     " "$(date +%H:%M:%S)" "$CURRENT" "${RSSI:-N/A}" "$STATUS"
        sleep 5
    done
}

reconnect() {
    echo "=== Reconnecting to $PRIMARY_SSID ==="
    
    CURRENT=$(get_current_network)
    if [ "$CURRENT" = "$PRIMARY_SSID" ]; then
        echo "Already connected to $PRIMARY_SSID"
        return 0
    fi
    
    echo "Attempting connection..."
    if networksetup -setairportnetwork "$WIFI_INTERFACE" "$PRIMARY_SSID" 2>/dev/null; then
        sleep 3
        CURRENT=$(get_current_network)
        if [ "$CURRENT" = "$PRIMARY_SSID" ]; then
            echo "Connected to $PRIMARY_SSID"
            return 0
        fi
    fi
    
    echo "Failed - may need password"
    return 1
}

failover() {
    echo "=== Failover to Backup ==="
    
    if ! check_internet; then
        echo "Internet down, trying $BACKUP_SSID..."
        if networksetup -setairportnetwork "$WIFI_INTERFACE" "$BACKUP_SSID" 2>/dev/null; then
            sleep 3
            if check_internet; then
                echo "Failover successful"
                return 0
            fi
        fi
        echo "Failover failed"
        return 1
    else
        echo "Internet working, no failover needed"
        return 0
    fi
}

show_help() {
    echo "WiFi Diagnostics Script"
    echo ""
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  diagnose    Full diagnostic report"
    echo "  monitor     Continuous monitoring"
    echo "  reconnect   Reconnect to primary"
    echo "  failover    Switch to backup"
    echo "  help        Show this help"
}

case "${1:-diagnose}" in
    diagnose)
        diagnose
        ;;
    monitor)
        monitor
        ;;
    reconnect)
        reconnect
        ;;
    failover)
        failover
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        echo "Unknown command: $1"
        show_help
        exit 1
        ;;
esac
