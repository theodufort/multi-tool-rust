#!/usr/bin/env bash
set -euo pipefail

# ── Configuration ──────────────────────────────────────────────
# APP_DIR: explicit env override wins, otherwise the repo root —
# the parent of the directory this script lives in. The adnanh/webhook
# hook runs this script from the checkout dir, so this resolves to
# the actual deploy location (e.g. /storage_block/multi-tool-rust).
APP_DIR="${APP_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
LOG_FILE="/var/log/multi-tool-rust/deploy.log"
COMPOSE_FILE="docker-compose.yml"
MAX_LOG_SIZE=10485760  # 10 MB
LOCK_FILE="/var/run/multi-tool-rust-deploy.lock"
LOCK_TIMEOUT=1500  # ~25 min: covers full build + up

# ── Concurrency guard ───────────────────────────────────────────
# adnanh/webhook can trigger two deploys back-to-back (e.g. two quick
# pushes). Interleaved `docker compose down/up` calls corrupt the
# running stack, so only one deploy may run at a time.
mkdir -p "$(dirname "$LOCK_FILE")"
exec 9>"$LOCK_FILE"
if ! flock -w "$LOCK_TIMEOUT" 9; then
  echo "[$(date '+%Y-%m-%d %H:%M:%S')] Another deploy is in progress -- aborting" >&2
  exit 1
fi

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
cd "$APP_DIR"
log "Deploying from $APP_DIR"

if [[ -d .git ]] && git remote get-url origin > /dev/null 2>&1; then
  log "Pulling latest changes from main..."
  git fetch origin main
  git reset --hard origin/main
  log "Code pulled successfully ($(git rev-parse --short HEAD))"
else
  log "No git origin remote -- deploying files as-is in $APP_DIR"
fi

if [[ ! -f "$COMPOSE_FILE" ]]; then
  log "ERROR: $COMPOSE_FILE not found in $APP_DIR"
  exit 1
fi

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
curl -fsS --retry 3 --retry-delay 2 --retry-connrefused --max-time 10 http://localhost:8093/ > /dev/null
log "HTTP smoke test passed (GET / -> 200)"

log "All containers healthy"
log "===== DEPLOY FINISHED ====="
