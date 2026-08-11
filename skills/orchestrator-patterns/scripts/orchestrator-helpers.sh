#!/usr/bin/env bash
# Orchestrator Helpers — source this in multi-agent commands
# Usage: source ~/.claude/skills/orchestrator-patterns/scripts/orchestrator-helpers.sh
#
# Provides: state management, resume detection, timing, telemetry, notifications
# Dependencies: jq, date (GNU coreutils)

# strict mode only when executed directly, never when sourced (avoid mutating the caller shell)
(return 0 2>/dev/null) || set -euo pipefail

# --- Path Conventions ---

# Canonical state file path for a command
# Usage: orch_state_path <command_name>
# Returns: $HOME/.claude/scripts/state/<command>-state.json
orch_state_path() {
  local cmd="$1"
  local dir="$HOME/.claude/scripts/state"
  mkdir -p "$dir"
  echo "${dir}/${cmd}-state.json"
}

# Canonical telemetry file path for a command
# Usage: orch_telemetry_path <command_name>
# Returns: $HOME/.claude/scripts/state/<command>-telemetry.jsonl
orch_telemetry_path() {
  local cmd="$1"
  local dir="$HOME/.claude/scripts/state"
  mkdir -p "$dir"
  echo "${dir}/${cmd}-telemetry.jsonl"
}

# --- State Management ---

# Initialize orchestrator state file
# Usage: orch_state_init <state_file> <git_sha> <phase_names...>
orch_state_init() {
  local state_file="$1" git_sha="$2"
  shift 2
  local phases=("$@")
  local now_iso now_ms phases_json tmp

  now_iso=$(date -Iseconds)
  now_ms=$(date +%s%3N)
  tmp="${state_file}.tmp"

  phases_json="["
  local first=true
  for phase in "${phases[@]}"; do
    if [[ "$first" == "true" ]]; then
      first=false
    else
      phases_json+=","
    fi
    phases_json+="{\"name\":\"${phase}\",\"status\":\"pending\",\"started_at\":null,\"completed_at\":null,\"agent_outputs\":{}}"
  done
  phases_json+="]"

  mkdir -p "$(dirname "$state_file")"
  cat > "$tmp" <<EOF
{
  "git_sha": "${git_sha}",
  "started_at": "${now_iso}",
  "started_at_ms": ${now_ms},
  "phases": ${phases_json}
}
EOF
  mv -f "$tmp" "$state_file"
}

# Mark a phase complete with optional outputs JSON
# Usage: orch_state_complete <state_file> <phase_name> [outputs_json]
orch_state_complete() {
  local state_file="$1" phase="$2"
  # `local outputs="${3:-{}}"` looks right but isn't: bash's `${...}` brace
  # matching finds the terminator at the SECOND `}` (the one that closes the
  # `{}` default), leaving the actual THIRD `}` as literal trailing text
  # appended to any real (non-default) $3 value — corrupting the JSON on
  # every call that passes real output data (verified: `{"x":1}` became
  # `{"x":1}}`), while silently working for the empty-default case. Two-step
  # form avoids the ambiguous inline brace default entirely.
  local outputs="${3-}"
  [ -z "$outputs" ] && outputs="{}"
  local now tmp

  now=$(date -Iseconds)
  tmp="${state_file}.tmp"

  jq --arg p "$phase" --arg now "$now" --argjson out "$outputs" '
    .phases |= map(
      if .name == $p then
        .status = "completed" | .completed_at = $now | .agent_outputs = $out
      else . end
    )
  ' "$state_file" > "$tmp" && mv -f "$tmp" "$state_file"
}

# Get phase status
# Usage: orch_state_phase_status <state_file> <phase_name>
orch_state_phase_status() {
  local state_file="$1" phase="$2"
  jq -r --arg p "$phase" '.phases[] | select(.name == $p) | .status' "$state_file"
}

