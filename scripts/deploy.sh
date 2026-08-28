#!/usr/bin/env bash
set -euo pipefail

# ── Configuration ──────────────────────────────────────────────
APP_DIR="/opt/multi-tool-rust"
LOG_FILE="/var/log/multi-tool-rust/deploy.log"
COMPOSE_FILE="docker-compose.yml"
MAX_LOG_SIZE=10485760  # 10 MB

# ── Helpers ────────────────────────────────────────────────────
timestamp() { date '+%Y-%m-%d %H:%M:%S'; }

log() {
  local msg="[$(timestamp)] $*"
  echo "$msg" | tee -a "$LOG_FILE"
}

rotate_log() {
  if [[ -f "$LOG_FILE" ]] && (( $(stat -c%s "$LOG_FILE" 2>/dev/null || echo 0) > MAX_LOG_SIZE )); then
    mv "$LOG_FILE" "${LOG_FILE}.$(date +%Y%m%d%H%M%S).bak"
    log "Rotated old log file"
  fi
}

# ── Pre-flight ─────────────────────────────────────────────────
mkdir -p "$(dirname "$LOG_FILE")"
rotate_log

log "===== DEPLOY STARTED ====="

# ── Pull latest code ───────────────────────────────────────────
log "Pulling latest changes from main..."
cd "$APP_DIR"
git fetch origin main
git reset --hard origin/main
log "Code pulled successfully"

# ── Docker Compose ─────────────────────────────────────────────
log "Stopping existing containers..."
docker compose -f "$COMPOSE_FILE" down

log "Building images..."
docker compose -f "$COMPOSE_FILE" build --no-cache

log "Starting containers..."
docker compose -f "$COMPOSE_FILE" up -d

# ── Post-deploy verification ───────────────────────────────────
sleep 5

log "Running container health check..."
if [[ -n "$(docker compose -f "$COMPOSE_FILE" ps -q multi-tool-rust)" ]] \
  && docker compose -f "$COMPOSE_FILE" ps -a | grep -q "Up"; then
  log "Containers are up"
else
  log "ERROR: Containers are not up"
  docker compose -f "$COMPOSE_FILE" ps -a
  exit 1
fi

# App-level smoke test (Rocket listens on 8091 in the container,
# published on the host port via docker-compose)
curl -fsS --retry 3 --retry-delay 2 --retry-connrefused --max-time 10 http://localhost:8091/ > /dev/null
log "HTTP smoke test passed (GET / -> 200)"

log "All containers healthy"
log "===== DEPLOY FINISHED ====="
