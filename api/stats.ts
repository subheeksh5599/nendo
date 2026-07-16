import { ethers } from "ethers";

const FUJI_RPC = "https://api.avax-test.network/ext/bc/C/rpc";
const NENDO_POLICY = "0xe3c5541F125a00C578FEA78ad0395473eC3D1386";
const NENDO_AUDIT = "0x932c1A3df8a93b46f72A9C862fC0F580650b8701";

const AUDIT_ABI = [
  "event TransactionAllowed(address indexed agent, address indexed recipient, uint256 amount, bytes32 indexed intentHash, uint256 timestamp)",
  "event TransactionBlocked(address indexed agent, address indexed recipient, uint256 amount, string reason, uint256 timestamp)",
];

const POLICY_ABI = [
  "function paused() view returns (bool)",
];

const provider = new ethers.JsonRpcProvider(FUJI_RPC);

async function getChainData() {
  try {
    const [blockNumber, gasPrice, network] = await Promise.all([
      provider.getBlockNumber(),
      provider.getFeeData(),
      provider.getNetwork(),
    ]);
    return {
      blockNumber,
      gasPrice: gasPrice.gasPrice ? parseFloat(ethers.formatUnits(gasPrice.gasPrice, "gwei")).toFixed(1) : "1.0",
      chainId: Number(network.chainId),
      network: network.name,
    };
  } catch {
    return { blockNumber: null, gasPrice: "1.0", chainId: 43113, network: "fuji" };
  }
}

// Try to fetch proxy metrics. If the proxy isn't running, fall back to on-chain data.
async function getProxyMetrics() {
  try {
    const res = await fetch("http://127.0.0.1:8545/metrics");
    if (!res.ok) return null;
    return await res.json();
  } catch {
    return null;
  }
}

async function getOnChainActivity() {
  try {
    const currentBlock = await provider.getBlockNumber();
    const fromBlock = Math.max(currentBlock - 500, 0);
    const audit = new ethers.Contract(NENDO_AUDIT, AUDIT_ABI, provider);

    const [allowed, blocked] = await Promise.all([
      audit.queryFilter("TransactionAllowed", fromBlock, currentBlock),
      audit.queryFilter("TransactionBlocked", fromBlock, currentBlock),
    ]);

    return {
      allowed: allowed.length,
      blocked: blocked.length,
    };
  } catch {
    return { allowed: 0, blocked: 0 };
  }
}

async function getPausedState() {
  try {
    const policy = new ethers.Contract(NENDO_POLICY, POLICY_ABI, provider);
    return await policy.paused();
  } catch {
    return false;
  }
}

export default async function handler(_req: any, res: any) {
  const [chain, proxy, onChain, paused] = await Promise.all([
    getChainData(),
    getProxyMetrics(),
    getOnChainActivity(),
    getPausedState(),
  ]);

  const processed = proxy?.processed ?? onChain.allowed;
  const blocked = proxy?.blocked ?? onChain.blocked;
  const total = processed + blocked;
  const blockRatio = total > 0 ? ((blocked / total) * 100).toFixed(2) : "0.00";

  res.setHeader("Access-Control-Allow-Origin", "*");
  res.setHeader("Cache-Control", "public, max-age=5, s-maxage=5");
  res.json({
    uptime: proxy ? `${Math.floor(proxy.uptime_secs / 3600)}h ${Math.floor((proxy.uptime_secs % 3600) / 60)}m` : "0h 0m",
    processedToday: processed,
    blockedToday: blocked,
    blockRatio,
    chain,
    proxyOnline: proxy !== null,
    paused,
    policyContract: NENDO_POLICY,
    auditContract: NENDO_AUDIT,
  });
}
