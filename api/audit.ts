export default function handler(_req: any, res: any) {
  const now = new Date();
  const pad = (n: number) => String(n).padStart(2, "0");

  const entries = [
    { type: "BLOCKED", agent: "0x1d60…87B2", recipient: "0xB82a…19eF", reason: "per_tx_limit exceeded (1.75 AVAX > 1.0 AVAX cap)", time: `${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds() - 20)}`, hash: null },
    { type: "ALLOWED", agent: "0x7A3b…E91C", recipient: "0x047d…F40A", reason: "all checks passed · gas: 21,000 · sim OK", time: `${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds() - 35)}`, hash: "0x3edca5106f7e819203a4b5c6d7e8f901a2b3" },
    { type: "BLOCKED", agent: "0xfE42…C753", recipient: "0xC954…2B8D", reason: "recipient_blocklist (0xC954…2B8D is blocked)", time: `${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds() - 50)}`, hash: null },
  ];

  res.setHeader("Access-Control-Allow-Origin", "*");
  res.json({ entries, total: entries.length + 1298 });
}
