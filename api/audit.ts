export default function handler(_req: any, res: any) {
  // Audit log comes from the proxy sled DB.
  // When the proxy is running, query its logs endpoint.
  // For now, return empty — real data only.
  res.setHeader("Access-Control-Allow-Origin", "*");
  res.json({ entries: [], total: 0 });
}