# Record agent start time (call before dispatch)
# Usage: orch_state_start_node <state_file> <phase_name> <node_name>
orch_state_start_node() {
  local state_file="$1" phase="$2" node="$3"
  local now_ms tmp

  now_ms=$(date +%s%3N)
  tmp="${state_file}.tmp"

  jq --arg p "$phase" --arg n "$node" --argjson now "$now_ms" '
    .phases |= map(
      if .name == $p then
        .status = "running" |
        .started_at = (.started_at // ($now | tostring)) |
        .agent_outputs[$n] = ((.agent_outputs[$n] // {}) + { status: "running", started_at_ms: $now })
      else . end
    )
  ' "$state_file" > "$tmp" && mv -f "$tmp" "$state_file"
}

# Update agent status with auto-computed wall_clock_ms
# Usage: orch_state_update_node <state_file> <phase_name> <node_name> <status> [data]
orch_state_update_node() {
  local state_file="$1" phase="$2" node="$3" status="$4" data="${5:-}"
  local key="report" now_ms tmp

  now_ms=$(date +%s%3N)
  tmp="${state_file}.tmp"
  [[ "$status" == "failed" ]] && key="error"

  jq --arg p "$phase" --arg n "$node" --arg s "$status" --arg d "$data" --arg k "$key" --argjson now "$now_ms" '
    .phases |= map(
      if .name == $p then
        .agent_outputs[$n] = (
          (.agent_outputs[$n] // {}) +
          { status: $s, ($k): $d, wall_clock_ms: ($now - (.agent_outputs[$n].started_at_ms // $now)) }
        ) |
        if (.agent_outputs | to_entries | length > 0) and
           (.agent_outputs | to_entries | all(.value.status == "completed" or .value.status == "failed")) then
          if (.agent_outputs | to_entries | all(.value.status == "completed")) then
            .status = "completed" | .completed_at = ($now / 1000 | strftime("%Y-%m-%dT%H:%M:%S+00:00"))
          else .status = "partial" end
        else . end
      else . end
    )
  ' "$state_file" > "$tmp" && mv -f "$tmp" "$state_file"
}

# Get node status within a phase
# Usage: orch_state_node_status <state_file> <phase_name> <node_name>
orch_state_node_status() {
  local state_file="$1" phase="$2" node="$3"
  jq -r --arg p "$phase" --arg n "$node" \
    '.phases[] | select(.name == $p) | .agent_outputs[$n].status // "pending"' "$state_file"
}

# --- Resume Detection ---

# Check if state file exists and matches current git SHA
# Returns: 0 if resumable, 1 if fresh start needed
# Usage: orch_check_resume <state_file> [--fresh]
orch_check_resume() {
  local state_file="$1"
  local fresh="${2:-}"
  local current_sha saved_sha completed

  current_sha=$(git rev-parse --short HEAD)

  # --fresh flag: delete state and restart
  if [[ "$fresh" == "--fresh" ]]; then
    rm -f "$state_file"
    echo "Fresh start requested. Ignoring any previous state."
    return 1
  fi

  # No state file → fresh start
  if [[ ! -f "$state_file" ]]; then
    return 1
  fi

  # SHA mismatch → fresh start
  saved_sha=$(jq -r '.git_sha' "$state_file")
  if [[ "$saved_sha" != "$current_sha" ]]; then
    echo "Code changed since last run ($saved_sha -> $current_sha). Starting fresh."
    rm -f "$state_file"
    return 1
  fi

  # Resumable
  completed=$(jq -r '[.phases[] | select(.status == "completed") | .name] | join(", ")' "$state_file")
  echo "Resuming from previous run. Completed: $completed"
  return 0
}

# --- Timing & Display ---

# Format milliseconds to human-readable (312000 -> "5m 12s")
# Usage: orch_fmt_duration <ms>
orch_fmt_duration() {
  local ms=$1
  if [[ $ms -lt 1000 ]]; then
    echo "${ms}ms"
  elif [[ $ms -lt 60000 ]]; then
    echo "$(( ms / 1000 ))s"
  elif [[ $ms -lt 3600000 ]]; then
    echo "$(( ms / 60000 ))m $(( (ms % 60000) / 1000 ))s"
  else
    echo "$(( ms / 3600000 ))h $(( (ms % 3600000) / 60000 ))m"
  fi
}

# Print timing report table from state file
# Usage: orch_timing_report <state_file>
orch_timing_report() {
  local state_file="$1"
  local audit_end_ms audit_start_ms total_ms

  audit_end_ms=$(date +%s%3N)
  audit_start_ms=$(jq -r '.started_at_ms // 0' "$state_file")
  total_ms=$((audit_end_ms - audit_start_ms))

  echo ""
  echo "Timing Report"
  echo "============================================"
  printf "| %-30s | %-15s |\n" "Phase" "Duration"
  echo "|--------------------------------|-----------------|"

  # Phase-level timing
  local phase_names
  phase_names=$(jq -r '.phases[].name' "$state_file")

  while IFS= read -r phase; do
    local phase_start phase_end ps pe dur
    phase_start=$(jq -r --arg p "$phase" '.phases[] | select(.name == $p) | .started_at // "n/a"' "$state_file")
    phase_end=$(jq -r --arg p "$phase" '.phases[] | select(.name == $p) | .completed_at // "n/a"' "$state_file")

    if [[ "$phase_start" != "n/a" && "$phase_start" != "null" && "$phase_end" != "n/a" && "$phase_end" != "null" ]]; then
      ps=$(date -d "$phase_start" +%s%3N 2>/dev/null || echo 0)
      pe=$(date -d "$phase_end" +%s%3N 2>/dev/null || echo 0)
      dur=$((pe - ps))
      printf "| %-30s | %-15s |\n" "$phase" "$(orch_fmt_duration $dur)"
    else
      printf "| %-30s | %-15s |\n" "$phase" "skipped"
    fi
  done <<< "$phase_names"

  echo "|--------------------------------|-----------------|"
  printf "| %-30s | %-15s |\n" "TOTAL" "$(orch_fmt_duration $total_ms)"
  echo ""

  # Per-node breakdown for phases with agent_outputs
  local phases_with_nodes
  phases_with_nodes=$(jq -r '.phases[] | select(.agent_outputs | length > 0) | .name' "$state_file")

  if [[ -n "$phases_with_nodes" ]]; then
    while IFS= read -r phase; do
      echo "### Node Breakdown ($phase)"
      printf "| %-10s | %-12s | %-15s |\n" "Node" "Status" "Duration"
      echo "|------------|--------------|-----------------|"

      local nodes
      nodes=$(jq -r --arg p "$phase" '.phases[] | select(.name == $p) | .agent_outputs | keys[]' "$state_file")

      while IFS= read -r node; do
        local node_status node_ms
        node_status=$(jq -r --arg p "$phase" --arg n "$node" \
          '.phases[] | select(.name == $p) | .agent_outputs[$n].status // "n/a"' "$state_file")
        node_ms=$(jq -r --arg p "$phase" --arg n "$node" \
          '.phases[] | select(.name == $p) | .agent_outputs[$n].wall_clock_ms // 0' "$state_file")

        if [[ "$node_status" != "n/a" ]]; then
          printf "| %-10s | %-12s | %-15s |\n" "$node" "$node_status" "$(orch_fmt_duration "$node_ms")"
        fi
      done <<< "$nodes"

      echo ""
    done <<< "$phases_with_nodes"
  fi
}

# --- Telemetry ---

# Unified telemetry sink — all commands log to a single file
# Usage: orch_telemetry_log <command> <phase> <agent_count> <wall_ms> <spec_name> <pipeline_id> [run_id]
# spec_name and pipeline_id may be "null" for commands without spec context (e.g., /p2p)
# run_id is optional — correlates with command_start events in agents.jsonl
orch_telemetry_log() {
  local command="$1" phase="$2" agents="$3" wall_ms="$4" spec_name="${5:-null}" pipeline_id="${6:-null}" run_id="${7:-null}"
  local telemetry_file="$HOME/.claude/scripts/state/telemetry.jsonl"
  local now count tmp

  now=$(date -Iseconds)
  mkdir -p "$(dirname "$telemetry_file")"

  # Quote strings, leave null unquoted
  local spec_val="null" pid_val="null" rid_val="null"
  [[ "$spec_name" != "null" ]] && spec_val="\"$spec_name\""
  [[ "$pipeline_id" != "null" ]] && pid_val="\"$pipeline_id\""
  [[ "$run_id" != "null" ]] && rid_val="\"$run_id\""

  local record="{\"timestamp\":\"$now\",\"pipeline_id\":$pid_val,\"spec_name\":$spec_val,\"command\":\"$command\",\"phase\":\"$phase\",\"agent_count\":$agents,\"duration_ms\":$wall_ms,\"domains\":\"all\""
  [[ "$run_id" != "null" ]] && record="${record},\"run_id\":$rid_val"
  record="${record}}"

  echo "$record" >> "$telemetry_file"

  # Rotate: if >100 entries, keep last 50
  count=$(wc -l < "$telemetry_file" 2>/dev/null || echo 0)
  if [[ $count -gt 100 ]]; then
    tmp="${telemetry_file}.tmp"
    tail -50 "$telemetry_file" > "$tmp" && mv -f "$tmp" "$telemetry_file"
  fi
}

# --- Pipeline ID ---

# Generate a UUIDv4 for pipeline correlation across commands
# Usage: orch_pipeline_id
# Returns: UUID string (e.g., "550e8400-e29b-41d4-a716-446655440000")
orch_pipeline_id() {
  if command -v uuidgen &>/dev/null; then
    uuidgen | tr '[:upper:]' '[:lower:]'
  elif [[ -r /proc/sys/kernel/random/uuid ]]; then
    cat /proc/sys/kernel/random/uuid
  else
    # Pure bash fallback — not cryptographically random but sufficient for correlation
    printf '%04x%04x-%04x-%04x-%04x-%04x%04x%04x\n' \
      $RANDOM $RANDOM $RANDOM $(( ($RANDOM & 0x0fff) | 0x4000 )) \
      $(( ($RANDOM & 0x3fff) | 0x8000 )) $RANDOM $RANDOM $RANDOM
  fi
}

# --- Notifications ---

# Send a bounded, fail-soft TTS notification.
# Usage: orch_tts_notify <message>
orch_tts_notify() {
  local message="$1"
  say_notify "$message"
}
