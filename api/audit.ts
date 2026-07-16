import { ethers } from "ethers";

const FUJI_RPC = "https://api.avax-test.network/ext/bc/C/rpc";
const NENDO_AUDIT = "0xdA9721d1D0706fa0F0A49a35Cbf45Bd95D60cEB7";

const AUDIT_ABI = [
  "event TransactionAllowed(address indexed agent, address indexed recipient, uint256 amount, bytes32 indexed intentHash, uint256 timestamp)",
  "event TransactionBlocked(address indexed agent, address indexed recipient, uint256 amount, string reason, uint256 timestamp)",
  "event AgentRegistered(address indexed agent, string name, uint256 timestamp)",
  "event EmergencyPause(address indexed by, uint256 timestamp)",
];

function fmt(addr: string) {
  return addr.slice(0, 6) + "..." + addr.slice(-4);
}

export default async function handler(_req: any, res: any) {
  try {
    const provider = new ethers.JsonRpcProvider(FUJI_RPC);
    const audit = new ethers.Contract(NENDO_AUDIT, AUDIT_ABI, provider);

    // Last 2000 blocks
    const currentBlock = await provider.getBlockNumber();
    const fromBlock = Math.max(currentBlock - 2000, 0);

    const [allowedLogs, blockedLogs, registeredLogs, pauseLogs] = await Promise.all([
      audit.queryFilter("TransactionAllowed", fromBlock, currentBlock),
      audit.queryFilter("TransactionBlocked", fromBlock, currentBlock),
      audit.queryFilter("AgentRegistered", fromBlock, currentBlock),
      audit.queryFilter("EmergencyPause", fromBlock, currentBlock),
    ]);

    const entries: any[] = [];

    for (const log of allowedLogs) {
      const parsed = audit.interface.parseLog({ topics: [...log.topics], data: log.data });
      if (!parsed) continue;
      entries.push({
        type: "ALLOWED",
        agent: parsed.args.agent,
        recipient: parsed.args.recipient,
        reason: `Allowed: ${ethers.formatEther(parsed.args.amount)} AVAX`,
        time: new Date(Number(parsed.args.timestamp) * 1000).toISOString(),
        hash: log.transactionHash,
      });
    }

    for (const log of blockedLogs) {
      const parsed = audit.interface.parseLog({ topics: [...log.topics], data: log.data });
      if (!parsed) continue;
      entries.push({
        type: "BLOCKED",
        agent: parsed.args.agent,
        recipient: parsed.args.recipient,
        reason: `Blocked: ${ethers.formatEther(parsed.args.amount)} AVAX — ${parsed.args.reason}`,
        time: new Date(Number(parsed.args.timestamp) * 1000).toISOString(),
        hash: log.transactionHash,
      });
    }

    for (const log of registeredLogs) {
      const parsed = audit.interface.parseLog({ topics: [...log.topics], data: log.data });
      if (!parsed) continue;
      entries.push({
        type: "ALLOWED",
        agent: parsed.args.agent,
        recipient: "",
        reason: `Agent registered: ${parsed.args.name}`,
        time: new Date(Number(parsed.args.timestamp) * 1000).toISOString(),
        hash: log.transactionHash,
      });
    }

    for (const log of pauseLogs) {
      const parsed = audit.interface.parseLog({ topics: [...log.topics], data: log.data });
      if (!parsed) continue;
      entries.push({
        type: "BLOCKED",
        agent: parsed.args.by,
        recipient: "",
        reason: "Emergency pause activated",
        time: new Date(Number(parsed.args.timestamp) * 1000).toISOString(),
        hash: log.transactionHash,
      });
    }

    entries.sort((a, b) => new Date(b.time).getTime() - new Date(a.time).getTime());

    const items = entries.slice(0, 200);

    res.setHeader("Access-Control-Allow-Origin", "*");
    res.setHeader("Cache-Control", "public, max-age=10, s-maxage=10");
    res.json({ entries: items, total: items.length });
  } catch (e: any) {
    res.setHeader("Access-Control-Allow-Origin", "*");
    res.json({ entries: [], total: 0, error: e.message });
  }
}
