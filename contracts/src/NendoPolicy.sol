// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/Ownable.sol";

/// @title NendoPolicy — On-chain policy enforcement for AI agents on Avalanche C-Chain
/// @notice Stores policy rules that the Nendo RPC proxy reads to allow/block transactions
/// @dev Deployed on Avalanche C-Chain. Only the owner can update rules.
contract NendoPolicy is Ownable {

    // ─── Policy State ────────────────────────────────────────────────────

    uint256 public maxPerTx;           // max AVAX per transaction (in wei)
    uint256 public maxDaily;          // max AVAX per 24h rolling window (in wei)
    uint256 public minIntervalSeconds; // min seconds between transactions
    bool public paused;               // circuit breaker

    // Per-agent overrides (agent address → override)
    struct AgentPolicy {
        uint256 maxPerTx;
        uint256 maxDaily;
        uint256 minInterval;
        bool hasOverride;
    }

    mapping(address => AgentPolicy) public agentPolicies;
    mapping(address => uint256) public lastTxTime;
    mapping(address => uint256) public dailySpent;
    mapping(address => uint256) public dailyWindowStart;

    // Allow/block lists
    mapping(address => bool) public allowedContracts;
    mapping(address => bool) public blockedRecipients;
    bool public allowlistMode; // if true, ONLY allowed contracts can be called

    // ─── Events ─────────────────────────────────────────────────────────

    event PolicyUpdated(
        address indexed owner,
        uint256 maxPerTx,
        uint256 maxDaily,
        uint256 minIntervalSeconds
    );

    event AgentPolicySet(address indexed agent, uint256 maxPerTx, uint256 maxDaily);
    event ContractAllowlistUpdated(address indexed contract_, bool allowed);
    event RecipientBlocklistUpdated(address indexed recipient, bool blocked);
    event EmergencyPause(address indexed by, uint256 timestamp);
    event EmergencyResume(address indexed by, uint256 timestamp);

    // ─── Errors ─────────────────────────────────────────────────────────

    error Paused();
    error SenderBlocked();
    error RecipientBlocked();
    error ContractNotAllowed();
    error ExceedsMaxPerTx();
    error ExceedsDailyLimit();
    error RateLimitExceeded();

    // ─── Initialize ────────────────────────────────────────────────────

    constructor() Ownable(msg.sender) {
        maxPerTx = 10 ether;           // 10 AVAX default
        maxDaily = 100 ether;         // 100 AVAX default
        minIntervalSeconds = 5;      // 5 second rate limit
        paused = false;
        allowlistMode = false;
    }

    // ─── Policy Evaluation (read by Nendo RPC proxy off-chain) ─────────

    /// @notice Check if a transaction from `from` to `to` with `value` would pass
    /// @dev This is a view function — no state changes. Called by the Nendo proxy.
    function check(
        address from,
        address to,
        uint256 value
    ) public view returns (bool allowed, string memory reason) {
        if (paused) return (false, "Firewall is paused");

        if (blockedRecipients[to]) return (false, "Recipient is blocklisted");

        AgentPolicy memory agent = agentPolicies[from];
        uint256 effectiveMaxPerTx = agent.hasOverride ? agent.maxPerTx : maxPerTx;

        if (value > effectiveMaxPerTx) {
            return (false, "Exceeds per-transaction cap");
        }

        if (allowlistMode && !allowedContracts[to]) {
            return (false, "Contract not in allowlist");
        }

        // Daily limit check
        uint256 effectiveMaxDaily = agent.hasOverride ? agent.maxDaily : maxDaily;
        _refreshDailyWindow(from);
        if (dailySpent[from] + value > effectiveMaxDaily) {
            return (false, "Exceeds daily spending limit");
        }

        // Rate limit
        uint256 effectiveMinInterval = agent.hasOverride ? agent.minInterval : minIntervalSeconds;
        if (block.timestamp - lastTxTime[from] < effectiveMinInterval) {
            return (false, "Rate limit exceeded");
        }

        return (true, "");
    }

    /// @notice Record a transaction (called by Nendo proxy after forwarding)
    function record(address from, address to, uint256 value) external onlyOwner {
        _refreshDailyWindow(from);
        dailySpent[from] += value;
        lastTxTime[from] = block.timestamp;
    }

    // ─── Admin Setters ──────────────────────────────────────────────────

    function setGlobalPolicy(
        uint256 _maxPerTx,
        uint256 _maxDaily,
        uint256 _minIntervalSeconds
    ) external onlyOwner {
        maxPerTx = _maxPerTx;
        maxDaily = _maxDaily;
        minIntervalSeconds = _minIntervalSeconds;
        emit PolicyUpdated(msg.sender, _maxPerTx, _maxDaily, _minIntervalSeconds);
    }

    function setAgentPolicy(
        address agent,
        uint256 _maxPerTx,
        uint256 _maxDaily
    ) external onlyOwner {
        agentPolicies[agent] = AgentPolicy({
            maxPerTx: _maxPerTx,
            maxDaily: _maxDaily,
            minInterval: 0,
            hasOverride: true
        });
        emit AgentPolicySet(agent, _maxPerTx, _maxDaily);
    }

    function setAllowedContract(address contract_, bool allowed) external onlyOwner {
        allowedContracts[contract_] = allowed;
        emit ContractAllowlistUpdated(contract_, allowed);
    }

    function setBlockedRecipient(address recipient, bool blocked) external onlyOwner {
        blockedRecipients[recipient] = blocked;
        emit RecipientBlocklistUpdated(recipient, blocked);
    }

    function setAllowlistMode(bool enabled) external onlyOwner {
        allowlistMode = enabled;
    }

    function pause() external onlyOwner {
        paused = true;
        emit EmergencyPause(msg.sender, block.timestamp);
    }

    function unpause() external onlyOwner {
        paused = false;
        emit EmergencyResume(msg.sender, block.timestamp);
    }

    // ─── Internal ────────────────────────────────────────────────────────

    function _refreshDailyWindow(address agent) internal {
        uint256 dayStart = (block.timestamp / 1 days) * 1 days;
        if (dailyWindowStart[agent] < dayStart) {
            dailySpent[agent] = 0;
            dailyWindowStart[agent] = dayStart;
        }
    }
}