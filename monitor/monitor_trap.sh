export MONITOR_TARGET_PID=$$
export MONITOR_MESSAGE_FILE="/tmp/termination_reason.$$"
readonly MONITOR_TARGET_PID
readonly MONITOR_MESSAGE_FILE

trap '{
    if [[ -f "$MONITOR_MESSAGE_FILE" ]]; then
        echo "[MONITOR TERMINATION]" >&2
        cat "$MONITOR_MESSAGE_FILE" >&2
        rm -f "$MONITOR_MESSAGE_FILE"
    fi
    exit 1
}' SIGUSR1