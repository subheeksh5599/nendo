import { ethers } from "ethers";

const FUJI_RPC = "https://api.avax-test.network/ext/bc/C/rpc";
const provider = new ethers.JsonRpcProvider(FUJI_RPC);

export default async function handler(_req: any, res: any) {
  let blockNumber: number | null = null;
  let chainId = 43113;
  try {
    blockNumber = await provider.getBlockNumber();
    const network = await provider.getNetwork();
    chainId = Number(network.chainId);
  } catch {}

  res.setHeader("Access-Control-Allow-Origin", "*");
  res.json({
    status: "ok",
    uptime: Math.floor(process.uptime()),
    chain: chainId,
    blockNumber,
  });
}
