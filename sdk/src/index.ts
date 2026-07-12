import { ethers } from 'ethers';

/**
 * NendoClient — TypeScript SDK for Nendo Agent RPC Firewall on Avalanche
 *
 * Connects to the Nendo RPC proxy (http://localhost:8545 by default)
 * to configure policies and fetch audit logs.
 */

export interface PolicyConfig {
  maxPerTx: string;           // in wei, e.g. "10000000000000000000" = 10 AVAX
  maxDaily: string;          // in wei
  minIntervalSeconds: number;
  allowedContracts: string[];
  blockedRecipients: string[];
}

export interface SimulationResult {
  allowed: boolean;
  reason?: string;
  rule?: string;
  netBalanceChange: string;
  gasUsed: number;
}

export interface AuditEntry {
  entryType: 'allowed' | 'blocked';
  from: string;
  to: string;
  value: string;
  reason?: string;
  timestamp: number;
}

export class NendoUtils {
  /**
   * Convert AVAX to wei string
   * e.g. 10 AVAX → "0xde0b6b3a7640000" (but as proper bigint)
   */
  avaxToWei(avax: string | number): string {
    const avaxNum = typeof avax === 'string' ? parseFloat(avax) : avax;
    const wei = BigInt(Math.floor(avaxNum * 1e18));
    return '0x' + wei.toString(16);
  }

  weiToAvax(wei: string): string {
    const weiBig = BigInt(wei);
    const avax = Number(weiBig) / 1e18;
    return avax.toFixed(6);
  }
}

export class NendoClient {
  private rpcUrl: string;
  private policyAddress: string;
  private auditAddress: string;
  private provider: ethers.JsonRpcProvider;

  constructor(
    rpcUrl: string = 'http://localhost:8545',
    policyAddress: string,
    auditAddress: string
  ) {
    this.rpcUrl = rpcUrl;
    this.policyAddress = policyAddress;
    this.auditAddress = auditAddress;
    this.provider = new ethers.JsonRpcProvider(rpcUrl);
  }

  utils = new NendoUtils();

  /**
   * Set the policy for an agent (requires owner wallet)
   */
  async setPolicy(policy: PolicyConfig, wallet: ethers.Wallet): Promise<void> {
    const contract = new ethers.Contract(
      this.policyAddress,
      [
        'function setAgentPolicy(address agent, uint256 maxPerTx, uint256 maxDaily)',
        'function setAllowedContract(address contract_, bool allowed)',
        'function setBlockedRecipient(address recipient, bool blocked)',
        'function setGlobalPolicy(uint256 maxPerTx, uint256 maxDaily, uint256 minIntervalSeconds)',
      ],
      wallet
    );

    const tx1 = await contract.setGlobalPolicy(
      policy.maxPerTx,
      policy.maxDaily,
      policy.minIntervalSeconds
    );
    await tx1.wait();
  }

  /**
   * Simulate a transaction — checks policy without sending
   */
  async simulate(params: {
    from: string;
    to: string;
    value: string;
    data?: string;
  }): Promise<SimulationResult> {
    // Call the policy contract directly
    const contract = new ethers.Contract(
      this.policyAddress,
      ['function check(address from, address to, uint256 value) view returns (bool allowed, string memory reason)'],
      this.provider
    );

    const [allowed, reason] = await contract.check(params.from, params.to, params.value);
    return {
      allowed,
      reason: reason || undefined,
      netBalanceChange: params.value,
      gasUsed: 21000,
    };
  }

  /**
   * Pause the firewall (emergency circuit breaker)
   */
  async emergencyPause(wallet: ethers.Wallet): Promise<void> {
    const contract = new ethers.Contract(
      this.policyAddress,
      ['function pause()'],
      wallet
    );
    const tx = await contract.pause();
    await tx.wait();
  }

  /**
   * Get recent audit log entries (from local sled DB via proxy API)
   */
  async getAuditLogs(limit: number = 50): Promise<AuditEntry[]> {
    const resp = await fetch(`${this.rpcUrl}/logs?limit=${limit}`);
    if (!resp.ok) return [];
    return resp.json();
  }
}