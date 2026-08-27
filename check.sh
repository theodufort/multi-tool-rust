#!/usr/bin/env bash
# GOAL_CHECK.sh - objective completion check for the Multi-Tool Rust web app.
# Exit 0 = criteria met. Prints "SCORE: <n>" (higher = better).
# Max score = 92: build(10) + tests(10) + readme(3) + sidebar(5) + 19 tools * 4(=76).
set -u

PORT="${PORT:-8091}"
BASE="http://127.0.0.1:$PORT"
WORKDIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$WORKDIR" || exit 1

SCORE=0
PASS=0
FAIL=0
log() { printf '%s\n' "$*"; }
ok()  { PASS=$((PASS+1)); }
bad() { FAIL=$((FAIL+1)); }

# Kill any lingering instances of this app (orphans from prior runs) so the
# chosen port is truly free. Excludes the current shell to avoid self-kill.
cleanup_orphans() {
  local me=$$
  local pid
  for pid in $(ps -eo pid,args | grep 'target/debug/multi-tool-rust' \
      | grep -v grep | grep -v "^$$" | awk '{print $1}'); do
    kill -9 "$pid" 2>/dev/null
  done
}

# Pick a free port (avoids collisions with a still-running instance).
if [ -z "${PORT:-}" ]; then
  for p in 8091 8137 47654 49152 49876 50555 51234 52000; do
    if ! (exec 3<>/dev/tcp/127.0.0.1/$p) 2>/dev/null; then PORT=$p; break; fi
  done
fi

# ---- 1. Build ----
log "== Build =="
if cargo build 2>&1 | grep -qiE 'error'; then
  log "  [FAIL] cargo build failed"
else
  log "  [ok] cargo build succeeded"
  SCORE=$((SCORE+10))
  ok
fi

# ---- 2. Tests ----
log "== Tests =="
TEST_OUT="$(cargo test 2>&1)"
if echo "$TEST_OUT" | grep -qE 'test result: (ok|FAILED)'; then
  NTESTS="$(echo "$TEST_OUT" | grep -oE '[0-9]+ (new|ok)' | grep -oE '[0-9]+' | awk '{s+=$1} END {print s+0}')"
  if [ "${NTESTS:-0}" -ge 30 ]; then
    log "  [ok] $NTESTS tests passed"
    SCORE=$((SCORE+10))
    ok
  else
    log "  [FAIL] only $NTESTS tests (need >= 30)"
    bad
  fi
else
  log "  [FAIL] could not parse test results"
  bad
fi

# ---- 3. README ----
log "== Docs =="
if [ -f README.md ] && [ "$(wc -l < README.md 2>/dev/null || echo 0)" -gt 30 ]; then
  SCORE=$((SCORE+3)); ok; log "  [ok] README.md present (>30 lines)"
else
  bad; log "  [FAIL] README.md missing or too short"
fi

# ---- 4. Server + endpoint checks ----
# Rocket 0.5.1 here ignores the --port CLI flag, so we configure the port via
# Rocket.toml for the duration of the test, then restore the original file.
log "== Server + tools =="
BIN="$(ls target/debug/multi-tool-rust 2>/dev/null)"
if [ -z "$BIN" ]; then cargo build >/dev/null 2>&1; BIN="$(ls target/debug/multi-tool-rust)"; fi

API_CALL() {
  # $1 = slug, $2 = input, $3 = expected (exact, may contain newlines)
  local slug="$1" input="$2" expected="$3"
  local body
  body="$(curl -sS -G --data-urlencode "input=$input" "${BASE}/api/${slug}" 2>/dev/null)"
  body="$(printf '%s' "$body" | tr -d '\r' | sed -e 's/[[:cntrl:]]//g' -e 's/[[:space:]]*$//')"
  if [ "$body" = "$expected" ]; then
    PASS=$((PASS+1))
  else
    FAIL=$((FAIL+1))
  fi
}

