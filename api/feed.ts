export default function handler(_req: any, res: any) {
  // Transaction feed comes from the proxy audit log.
  // When the proxy is running, query its logs endpoint.
  // For now, return empty — real data only.
  res.setHeader("Access-Control-Allow-Origin", "*");
  res.json({ transactions: [], total: 0 });
}
