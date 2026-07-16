import { ethers } from "ethers";

const FUJI_RPC = "https://api.avax-test.network/ext/bc/C/rpc";
const NENDO_POLICY = "0xe3c5541F125a00C578FEA78ad0395473eC3D1386";

const POLICY_ABI = [
  "function maxPerTx() view returns (uint256)",
  "function maxDaily() view returns (uint256)",
  "function minIntervalSeconds() view returns (uint256)",
  "function paused() view returns (bool)",
  "function allowlistMode() view returns (bool)",
  "event PolicyUpdated(address indexed owner, uint256 maxPerTx, uint256 maxDaily, uint256 minIntervalSeconds)",
];

export default async function handler(_req: any, res: any) {
  try {
    const provider = new ethers.JsonRpcProvider(FUJI_RPC);
    const policy = new ethers.Contract(NENDO_POLICY, POLICY_ABI, provider);

    const [maxPerTx, maxDaily, minInterval, paused, allowlistMode] = await Promise.all([
      policy.maxPerTx(),
      policy.maxDaily(),
      policy.minIntervalSeconds(),
      policy.paused(),
      policy.allowlistMode(),
    ]);

    // Get last PolicyUpdated event for timing
    const currentBlock = await provider.getBlockNumber();
    const events = await policy.queryFilter("PolicyUpdated", currentBlock - 5000, currentBlock);
    const lastUpdated = events.length > 0
      ? new Date((await events[events.length - 1].getBlock()).timestamp * 1000).toISOString()
      : new Date().toISOString();

    res.setHeader("Access-Control-Allow-Origin", "*");
    res.setHeader("Cache-Control", "public, max-age=10, s-maxage=10");
    res.json({
      maxPerTx: ethers.formatEther(maxPerTx) + " AVAX",
      maxDaily: ethers.formatEther(maxDaily) + " AVAX",
      minInterval: minInterval.toString() + "s",
      circuitBreaker: paused,
      allowlistMode,
      contract: NENDO_POLICY,
      auditContract: "0x932c1A3df8a93b46f72A9C862fC0F580650b8701",
      lastUpdated,
    });
  } catch (e: any) {
    res.setHeader("Access-Control-Allow-Origin", "*");
    res.json({ error: e.message });
  }
}
