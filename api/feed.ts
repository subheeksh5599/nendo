import { ethers } from "ethers";

const FUJI_RPC = "https://api.avax-test.network/ext/bc/C/rpc";
const NENDO_AUDIT = "0xdA9721d1D0706fa0F0A49a35Cbf45Bd95D60cEB7";

const AUDIT_ABI = [
  "event TransactionAllowed(address indexed agent, address indexed recipient, uint256 amount, bytes32 indexed intentHash, uint256 timestamp)",
  "event TransactionBlocked(address indexed agent, address indexed recipient, uint256 amount, string reason, uint256 timestamp)",
];

function fmt(addr: string) {
  return addr.slice(0, 6) + "..." + addr.slice(-4);
}

export default async function handler(_req: any, res: any) {
  try {
    const provider = new ethers.JsonRpcProvider(FUJI_RPC);
    const audit = new ethers.Contract(NENDO_AUDIT, AUDIT_ABI, provider);

    // Fetch last 20 blocks worth of events (~20 seconds on Fuji)
    const currentBlock = await provider.getBlockNumber();
    const fromBlock = Math.max(currentBlock - 500, 0);

    const [allowedLogs, blockedLogs] = await Promise.all([
      audit.queryFilter("TransactionAllowed", fromBlock, currentBlock),
      audit.queryFilter("TransactionBlocked", fromBlock, currentBlock),
    ]);

    const items: any[] = [];

    for (const log of allowedLogs) {
      const parsed = audit.interface.parseLog({ topics: [...log.topics], data: log.data });
      if (!parsed) continue;
      items.push({
        time: new Date(Number(parsed.args.timestamp) * 1000).toISOString(),
        agent: fmt(parsed.args.agent),
        recipient: fmt(parsed.args.recipient),
        amount: ethers.formatEther(parsed.args.amount) + " AVAX",
        decision: "ALLOWED",
        policy: "Policy passed",
        hash: log.transactionHash,
      });
    }

    for (const log of blockedLogs) {
      const parsed = audit.interface.parseLog({ topics: [...log.topics], data: log.data });
      if (!parsed) continue;
      items.push({
        time: new Date(Number(parsed.args.timestamp) * 1000).toISOString(),
        agent: fmt(parsed.args.agent),
        recipient: fmt(parsed.args.recipient),
        amount: ethers.formatEther(parsed.args.amount) + " AVAX",
        decision: "BLOCKED",
        policy: parsed.args.reason,
        hash: log.transactionHash,
      });
    }

    // Sort by most recent first
    items.sort((a, b) => new Date(b.time).getTime() - new Date(a.time).getTime());

    res.setHeader("Access-Control-Allow-Origin", "*");
    res.setHeader("Cache-Control", "public, max-age=5, s-maxage=5");
    res.json({ transactions: items.slice(0, 50), total: items.length });
  } catch (e: any) {
    res.setHeader("Access-Control-Allow-Origin", "*");
    res.json({ transactions: [], total: 0, error: e.message });
  }
}
