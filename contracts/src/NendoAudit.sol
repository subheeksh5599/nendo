// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title NendoAudit — Immutable on-chain audit trail for AI agent transactions
/// @notice Every allow/block decision is recorded here. Events are the source of truth.
/// @dev Lightweight — only emits events, no storage writes. View on Snowtrace/Avalanche Explorer.
contract NendoAudit {

    // ─── Events (immutable, queryable on-chain) ───────────────────────

    event TransactionAllowed(
        address indexed agent,
        address indexed recipient,
        uint256 amount,
        bytes32 indexed intentHash,
        uint256 timestamp
    );

    event TransactionBlocked(
        address indexed agent,
        address indexed recipient,
        uint256 amount,
        string reason,
        uint256 timestamp
    );

    event AgentRegistered(
        address indexed agent,
        string name,
        uint256 timestamp
    );

    event EmergencyPause(
        address indexed by,
        uint256 timestamp
    );

    // ─── Write Methods ──────────────────────────────────────────────────

    /// @notice Call this when Nendo allows a transaction
    function logAllowed(
        address agent,
        address recipient,
        uint256 amount,
        bytes32 intentHash
    ) external {
        emit TransactionAllowed(
            agent,
            recipient,
            amount,
            intentHash,
            block.timestamp
        );
    }

    /// @notice Call this when Nendo blocks a transaction
    function logBlocked(
        address agent,
        address recipient,
        uint256 amount,
        string memory reason
    ) external {
        emit TransactionBlocked(
            agent,
            recipient,
            amount,
            reason,
            block.timestamp
        );
    }

    /// @notice Register an AI agent on-chain
    function registerAgent(address agent, string memory name) external {
        emit AgentRegistered(agent, name, block.timestamp);
    }

    /// @notice Emergency pause event
    function logPause(address by) external {
        emit EmergencyPause(by, block.timestamp);
    }
}