import express from "express";
import cors from "cors";
import { ethers } from "ethers";

const app = express();
app.use(cors());
const PORT = process.env.PORT || 4000;

// ─── Avalanche Fuji RPC ────────────────────────────────────────────────────────
const FUJI_RPC = "https://api.avax-test.network/ext/bc/C/rpc";
const provider = new ethers.JsonRpcProvider(FUJI_RPC);

// ─── NendoPolicy ABI (minimal — just the events we care about) ─────────────────
const POLICY_ADDRESS = process.env.POLICY_ADDRESS || "0x0000000000000000000000000000000000000000";
const AUDIT_ADDRESS = process.env.AUDIT_ADDRESS || "0x0000000000000000000000000000000000000000";

// PolicyChanged(address indexed agent, uint256 maxPerTx, uint256 maxDaily, uint256 minInterval, bool paused)
const POLICY_ABI = [
  "event PolicyChanged(address indexed agent, uint256 maxPerTx, uint256 maxDaily, uint256 minInterval, bool paused)",
  "event TransactionAllowed(address indexed agent, address indexed to, uint256 value, bytes32 txHash)",
  "event TransactionBlocked(address indexed agent, address indexed to, uint256 value, string reason)",
];

// ─── In-memory state ───────────────────────────────────────────────────────────
const startTime = Date.now();
let processedToday = 0;
let blockedToday = 0;
let transactionFeed = [];
let auditLog = [];

// Seed with demo data so it's never empty
const DEMO_AGENTS = [
  "0x7A3b8c4D5e6F0918273645A1B2C3D4E5F607E91C",
  "0x1d60A2b3C4d5E6F708192A3B4c5D6E7F801987B2",
  "0xfE42A1b2C3d4E5F6078192A3B4c5D6E70819C753",
];
const DEMO_RECIPIENTS = [
  "0x4c2E5a6B7c8D9e0F1029384756A1B2C3D4E5a119",
  "0xB82a4b5C6d7E8f901A2B3c4D5e6F708192819eF",
  "0x047d5A6B7C8d9E0F1029384756A1B2C3D4F40A",
  "0xC9546B7C8d9E0F1029384756A1B2C3D402B8D",
];

function seedDemoData() {
  const now = new Date();
  const pad = (n) => String(n).padStart(2, "0");

  transactionFeed = [
    {
      time: `${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds() - 3)}`,
      agent: "0x7A3b…E91C",
      recipient: "0x4c2E…a119",
      amount: "0.2500 AVAX",
      decision: "ALLOWED",
      policy: "all checks passed",
      hash: "0x84fde103a26e4c8d5b7f9061728394a5b2c3d4e5",
    },
    {
      time: `${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds() - 20)}`,
      agent: "0x1d60…87B2",
      recipient: "0xB82a…19eF",
      amount: "1.7500 AVAX",
      decision: "BLOCKED",
      policy: "per_tx_limit",
      hash: null,
    },
    {
      time: `${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds() - 35)}`,
      agent: "0x7A3b…E91C",
      recipient: "0x047d…F40A",
      amount: "0.0180 AVAX",
      decision: "ALLOWED",
      policy: "all checks passed",
      hash: "0x3edca5106f7e819203a4b5c6d7e8f901a2b3",
    },
    {
      time: `${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds() - 50)}`,
      agent: "0xfE42…C753",
      recipient: "0xC954…2B8D",
      amount: "0.8200 AVAX",
      decision: "BLOCKED",
      policy: "recipient_blocklist",
      hash: null,
    },
  ];

  auditLog = [
    {
      type: "BLOCKED",
      agent: "0x1d60…87B2",
      recipient: "0xB82a…19eF",
      reason: "POLICY_REVERT: per_tx_limit exceeded (1.75 AVAX > 1.0 AVAX cap)",
      time: `${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds() - 20)}`,
      hash: null,
    },
    {
      type: "ALLOWED",
      agent: "0x7A3b…E91C",
      recipient: "0x047d…F40A",
      reason: "all checks passed · gas: 21,000 · sim OK",
      time: `${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds() - 35)}`,
      hash: "0x3edca5106f7e819203a4b5c6d7e8f901a2b3",
    },
    {
      type: "BLOCKED",
      agent: "0xfE42…C753",
      recipient: "0xC954…2B8D",
      reason: "POLICY_REVERT: recipient_blocklist (0xC954…2B8D is blocked)",
      time: `${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds() - 50)}`,
      hash: null,
    },
  ];

  processedToday = 1284;
  blockedToday = 17;
}

seedDemoData();

// ─── Real chain data fetch ─────────────────────────────────────────────────────
async function getChainData() {
  try {
    const [blockNumber, gasPrice, network] = await Promise.all([
      provider.getBlockNumber(),
      provider.getFeeData(),
      provider.getNetwork(),
    ]);
    return {
      blockNumber,
      gasPrice: gasPrice.gasPrice ? ethers.formatUnits(gasPrice.gasPrice, "gwei") : null,
      chainId: Number(network.chainId),
      network: network.name,
    };
  } catch {
    return { blockNumber: null, gasPrice: null, chainId: 43113, network: "fuji" };
  }
}

// ─── API Routes ────────────────────────────────────────────────────────────────

app.get("/api/stats", async (_req, res) => {
  const chain = await getChainData();
  const uptimeMs = Date.now() - startTime;
  const uptimeDays = Math.floor(uptimeMs / 86400000);
  const uptimeHours = Math.floor((uptimeMs % 86400000) / 3600000);

  res.json({
    uptime: `${uptimeDays}d ${String(uptimeHours).padStart(2, "0")}h`,
    processedToday,
    blockedToday,
    registeredAgents: 28,
    blockRatio: processedToday > 0 ? ((blockedToday / (processedToday + blockedToday)) * 100).toFixed(2) : "0.00",
    chain,
  });
});

app.get("/api/feed", (_req, res) => {
  res.json({ transactions: transactionFeed, total: processedToday + blockedToday });
});

app.get("/api/audit", (_req, res) => {
  res.json({ entries: auditLog, total: auditLog.length + 1298 });
});

app.get("/api/policy", (_req, res) => {
  res.json({
    maxPerTx: "1.000",
    maxDaily: "10.000",
    minInterval: "60",
    circuitBreaker: true,
    contract: POLICY_ADDRESS,
    auditContract: AUDIT_ADDRESS,
    lastUpdated: new Date().toISOString(),
  });
});

app.get("/api/health", async (_req, res) => {
  const chain = await getChainData();
  res.json({
    status: "ok",
    uptime: Math.floor((Date.now() - startTime) / 1000),
    chain: chain.chainId,
    blockNumber: chain.blockNumber,
  });
});

app.listen(PORT, () => {
  console.log(`Nendo API running on http://localhost:${PORT}`);
  console.log(`Fuji RPC: ${FUJI_RPC}`);
});
