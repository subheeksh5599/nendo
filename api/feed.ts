export default function handler(_req: any, res: any) {
  const now = new Date();
  const pad = (n: number) => String(n).padStart(2, "0");

  const transactions = [
    { time: `${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds() - 3)}`, agent: "0x7A3b…E91C", recipient: "0x4c2E…a119", amount: "0.2500 AVAX", decision: "ALLOWED", policy: "all checks passed", hash: "0x84fde103a26e4c8d5b7f9061728394a5b2c3d4e5" },
    { time: `${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds() - 20)}`, agent: "0x1d60…87B2", recipient: "0xB82a…19eF", amount: "1.7500 AVAX", decision: "BLOCKED", policy: "per_tx_limit", hash: null },
    { time: `${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds() - 35)}`, agent: "0x7A3b…E91C", recipient: "0x047d…F40A", amount: "0.0180 AVAX", decision: "ALLOWED", policy: "all checks passed", hash: "0x3edca5106f7e819203a4b5c6d7e8f901a2b3" },
    { time: `${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds() - 50)}`, agent: "0xfE42…C753", recipient: "0xC954…2B8D", amount: "0.8200 AVAX", decision: "BLOCKED", policy: "recipient_blocklist", hash: null },
  ];

  res.setHeader("Access-Control-Allow-Origin", "*");
  res.json({ transactions, total: 1284 + 17 });
}
