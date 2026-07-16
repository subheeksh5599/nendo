// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title INendoAudit — Interface for immutable on-chain audit trail
/// @notice Events emitted by NendoAudit.sol. The dashboard and SDK query
///         these events via eth_getLogs. Separated from implementation
///         for type-safe SDK integration.
interface INendoAudit {
    // ── Native AVAX events ──
    event TransactionAllowed(
        address indexed agent, address indexed recipient,
        uint256 amount, bytes32 indexed intentHash, uint256 timestamp
    );
    event TransactionBlocked(
        address indexed agent, address indexed recipient,
        uint256 amount, string reason, uint256 timestamp
    );

    // ── ICM cross-subnet events ──
    event CrossSubnetIntentAllowed(
        address indexed agent, bytes32 indexed targetSubnetId,
        string targetSubnetLabel, uint256 amount, uint256 confirmations, uint256 timestamp
    );
    event CrossSubnetIntentBlocked(
        address indexed agent, bytes32 indexed targetSubnetId,
        string targetSubnetLabel, uint256 amount, string reason, uint256 timestamp
    );

    // ── Stablecoin events ──
    event StablecoinTransferAllowed(
        address indexed agent, address indexed recipient, address indexed token,
        string tokenSymbol, uint256 amount, uint256 timestamp
    );
    event StablecoinTransferBlocked(
        address indexed agent, address indexed recipient, address indexed token,
        string tokenSymbol, uint256 amount, string reason, uint256 timestamp
    );

    // ── DID registry events ──
    event DIDAgentRegistered(
        address indexed agent, string did, string name,
        string agentType, address indexed controller, uint256 timestamp
    );

    // ── Write methods ──
    function logAllowed(address agent, address recipient, uint256 amount, bytes32 intentHash) external;
    function logBlocked(address agent, address recipient, uint256 amount, string memory reason) external;
    function logStablecoinAllowed(address agent, address recipient, address token, string memory symbol, uint256 amount) external;
    function logStablecoinBlocked(address agent, address recipient, address token, string memory symbol, uint256 amount, string memory reason) external;
    function logCrossSubnetAllowed(address agent, bytes32 targetSubnetId, string memory label, uint256 amount, uint256 confirmations) external;
    function logCrossSubnetBlocked(address agent, bytes32 targetSubnetId, string memory label, uint256 amount, string memory reason) external;
}
