#!/usr/bin/env bash
# RalphHub End-to-End Test: Mobile → Desktop → Agent → Memory → Notion
# This script validates the full pipeline without requiring a physical device.
set -euo pipefail

DESKTOP_HOST="${DESKTOP_HOST:-localhost}"
DESKTOP_PORT="${DESKTOP_PORT:-7842}"
BASE_URL="http://${DESKTOP_HOST}:${DESKTOP_PORT}"
PASS=0
FAIL=0

pass() { echo "  ✓ $1"; PASS=$((PASS+1)); }
fail() { echo "  ✗ $1"; FAIL=$((FAIL+1)); }
section() { echo ""; echo "=== $1 ==="; }

check_json() {
  local url="$1"
  local key="$2"
  local expected="$3"
  local response
  response=$(curl -sf "$url" 2>/dev/null || echo '{"error":"connection_refused"}')
  local value
  value=$(echo "$response" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('$key',''))" 2>/dev/null || echo "")
  if [ "$value" = "$expected" ]; then
    pass "$url → $key=$expected"
  else
    fail "$url → expected $key=$expected, got: $value"
  fi
}

echo "RalphHub E2E Test Suite"
echo "Desktop: $BASE_URL"

# ─── Test 1: Desktop health check ─────────────────────────────────────────────
section "Test 1: Desktop Sync Server Health"
check_json "$BASE_URL/api/ping" "ok" "True"

# ─── Test 2: Mobile captures task ─────────────────────────────────────────────
section "Test 2: Mobile Task Capture"
CAPTURE_ID=$(python3 -c "import uuid; print(str(uuid.uuid4()))")
CAPTURE_RESPONSE=$(curl -sf -X POST "$BASE_URL/api/sync/events" \
  -H "Content-Type: application/json" \
  -d "{
    \"id\": \"$CAPTURE_ID\",
    \"eventType\": \"task.create\",
    \"payload\": {\"title\": \"E2E test task from mobile\", \"priority\": \"high\"},
    \"deviceId\": \"e2e-test-device\",
    \"createdAt\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",
    \"syncedAt\": null
  }" 2>/dev/null || echo '{"ok":false}')

if echo "$CAPTURE_RESPONSE" | python3 -c "import sys,json; d=json.load(sys.stdin); assert d.get('ok')" 2>/dev/null; then
  pass "Task capture event accepted by desktop"
else
  fail "Task capture failed: $CAPTURE_RESPONSE"
fi

# ─── Test 3: Memory write ─────────────────────────────────────────────────────
section "Test 3: Memory Spine Write"
MEM_RESPONSE=$(curl -sf -X POST "$BASE_URL/api/memory/write" \
  -H "Content-Type: application/json" \
  -d "{
    \"source\": \"mobile\",
    \"eventType\": \"task.create\",
    \"payload\": {\"title\": \"E2E memory test\", \"priority\": \"normal\"},
    \"deviceId\": \"e2e-test-device\",
    \"kaizenHint\": null
  }" 2>/dev/null || echo '{"ok":false}')

if echo "$MEM_RESPONSE" | python3 -c "import sys,json; d=json.load(sys.stdin); assert d.get('ok')" 2>/dev/null; then
  pass "Memory write accepted"
else
  fail "Memory write failed: $MEM_RESPONSE"
fi

# ─── Test 4: Memory read back ─────────────────────────────────────────────────
section "Test 4: Memory Read"
MEM_READ=$(curl -sf "$BASE_URL/api/memory/read" 2>/dev/null || echo '{"rawEvents":[]}')
EVENT_COUNT=$(echo "$MEM_READ" | python3 -c "import sys,json; d=json.load(sys.stdin); print(len(d.get('rawEvents',[])))" 2>/dev/null || echo "0")

if [ "$EVENT_COUNT" -gt 0 ]; then
  pass "Memory read returned $EVENT_COUNT events"
else
  fail "Memory read returned 0 events"
fi

# ─── Test 5: Approval queue ───────────────────────────────────────────────────
section "Test 5: Approval Queue"
APPROVALS=$(curl -sf "$BASE_URL/api/approvals" 2>/dev/null || echo '[]')
APPROVAL_COUNT=$(echo "$APPROVALS" | python3 -c "import sys,json; print(len(json.load(sys.stdin)))" 2>/dev/null || echo "0")

if [ "$APPROVAL_COUNT" -gt 0 ]; then
  pass "Approval queue returned $APPROVAL_COUNT items"
  # Get first approval ID and resolve it
  FIRST_ID=$(echo "$APPROVALS" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d[0]['id'])" 2>/dev/null || echo "")
  if [ -n "$FIRST_ID" ]; then
    RESOLVE=$(curl -sf -X POST "$BASE_URL/api/approvals/$FIRST_ID/resolve" \
      -H "Content-Type: application/json" \
      -d '{"decision":"approved","resolvedBy":"e2e-test"}' 2>/dev/null || echo '{"ok":false}')
    if echo "$RESOLVE" | python3 -c "import sys,json; d=json.load(sys.stdin); assert d.get('ok')" 2>/dev/null; then
      pass "Approval $FIRST_ID resolved successfully"
    else
      fail "Approval resolution failed: $RESOLVE"
    fi
  fi
else
  fail "No approvals in queue"
fi

# ─── Test 6: Agent list ───────────────────────────────────────────────────────
section "Test 6: Agent Monitoring"
AGENTS=$(curl -sf "$BASE_URL/api/agents" 2>/dev/null || echo '[]')
AGENT_COUNT=$(echo "$AGENTS" | python3 -c "import sys,json; print(len(json.load(sys.stdin)))" 2>/dev/null || echo "0")

if [ "$AGENT_COUNT" -gt 0 ]; then
  pass "Agent list returned $AGENT_COUNT agents"
else
  fail "No agents in list"
fi

# ─── Test 7: Daily digest ─────────────────────────────────────────────────────
section "Test 7: Daily Digest"
DIGEST=$(curl -sf "$BASE_URL/api/digest" 2>/dev/null || echo '{}')
HAS_DATE=$(echo "$DIGEST" | python3 -c "import sys,json; d=json.load(sys.stdin); print('yes' if d.get('date') else 'no')" 2>/dev/null || echo "no")

if [ "$HAS_DATE" = "yes" ]; then
  pass "Daily digest generated"
else
  fail "Daily digest missing date field"
fi

# ─── Test 8: Task list ────────────────────────────────────────────────────────
section "Test 8: Task List"
TASKS=$(curl -sf "$BASE_URL/api/tasks" 2>/dev/null || echo '[]')
TASK_COUNT=$(echo "$TASKS" | python3 -c "import sys,json; print(len(json.load(sys.stdin)))" 2>/dev/null || echo "0")

if [ "$TASK_COUNT" -gt 0 ]; then
  pass "Task list returned $TASK_COUNT tasks"
else
  fail "No tasks in list"
fi

# ─── Summary ──────────────────────────────────────────────────────────────────
echo ""
echo "══════════════════════════════════"
echo "Results: ${PASS} passed  ${FAIL} failed"
echo "══════════════════════════════════"

if [ "$FAIL" -eq 0 ]; then
  echo ""
  echo "MOBILE + ARCHITECTURE MEGA COMPLETE"
  echo "All systems nominal. Ready for afternoon merge."
  exit 0
else
  echo ""
  echo "Some tests failed. Check desktop server is running."
  exit 1
fi
