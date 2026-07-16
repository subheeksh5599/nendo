// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title NendoAudit — Immutable on-chain audit trail for AI agent transactions
/// @notice Every allow/block decision is recorded here. Events are the source of truth.
///         Supports native AVAX, ICM cross-subnet, and stablecoin operations.
/// @dev Lightweight — only emits events, no storage writes. Query on SnowTrace.
contract NendoAudit {

    // ═══════════════════════════════════════════════════════════════════
    // NATIVE AVAX EVENTS
    // ═══════════════════════════════════════════════════════════════════

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

    event EmergencyResume(
        address indexed by,
        uint256 timestamp
    );

    // ═══════════════════════════════════════════════════════════════════
    // ICM CROSS-SUBNET EVENTS
    // ═══════════════════════════════════════════════════════════════════

    event CrossSubnetIntentAllowed(
        address indexed agent,
        bytes32 indexed targetSubnetId,
        string targetSubnetLabel,
        uint256 amount,
        uint256 requiredConfirmations,
        uint256 timestamp
    );

    event CrossSubnetIntentBlocked(
        address indexed agent,
        bytes32 indexed targetSubnetId,
        string targetSubnetLabel,
        uint256 amount,
        string reason,
        uint256 timestamp
    );

    event SubnetTrustUpdated(
        bytes32 indexed subnetId,
        string label,
        bool trusted,
        uint256 timestamp
    );

    // ═══════════════════════════════════════════════════════════════════
    // STABLECOIN EVENTS
    // ═══════════════════════════════════════════════════════════════════

    event StablecoinTransferAllowed(
        address indexed agent,
        address indexed recipient,
        address indexed token,
        string tokenSymbol,
        uint256 amount,
        uint256 timestamp
    );

    event StablecoinTransferBlocked(
        address indexed agent,
        address indexed recipient,
        address indexed token,
        string tokenSymbol,
        uint256 amount,
        string reason,
        uint256 timestamp
    );

    // ═══════════════════════════════════════════════════════════════════
    // DID REGISTRY EVENTS
    // ═══════════════════════════════════════════════════════════════════

    event DIDAgentRegistered(
        address indexed agent,
        string did,
        string name,
        string agentType,
        address indexed controller,
        uint256 timestamp
    );

    event DIDDocumentUpdated(
        address indexed agent,
        string did,
        string didDocument,
        uint256 timestamp
    );

    event VerificationMethodAdded(
        address indexed agent,
        string methodId,
        uint256 timestamp
    );

    event ServiceEndpointAdded(
        address indexed agent,
        string serviceId,
        string serviceType,
        string endpoint,
        uint256 timestamp
    );

    // ═══════════════════════════════════════════════════════════════════
    // NATIVE AVAX — WRITE METHODS
    // ═══════════════════════════════════════════════════════════════════

    function logAllowed(
        address agent,
        address recipient,
        uint256 amount,
        bytes32 intentHash
    ) external {
        emit TransactionAllowed(agent, recipient, amount, intentHash, block.timestamp);
    }

    function logBlocked(
        address agent,
        address recipient,
        uint256 amount,
        string memory reason
    ) external {
        emit TransactionBlocked(agent, recipient, amount, reason, block.timestamp);
    }

    function registerAgent(address agent, string memory name) external {
        emit AgentRegistered(agent, name, block.timestamp);
    }

    function logPause(address by) external {
        emit EmergencyPause(by, block.timestamp);
    }

    function logResume(address by) external {
        emit EmergencyResume(by, block.timestamp);
    }

    // ═══════════════════════════════════════════════════════════════════
    // ICM CROSS-SUBNET — WRITE METHODS
    // ═══════════════════════════════════════════════════════════════════

    function logCrossSubnetAllowed(
        address agent,
        bytes32 targetSubnetId,
        string memory targetSubnetLabel,
        uint256 amount,
        uint256 confirmations
    ) external {
        emit CrossSubnetIntentAllowed(
            agent, targetSubnetId, targetSubnetLabel,
            amount, confirmations, block.timestamp
        );
    }

    function logCrossSubnetBlocked(
        address agent,
        bytes32 targetSubnetId,
        string memory targetSubnetLabel,
        uint256 amount,
        string memory reason
    ) external {
        emit CrossSubnetIntentBlocked(
            agent, targetSubnetId, targetSubnetLabel,
            amount, reason, block.timestamp
        );
    }

    function logSubnetTrustUpdate(
        bytes32 subnetId,
        string memory label,
        bool trusted
    ) external {
        emit SubnetTrustUpdated(subnetId, label, trusted, block.timestamp);
    }

    // ═══════════════════════════════════════════════════════════════════
    // STABLECOIN — WRITE METHODS
    // ═══════════════════════════════════════════════════════════════════

    function logStablecoinAllowed(
        address agent,
        address recipient,
        address token,
        string memory tokenSymbol,
        uint256 amount
    ) external {
        emit StablecoinTransferAllowed(
            agent, recipient, token, tokenSymbol,
            amount, block.timestamp
        );
    }

    function logStablecoinBlocked(
        address agent,
        address recipient,
        address token,
        string memory tokenSymbol,
        uint256 amount,
        string memory reason
    ) external {
        emit StablecoinTransferBlocked(
            agent, recipient, token, tokenSymbol,
            amount, reason, block.timestamp
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    // DID REGISTRY — WRITE METHODS
    // ═══════════════════════════════════════════════════════════════════

    function logDIDAgentRegistered(
        address agent,
        string memory did,
        string memory name,
        string memory agentType,
        address controller
    ) external {
        emit DIDAgentRegistered(agent, did, name, agentType, controller, block.timestamp);
    }

    function logDIDDocumentUpdated(
        address agent,
        string memory did,
        string memory didDocument
    ) external {
        emit DIDDocumentUpdated(agent, did, didDocument, block.timestamp);
    }

    function logVerificationMethodAdded(
        address agent,
        string memory methodId
    ) external {
        emit VerificationMethodAdded(agent, methodId, block.timestamp);
    }

    function logServiceEndpointAdded(
        address agent,
        string memory serviceId,
        string memory serviceType,
        string memory endpoint
    ) external {
        emit ServiceEndpointAdded(agent, serviceId, serviceType, endpoint, block.timestamp);
    }
}
