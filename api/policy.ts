export default function handler(_req: any, res: any) {
  res.setHeader("Access-Control-Allow-Origin", "*");
  res.json({
    maxPerTx: "1.000",
    maxDaily: "10.000",
    minInterval: "60",
    circuitBreaker: true,
    contract: "0x0000000000000000000000000000000000000000",
    auditContract: "0x0000000000000000000000000000000000000000",
    lastUpdated: new Date().toISOString(),
  });
}