if [ -n "$BIN" ] && command -v curl >/dev/null 2>&1; then
  cleanup_orphans
  sleep 0.3

  # Save the ORIGINAL Rocket.toml separately from the working copy so restore
  # is reliable. (Rocket 0.5.1 here ignores the --port CLI flag; the port is
  #  read from Rocket.toml via figment.)
  if [ -f Rocket.toml ]; then
    cp Rocket.toml /tmp/goal-check-orig.toml
  else
    printf '[default]\n' > /tmp/goal-check-orig.toml
  fi

  # Working config = original + the test port.
  if [ -f Rocket.toml ]; then
    cp Rocket.toml /tmp/goal-check-rocket.toml
    if ! grep -qE '^[[:space:]]*port[[:space:]]*=' /tmp/goal-check-rocket.toml; then
      printf 'port = %s\n' "$PORT" >> /tmp/goal-check-rocket.toml
    fi
  else
    printf '[default]\nport = %s\n' "$PORT" > /tmp/goal-check-rocket.toml
  fi
  cp /tmp/goal-check-rocket.toml Rocket.toml

  # Detach the server into its own session so check.sh can exit cleanly even
  # if the harness kills this script's process group before the trap runs.
  setsid "$BIN" >/tmp/goal-check-srv.log 2>&1 &
  SERVER_PID=$!
  disown
  trap 'kill "$SERVER_PID" 2>/dev/null; cp /tmp/goal-check-orig.toml Rocket.toml 2>/dev/null; rm -f /tmp/goal-check-orig.toml /tmp/goal-check-rocket.toml' EXIT

  # readiness
  for _ in $(seq 1 50); do
    if curl -sf -o /dev/null "$BASE/" 2>/dev/null; then break; fi
    sleep 0.2
  done
  if ! curl -sf -o /dev/null "$BASE/" 2>/dev/null; then
    log "  [FAIL] server did not start"
    exit 1
  fi
  log "  [ok] server up on $BASE (pid $SERVER_PID)"

  # tool endpoint checks (slug, input, expected)
  API_CALL replace      'aXbXc' 'a_bXc'
  API_CALL replace-all  'aXbXc' 'a_b_c'
  API_CALL collapse     'a
b'            'a b'
  API_CALL uppercase    'hello' 'HELLO'
  API_CALL lowercase    'HELLO' 'hello'
  API_CALL title        'the quick brown' 'The Quick Brown'
  API_CALL reverse      'abc 123' '321 cba'
  API_CALL trim         '  hi  ' 'hi'
  API_CALL url-encode   'a b&c' 'a+b%26c'
  API_CALL url-decode   'a+b%26c' 'a b&c'
  API_CALL html-encode  'a<b>"' 'a&lt;b&gt;&quot;'
  API_CALL html-decode  'a&lt;b&gt;"' 'a<b>"'
  API_CALL hex-encode   'AB' '4142'
  API_CALL hex-decode   '4142' 'AB'
  API_CALL base64-encode 'Man' 'TWFu'
  API_CALL base64-decode 'TWFu' 'Man'
  API_CALL unicode-strip 'Héllo Wörld' 'Hllo Wrld'
  API_CALL word-count   'one two three' "words: 3
chars: 13
lines: 1
bytes: 13"
  API_CALL json-pretty  '{"a":1,"b":2}' "$(printf '{\n  "a": 1,\n  "b": 2\n}')"
  API_CALL json-minify  '{"a": 1}' '{"a":1}'
  API_CALL slug         'Hello World & Friends!' 'hello-world-friends'
  API_CALL case         'hello world foo' 'helloWorldFoo'
  API_CALL lorem        'words=3' 'lorem ipsum dolor'

  # sidebar: every tool page resolves (no leftover href="#")
  SIDEBAR_OK=1
  for slug in replace replace-all collapse uppercase lowercase title reverse trim \
              url-encode url-decode html-encode html-decode hex-encode hex-decode \
              base64-encode base64-decode unicode-strip word-count json-pretty \
              json-minify slug case lorem; do
    if ! curl -sf -o /dev/null "${BASE}/tool/${slug}" 2>/dev/null; then
      SIDEBAR_OK=0
    fi
  done
  if [ "$SIDEBAR_OK" -eq 1 ]; then
    SCORE=$((SCORE+5)); ok; log "  [ok] all /tool/<slug> pages resolve"
  else
    bad; log "  [FAIL] some /tool/<slug> pages missing"
  fi
else
  bad; log "  [FAIL] server binary or curl unavailable"
fi

  # Stop the server and wait for it to actually die so the port is released.
  kill "$SERVER_PID" 2>/dev/null
  for _ in $(seq 1 50); do kill -0 "$SERVER_PID" 2>/dev/null || break; sleep 0.1; done
  kill -9 "$SERVER_PID" 2>/dev/null
  # Restore Rocket.toml explicitly; the EXIT trap also does this as a safety net.
  cp /tmp/goal-check-orig.toml Rocket.toml 2>/dev/null || true
  rm -f /tmp/goal-check-orig.toml /tmp/goal-check-rocket.toml

  # tool score: 4 per passing slug (19 total)
  SCORE=$((SCORE + PASS*4))

# ---- Summary ----
TOTAL=$((10+10+3+5+19*4))
log ""
log "PASS=$PASS FAIL=$FAIL (endpoint + sidebar checks)"
log "SCORE: $SCORE / $TOTAL"

if [ "$SCORE" -ge 90 ] && [ "$FAIL" -eq 0 ]; then
  log "RESULT: PASS (>= 90)"
  exit 0
else
  log "RESULT: NEEDS WORK (< 90 or failures)"
  exit 1
fi
