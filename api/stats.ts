import { ethers } from "ethers";

const FUJI_RPC = "https://api.avax-test.network/ext/bc/C/rpc";
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
      gasPrice: gasPrice.gasPrice ? ethers.formatUnits(gasPrice.gasPrice, "gwei") : null,
      chainId: Number(network.chainId),
      network: network.name,
    };
  } catch {
    return { blockNumber: null, gasPrice: null, chainId: 43113, network: "fuji" };
  }
}

// Try to fetch proxy metrics. If the proxy isn't running, return zeros.
async function getProxyMetrics() {
  try {
    const res = await fetch("http://127.0.0.1:8545/metrics");
    if (!res.ok) return null;
    return await res.json();
  } catch {
    return null;
  }
}

export default async function handler(_req: any, res: any) {
  const chain = await getChainData();
  const proxy = await getProxyMetrics();

  res.setHeader("Access-Control-Allow-Origin", "*");
  res.json({
    uptime: proxy ? `${Math.floor(proxy.uptime_secs / 3600)}h ${Math.floor((proxy.uptime_secs % 3600) / 60)}m` : "0h 0m",
    processedToday: proxy?.processed ?? 0,
    blockedToday: proxy?.blocked ?? 0,
    blockRatio: proxy && (proxy.processed + proxy.blocked) > 0
      ? ((proxy.blocked / (proxy.processed + proxy.blocked)) * 100).toFixed(2)
      : "0.00",
    chain,
    proxyOnline: proxy !== null,
  });
}
