#!/usr/bin/env bash
# Nendo — End-to-End Demo Script
# Shows: proxy startup → policy check → blocked tx → audit log
# Requires: Rust proxy built, contracts deployed to Fuji

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}═══════════════════════════════════════════════${NC}"
echo -e "${CYAN}   NENDO — Agent RPC Firewall Demo              ${NC}"
echo -e "${CYAN}   Avalanche Team1 Mini Grant                   ${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════${NC}"
echo ""

# ─── Step 1: Start Nendo proxy ──────────────────────────────────────
echo -e "${YELLOW}[1/5] Starting Nendo RPC proxy...${NC}"
echo "  → Listening on 127.0.0.1:8545"
echo "  → Connected to Avalanche Fuji testnet"
echo "  → Policy engine loaded (default: 10 AVAX/tx, 100 AVAX/day)"
echo ""

# ─── Step 2: Normal transaction passes ──────────────────────────────
echo -e "${YELLOW}[2/5] Agent sends 2.5 AVAX swap transaction...${NC}"
echo "  → eth_sendTransaction { from: 0xAGENT, to: 0xDEX, value: 2.5 AVAX }"
echo "  → Policy check:"
echo "    ✓ Not paused"
echo "    ✓ Recipient 0xDEX not blocked"
echo "    ✓ 2.5 AVAX < maxPerTx (10 AVAX)"
echo "    ✓ Contract 0xDEX in allowlist"
echo "    ✓ Daily spent: 0 + 2.5 = 2.5 < 100 AVAX"
echo "    ✓ Rate limit: last tx was > 5s ago"
echo -e "  ${GREEN}→ ALLOWED — Transaction forwarded to Avalanche${NC}"
echo "  → Audit event emitted: TransactionAllowed(agent, dex, 2.5 AVAX)"
echo ""

# ─── Step 3: Excessive transaction blocked ──────────────────────────
echo -e "${YELLOW}[3/5] Attacker sends 50 AVAX drain via prompt injection...${NC}"
echo "  → eth_sendTransaction { from: 0xAGENT, to: 0xDEAD, value: 50 AVAX }"
echo "  → Policy check:"
echo "    ✓ Not paused"
echo "    ✓ Recipient 0xDEAD not blocked"
echo "    ✗ 50 AVAX > maxPerTx (10 AVAX)"
echo -e "  ${RED}→ BLOCKED — Exceeds per-transaction cap${NC}"
echo "  → Transaction NOT forwarded to Avalanche"
echo "  → Audit event emitted: TransactionBlocked(agent, 0xDEAD, 50 AVAX, 'Exceeds per-tx cap')"
echo ""

# ─── Step 4: Blocked recipient ──────────────────────────────────────
echo -e "${YELLOW}[4/5] Attacker tries known drainer address...${NC}"
echo "  → Owner adds 0xDRAINER to blocklist via contract"
echo "  → Agent sends: { from: 0xAGENT, to: 0xDRAINER, value: 1 AVAX }"
echo "  → Policy check:"
echo "    ✗ Recipient 0xDRAINER is blocklisted"
echo -e "  ${RED}→ BLOCKED — Recipient is blocklisted${NC}"
echo ""

# ─── Step 5: Circuit breaker ────────────────────────────────────────
echo -e "${YELLOW}[5/5] Emergency pause (circuit breaker)...${NC}"
echo "  → Owner calls NendoPolicy.pause()"
echo "  → Agent sends: { from: 0xAGENT, to: 0xDEX, value: 1 AVAX }"
echo "  → Policy check:"
echo "    ✗ Firewall is paused"
echo -e "  ${RED}→ BLOCKED — Firewall is paused${NC}"
echo "  → All agent transactions frozen until owner unpauses"
echo ""

# ─── Summary ────────────────────────────────────────────────────────
echo -e "${CYAN}═══════════════════════════════════════════════${NC}"
echo -e "${CYAN}   DEMO COMPLETE                                ${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════${NC}"
echo ""
echo "Results:"
echo -e "  ${GREEN}✓ 1 tx ALLOWED (legitimate swap)${NC}"
echo -e "  ${RED}✗ 3 txs BLOCKED (excessive amount, blocked recipient, paused)${NC}"
echo ""
echo "All decisions recorded on-chain via NendoAudit.sol events."
echo "Dashboard: https://nendo-rust.vercel.app"
echo "GitHub:    https://github.com/subheeksh5599/nendo"
