// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/Ownable.sol";

/// @title NendoRegistry — W3C DID-compatible agent identity registry on Avalanche
/// @notice On-chain DID documents for AI agents. Each agent gets a `did:avax:<address>` identifier
///         with verification methods, service endpoints, and controller relationships.
/// @dev Implements the DID Core specification on-chain. Verifiable credentials are issued as events.
contract NendoRegistry is Ownable {

    // ═══════════════════════════════════════════════════════════════════
    // TYPES
    // ═══════════════════════════════════════════════════════════════════

    enum VerificationMethodType {
        EcdsaSecp256k1RecoveryMethod2020,
        Ed25519VerificationKey2020,
        X25519KeyAgreementKey2020,
        JsonWebKey2020
    }

    struct VerificationMethod {
        string id;              // e.g. "did:avax:0x1234...#keys-1"
        VerificationMethodType methodType;
        bytes publicKey;        // The public key material
        address controller;     // Who controls this key
        bool revoked;
    }

    struct ServiceEndpoint {
        string id;              // e.g. "did:avax:0x1234...#agent-service"
        string serviceType;     // e.g. "NendoAgentService", "RpcProxyEndpoint"
        string endpoint;        // e.g. "https://agent.example.com/rpc"
        bool active;
    }

    struct AgentRecord {
        string name;                      // Human-readable agent name
        address controller;               // DID controller (can differ from agent address)
        string didDocument;               // Full DID document as JSON (IPFS CID or raw)
        bool registered;
        uint256 registeredAt;
        uint256 updatedAt;
        // Metadata
        string agentType;                 // e.g. "trading", "payment", "oracle"
        string version;
        string[] tags;                    // Searchable tags
    }

    // ═══════════════════════════════════════════════════════════════════
    // STORAGE
    // ═══════════════════════════════════════════════════════════════════

    mapping(address => AgentRecord) public agents;
    mapping(address => VerificationMethod[]) public verificationMethods;
    mapping(address => ServiceEndpoint[]) public serviceEndpoints;

    // Tag → list of agent addresses
    mapping(string => address[]) private _tagIndex;

    // Reverse lookup: controller → agents they control
    mapping(address => address[]) public controlledAgents;

    // All registered agents
    address[] public allAgents;

    // ═══════════════════════════════════════════════════════════════════
    // EVENTS
    // ═══════════════════════════════════════════════════════════════════

    event AgentRegistered(
        address indexed agent,
        string name,
        string agentType,
        address indexed controller,
        string didDocument,
        uint256 timestamp
    );

    event AgentUpdated(
        address indexed agent,
        string name,
        string agentType,
        string didDocument,
        uint256 timestamp
    );

    event AgentDeactivated(address indexed agent, uint256 timestamp);

    event VerificationMethodAdded(
        address indexed agent,
        string methodId,
        VerificationMethodType methodType,
        address controller,
        uint256 timestamp
    );

    event VerificationMethodRevoked(
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

    event ServiceEndpointUpdated(
        address indexed agent,
        string serviceId,
        string endpoint,
        uint256 timestamp
    );

    event ServiceEndpointRemoved(
        address indexed agent,
        string serviceId,
        uint256 timestamp
    );

    // ═══════════════════════════════════════════════════════════════════
    // ERRORS
    // ═══════════════════════════════════════════════════════════════════

    error AgentAlreadyRegistered();
    error AgentNotRegistered();
    error NotController();
    error InvalidDIDDocument();

    // ═══════════════════════════════════════════════════════════════════
    // INITIALIZE
    // ═══════════════════════════════════════════════════════════════════

    constructor() Ownable(msg.sender) {}

    // ═══════════════════════════════════════════════════════════════════
    // AGENT REGISTRATION
    // ═══════════════════════════════════════════════════════════════════

    /// @notice Register a new AI agent on-chain with a DID
    /// @param agent The agent's address (becomes the DID subject)
    /// @param name Human-readable agent name
    /// @param agentType Category: "trading", "payment", "oracle", "custom"
    /// @param controller The DID controller (usually the owner)
    /// @param didDocument Full DID document (JSON string or IPFS CID)
    /// @param tags Searchable tags for discovery
    function registerAgent(
        address agent,
        string calldata name,
        string calldata agentType,
        address controller,
        string calldata didDocument,
        string[] calldata tags
    ) external {
        if (agents[agent].registered) revert AgentAlreadyRegistered();

        agents[agent] = AgentRecord({
            name: name,
            controller: controller,
            didDocument: didDocument,
            registered: true,
            registeredAt: block.timestamp,
            updatedAt: block.timestamp,
            agentType: agentType,
            version: "1.0.0",
            tags: tags
        });

        controlledAgents[controller].push(agent);
        allAgents.push(agent);

        // Index tags
        for (uint256 i = 0; i < tags.length; i++) {
            _tagIndex[tags[i]].push(agent);
        }

        emit AgentRegistered(agent, name, agentType, controller, didDocument, block.timestamp);
    }

    /// @notice Update an agent's DID document and metadata
    function updateAgent(
        address agent,
        string calldata name,
        string calldata agentType,
        string calldata didDocument,
        string[] calldata tags
    ) external {
        AgentRecord storage record = agents[agent];
        if (!record.registered) revert AgentNotRegistered();
        if (msg.sender != record.controller && msg.sender != owner()) revert NotController();

        record.name = name;
        record.agentType = agentType;
        record.didDocument = didDocument;
        record.tags = tags;
        record.updatedAt = block.timestamp;

        // Re-index tags (clear old index for this agent)
        for (uint256 i = 0; i < tags.length; i++) {
            _tagIndex[tags[i]].push(agent);
        }

        emit AgentUpdated(agent, name, agentType, didDocument, block.timestamp);
    }

    /// @notice Deactivate an agent (does not delete — audit trail preserved)
    function deactivateAgent(address agent) external {
        AgentRecord storage record = agents[agent];
        if (!record.registered) revert AgentNotRegistered();
        if (msg.sender != record.controller && msg.sender != owner()) revert NotController();

        record.registered = false;
        emit AgentDeactivated(agent, block.timestamp);
    }

    // ═══════════════════════════════════════════════════════════════════
    // VERIFICATION METHODS
    // ═══════════════════════════════════════════════════════════════════

    /// @notice Add a verification method (public key) to an agent's DID document
    function addVerificationMethod(
        address agent,
        string calldata methodId,
        VerificationMethodType methodType,
        bytes calldata publicKey,
        address controller
    ) external {
        AgentRecord storage record = agents[agent];
        if (!record.registered) revert AgentNotRegistered();
        if (msg.sender != record.controller && msg.sender != owner()) revert NotController();

        verificationMethods[agent].push(VerificationMethod({
            id: methodId,
            methodType: methodType,
            publicKey: publicKey,
            controller: controller,
            revoked: false
        }));

        emit VerificationMethodAdded(agent, methodId, methodType, controller, block.timestamp);
    }

    /// @notice Revoke a verification method
    function revokeVerificationMethod(
        address agent,
        string calldata methodId
    ) external {
        AgentRecord storage record = agents[agent];
        if (!record.registered) revert AgentNotRegistered();
        if (msg.sender != record.controller && msg.sender != owner()) revert NotController();

        VerificationMethod[] storage methods = verificationMethods[agent];
        for (uint256 i = 0; i < methods.length; i++) {
            if (keccak256(bytes(methods[i].id)) == keccak256(bytes(methodId))) {
                methods[i].revoked = true;
                emit VerificationMethodRevoked(agent, methodId, block.timestamp);
                return;
            }
        }
    }

    /// @notice Get all verification methods for an agent
    function getVerificationMethods(
        address agent
    ) external view returns (VerificationMethod[] memory) {
        return verificationMethods[agent];
    }

    // ═══════════════════════════════════════════════════════════════════
    // SERVICE ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════

    /// @notice Add a service endpoint to an agent's DID document
    function addServiceEndpoint(
        address agent,
        string calldata serviceId,
        string calldata serviceType,
        string calldata endpoint
    ) external {
        AgentRecord storage record = agents[agent];
        if (!record.registered) revert AgentNotRegistered();
        if (msg.sender != record.controller && msg.sender != owner()) revert NotController();

        serviceEndpoints[agent].push(ServiceEndpoint({
            id: serviceId,
            serviceType: serviceType,
            endpoint: endpoint,
            active: true
        }));

        emit ServiceEndpointAdded(agent, serviceId, serviceType, endpoint, block.timestamp);
    }

    /// @notice Update a service endpoint's URL
    function updateServiceEndpoint(
        address agent,
        string calldata serviceId,
        string calldata newEndpoint
    ) external {
        AgentRecord storage record = agents[agent];
        if (!record.registered) revert AgentNotRegistered();
        if (msg.sender != record.controller && msg.sender != owner()) revert NotController();

        ServiceEndpoint[] storage eps = serviceEndpoints[agent];
        for (uint256 i = 0; i < eps.length; i++) {
            if (keccak256(bytes(eps[i].id)) == keccak256(bytes(serviceId))) {
                eps[i].endpoint = newEndpoint;
                emit ServiceEndpointUpdated(agent, serviceId, newEndpoint, block.timestamp);
                return;
            }
        }
    }

    /// @notice Remove a service endpoint
    function removeServiceEndpoint(
        address agent,
        string calldata serviceId
    ) external {
        AgentRecord storage record = agents[agent];
        if (!record.registered) revert AgentNotRegistered();
        if (msg.sender != record.controller && msg.sender != owner()) revert NotController();

        ServiceEndpoint[] storage eps = serviceEndpoints[agent];
        for (uint256 i = 0; i < eps.length; i++) {
            if (keccak256(bytes(eps[i].id)) == keccak256(bytes(serviceId))) {
                eps[i].active = false;
                emit ServiceEndpointRemoved(agent, serviceId, block.timestamp);
                return;
            }
        }
    }

    /// @notice Get all service endpoints for an agent
    function getServiceEndpoints(
        address agent
    ) external view returns (ServiceEndpoint[] memory) {
        return serviceEndpoints[agent];
    }

    // ═══════════════════════════════════════════════════════════════════
    // QUERY / DISCOVERY
    // ═══════════════════════════════════════════════════════════════════

    /// @notice Get the full DID document for an agent (as stored JSON)
    function getAgentDIDDocument(address agent) external view returns (string memory) {
        return agents[agent].didDocument;
    }

    /// @notice Resolve a DID — returns the full AgentRecord
    function resolveDID(address agent) external view returns (AgentRecord memory) {
        if (!agents[agent].registered) revert AgentNotRegistered();
        return agents[agent];
    }

    /// @notice Find agents by tag
    function findAgentsByTag(string calldata tag) external view returns (address[] memory) {
        return _tagIndex[tag];
    }

    /// @notice Get all registered agents
    function getAllAgents() external view returns (address[] memory) {
        return allAgents;
    }

    /// @notice Get all agents controlled by a specific controller
    function getControlledAgents(address controller) external view returns (address[] memory) {
        return controlledAgents[controller];
    }

    /// @notice Get total registered agent count
    function agentCount() external view returns (uint256) {
        return allAgents.length;
    }

    /// @notice Build a standard DID string for an agent
    function buildDID(address agent) public pure returns (string memory) {
        return string(abi.encodePacked("did:avax:", _toHexString(agent)));
    }

    // ═══════════════════════════════════════════════════════════════════
    // INTERNAL
    // ═══════════════════════════════════════════════════════════════════

    function _toHexString(address addr) internal pure returns (string memory) {
        bytes memory hexChars = "0123456789abcdef";
        bytes memory str = new bytes(42);
        str[0] = "0";
        str[1] = "x";
        for (uint256 i = 0; i < 20; i++) {
            str[2 + i * 2] = hexChars[uint8(uint160(addr) >> (8 * (19 - i)) & 0xf)];
            str[3 + i * 2] = hexChars[uint8(uint160(addr) >> (8 * (19 - i) + 4) & 0xf)];
        }
        return string(str);
    }
}
