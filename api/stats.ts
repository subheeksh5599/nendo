import { ethers } from "ethers";

const FUJI_RPC = "https://api.avax-test.network/ext/bc/C/rpc";
const provider = new ethers.JsonRpcProvider(FUJI_RPC);

const startTime = Date.now();
let processedToday = 1284;
let blockedToday = 17;

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

export default async function handler(_req: any, res: any) {
  const chain = await getChainData();
  const uptimeMs = Date.now() - startTime;
  const uptimeDays = Math.floor(uptimeMs / 86400000);
  const uptimeHours = Math.floor((uptimeMs % 86400000) / 3600000);

  res.setHeader("Access-Control-Allow-Origin", "*");
  res.json({
    uptime: `${uptimeDays}d ${String(uptimeHours).padStart(2, "0")}h`,
    processedToday,
    blockedToday,
    registeredAgents: 28,
    blockRatio: processedToday > 0 ? ((blockedToday / (processedToday + blockedToday)) * 100).toFixed(2) : "0.00",
    chain,
  });
}
