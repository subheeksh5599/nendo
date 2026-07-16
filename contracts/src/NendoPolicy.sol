// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";

/// @title NendoPolicy — On-chain policy enforcement for AI agents on Avalanche C-Chain
/// @notice Stores policy rules that the Nendo RPC proxy reads to allow/block transactions.
///         Supports native AVAX, ICM cross-subnet intents, and Avalanche Payments Collective stablecoins.
/// @dev Deployed on Avalanche C-Chain. Only the owner can update rules.
contract NendoPolicy is Ownable {

    // ═══════════════════════════════════════════════════════════════════
    // 1. NATIVE AVAX POLICY
    // ═══════════════════════════════════════════════════════════════════

    uint256 public maxPerTx;           // max AVAX per transaction (in wei)
    uint256 public maxDaily;          // max AVAX per 24h rolling window (in wei)
    uint256 public minIntervalSeconds; // min seconds between transactions
    bool public paused;               // circuit breaker

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

    mapping(address => bool) public allowedContracts;
    mapping(address => bool) public blockedRecipients;
    bool public allowlistMode;

    // ═══════════════════════════════════════════════════════════════════
    // 2. ICM CROSS-SUBNET POLICY
    // ═══════════════════════════════════════════════════════════════════

    /// @dev Avalanche subnet IDs are 32-byte hashes (ICM blockchainID)
    mapping(bytes32 => bool) public trustedSubnets;
    mapping(bytes32 => string) public subnetLabels;   // human-readable label

    uint256 public maxCrossSubnetAmount;     // max AVAX per cross-subnet transfer
    uint256 public requiredConfirmations;     // blocks to wait before forwarding ICM intent

    // Per-subnet caps (override the global maxCrossSubnetAmount)
    mapping(bytes32 => uint256) public subnetMaxAmount;

    // ═══════════════════════════════════════════════════════════════════
    // 3. STABLECOIN / ERC-20 POLICY (Avalanche Payments Collective)
    // ═══════════════════════════════════════════════════════════════════

    /// @dev Supported stablecoins on Avalanche C-Chain
    ///      Fuji testnet: USDC = 0x5425890298aed601595a70AB815c96711a31Bc65 (example)
    ///      Mainnet: USDC = 0xB97EF9Ef8734C71904D8002F8b6Bc66Dd9c48a6E
    ///               USDT = 0x9702230A8Ea53601f5cD2dc00fDBc13d4dF4A8c7
    mapping(address => bool) public supportedStablecoins;
    mapping(address => string) public stablecoinSymbols;  // e.g. "USDC", "USDT"
    mapping(address => uint8) public stablecoinDecimals;  // cached decimals

    // Per-stablecoin caps (in token base units)
    mapping(address => uint256) public stablecoinMaxPerTx;
    mapping(address => uint256) public stablecoinMaxDaily;
    // Per-agent + per-stablecoin daily spent
    mapping(address => mapping(address => uint256)) public stablecoinDailySpent;

    // ─── Events ─────────────────────────────────────────────────────────

    event PolicyUpdated(address indexed owner, uint256 maxPerTx, uint256 maxDaily, uint256 minIntervalSeconds);
    event AgentPolicySet(address indexed agent, uint256 maxPerTx, uint256 maxDaily);
    event ContractAllowlistUpdated(address indexed contract_, bool allowed);
    event RecipientBlocklistUpdated(address indexed recipient, bool blocked);
    event EmergencyPause(address indexed by, uint256 timestamp);
    event EmergencyResume(address indexed by, uint256 timestamp);

    // ICM events
    event SubnetTrustUpdated(bytes32 indexed subnetId, string label, bool trusted);
    event CrossSubnetPolicyUpdated(uint256 maxAmount, uint256 confirmations);
    event SubnetCapSet(bytes32 indexed subnetId, uint256 maxAmount);

    // Stablecoin events
    event StablecoinRegistered(address indexed token, string symbol, uint8 decimals);
    event StablecoinRemoved(address indexed token);
    event StablecoinPolicyUpdated(address indexed token, uint256 maxPerTx, uint256 maxDaily);

    // ─── Errors ─────────────────────────────────────────────────────────

    error Paused();
    error RecipientBlocked();
    error ContractNotAllowed();
    error ExceedsMaxPerTx();
    error ExceedsDailyLimit();
    error RateLimitExceeded();
    error SubnetNotTrusted();
    error ExceedsCrossSubnetCap();
    error StablecoinNotSupported();

    // ═══════════════════════════════════════════════════════════════════
    // INITIALIZE
    // ═══════════════════════════════════════════════════════════════════

    constructor() Ownable(msg.sender) {
        maxPerTx = 10 ether;
        maxDaily = 100 ether;
        minIntervalSeconds = 5;
        paused = false;
        allowlistMode = false;

        // ICM defaults
        maxCrossSubnetAmount = 5 ether;
        requiredConfirmations = 3;
    }

    // ═══════════════════════════════════════════════════════════════════
    // NATIVE AVAX — POLICY EVALUATION
    // ═══════════════════════════════════════════════════════════════════

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

        uint256 effectiveMaxDaily = agent.hasOverride ? agent.maxDaily : maxDaily;
        uint256 effectiveDailySpent = _getEffectiveDailySpent(from);
        if (effectiveDailySpent + value > effectiveMaxDaily) {
            return (false, "Exceeds daily spending limit");
        }

        uint256 effectiveMinInterval = agent.hasOverride ? agent.minInterval : minIntervalSeconds;
        uint256 lastTx = lastTxTime[from];
        if (lastTx > 0 && block.timestamp - lastTx < effectiveMinInterval) {
            return (false, "Rate limit exceeded");
        }

        return (true, "");
    }

    function record(address from, address /* to */, uint256 value) external onlyOwner {
        _refreshDailyWindow(from);
        dailySpent[from] += value;
        lastTxTime[from] = block.timestamp;
    }

    // ═══════════════════════════════════════════════════════════════════
    // ICM CROSS-SUBNET — POLICY EVALUATION
    // ═══════════════════════════════════════════════════════════════════

    /// @notice Check if a cross-subnet (ICM) intent is allowed
    /// @param agent The agent initiating the cross-subnet transfer
    /// @param targetSubnetId The 32-byte blockchainID of the target subnet
    /// @param amount The amount of AVAX being moved across subnets
    /// @return allowed Whether the intent passes all checks
    /// @return reason Human-readable reason if blocked
    function checkCrossSubnet(
        address agent,
        bytes32 targetSubnetId,
        uint256 amount
    ) public view returns (bool allowed, string memory reason) {
        if (paused) return (false, "Firewall is paused");

        if (!trustedSubnets[targetSubnetId]) {
            return (false, "Target subnet is not trusted");
        }

        // Per-subnet cap takes priority, fall back to global
        uint256 effectiveCap = subnetMaxAmount[targetSubnetId] > 0
            ? subnetMaxAmount[targetSubnetId]
            : maxCrossSubnetAmount;

        if (amount > effectiveCap) {
            return (false, "Exceeds cross-subnet transfer cap");
        }

        // Also run native AVAX checks for the agent
        return check(agent, address(0), amount);
    }

    /// @notice Add or remove a trusted subnet for ICM operations
    function setTrustedSubnet(bytes32 subnetId, string calldata label, bool trusted) external onlyOwner {
        trustedSubnets[subnetId] = trusted;
        if (trusted) {
            subnetLabels[subnetId] = label;
        }
        emit SubnetTrustUpdated(subnetId, label, trusted);
    }

    function setCrossSubnetPolicy(uint256 _maxAmount, uint256 _confirmations) external onlyOwner {
        maxCrossSubnetAmount = _maxAmount;
        requiredConfirmations = _confirmations;
        emit CrossSubnetPolicyUpdated(_maxAmount, _confirmations);
    }

    function setSubnetCap(bytes32 subnetId, uint256 maxAmount) external onlyOwner {
        subnetMaxAmount[subnetId] = maxAmount;
        emit SubnetCapSet(subnetId, maxAmount);
    }

    // ═══════════════════════════════════════════════════════════════════
    // STABLECOIN / ERC-20 — POLICY EVALUATION
    // ═══════════════════════════════════════════════════════════════════

    /// @notice Register a stablecoin for policy enforcement
    function registerStablecoin(
        address token,
        string calldata symbol,
        uint256 _maxPerTx,
        uint256 _maxDaily
    ) external onlyOwner {
        uint8 decimals = IERC20Metadata(token).decimals();
        supportedStablecoins[token] = true;
        stablecoinSymbols[token] = symbol;
        stablecoinDecimals[token] = decimals;
        stablecoinMaxPerTx[token] = _maxPerTx;
        stablecoinMaxDaily[token] = _maxDaily;
        emit StablecoinRegistered(token, symbol, decimals);
        emit StablecoinPolicyUpdated(token, _maxPerTx, _maxDaily);
    }

    function removeStablecoin(address token) external onlyOwner {
        supportedStablecoins[token] = false;
        emit StablecoinRemoved(token);
    }

    function setStablecoinPolicy(
        address token,
        uint256 _maxPerTx,
        uint256 _maxDaily
    ) external onlyOwner {
        if (!supportedStablecoins[token]) revert StablecoinNotSupported();
        stablecoinMaxPerTx[token] = _maxPerTx;
        stablecoinMaxDaily[token] = _maxPerTx;
        emit StablecoinPolicyUpdated(token, _maxPerTx, _maxDaily);
    }

    /// @notice Check if an ERC-20 transfer from `from` to `to` of `token` with `amount` is allowed
    function checkToken(
        address from,
        address to,
        address token,
        uint256 amount
    ) public view returns (bool allowed, string memory reason) {
        if (paused) return (false, "Firewall is paused");
        if (!supportedStablecoins[token]) return (false, "Token not supported");
        if (blockedRecipients[to]) return (false, "Recipient is blocklisted");

        AgentPolicy memory agent = agentPolicies[from];

        // Per-tx cap
        uint256 effectiveMaxPerTx = stablecoinMaxPerTx[token];
        if (amount > effectiveMaxPerTx) {
            return (false, "Exceeds stablecoin per-transaction cap");
        }

        // Daily cap
        uint256 effectiveMaxDaily = stablecoinMaxDaily[token];
        uint256 effectiveDailySpent = _getEffectiveStablecoinDailySpent(from, token);
        if (effectiveDailySpent + amount > effectiveMaxDaily) {
            return (false, "Exceeds stablecoin daily limit");
        }

        // Rate limit
        uint256 effectiveMinInterval = agent.hasOverride ? agent.minInterval : minIntervalSeconds;
        uint256 lastTx = lastTxTime[from];
        if (lastTx > 0 && block.timestamp - lastTx < effectiveMinInterval) {
            return (false, "Rate limit exceeded");
        }

        if (allowlistMode && !allowedContracts[to]) {
            return (false, "Contract not in allowlist");
        }

        return (true, "");
    }

    /// @notice Record a stablecoin transfer (called by proxy after forwarding)
    function recordStablecoin(address from, address token, uint256 amount) external onlyOwner {
        if (!supportedStablecoins[token]) revert StablecoinNotSupported();
        _refreshDailyWindow(from);
        stablecoinDailySpent[from][token] += amount;
        dailySpent[from] += 1; // track activity without conflating AVAX/stablecoin
        lastTxTime[from] = block.timestamp;
    }

    function _getEffectiveStablecoinDailySpent(address agent, address token) internal view returns (uint256) {
        uint256 dayStart = (block.timestamp / 1 days) * 1 days;
        if (dailyWindowStart[agent] < dayStart) return 0;
        return stablecoinDailySpent[agent][token];
    }

    // ═══════════════════════════════════════════════════════════════════
    // ADMIN SETTERS (existing)
    // ═══════════════════════════════════════════════════════════════════

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

    // ═══════════════════════════════════════════════════════════════════
    // INTERNAL
    // ═══════════════════════════════════════════════════════════════════

    function _getEffectiveDailySpent(address agent) internal view returns (uint256) {
        uint256 dayStart = (block.timestamp / 1 days) * 1 days;
        if (dailyWindowStart[agent] < dayStart) return 0;
        return dailySpent[agent];
    }

    function _refreshDailyWindow(address agent) internal {
        uint256 dayStart = (block.timestamp / 1 days) * 1 days;
        if (dailyWindowStart[agent] < dayStart) {
            dailySpent[agent] = 0;
            dailyWindowStart[agent] = dayStart;
        }
    }
}
