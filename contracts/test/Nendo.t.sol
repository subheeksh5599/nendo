// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import { Test } from "forge-std/Test.sol";
import { Vm } from "forge-std/Vm.sol";
import { NendoPolicy } from "../src/NendoPolicy.sol";
import { NendoAudit } from "../src/NendoAudit.sol";
import { NendoRegistry } from "../src/NendoRegistry.sol";
import { IERC20Metadata } from "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";

contract NendoTest is Test {
    NendoPolicy public policy;
    NendoAudit public audit;
    NendoRegistry public registry;

    address constant AGENT = address(0xABCD);
    address constant RECIPIENT = address(0xDEAD);
    address constant DEX = address(0xBEEF);
    address constant CONTROLLER = address(0xFACE);

    bytes32 constant TEST_SUBNET = bytes32(uint256(0x1));
    bytes32 constant TEST_SUBNET_2 = bytes32(uint256(0x2));

    function setUp() public {
        policy = new NendoPolicy();
        audit = new NendoAudit();
        registry = new NendoRegistry();
    }

    // ═══════════════════════════════════════════════════════════════════
    // NATIVE AVAX TESTS (existing)
    // ═══════════════════════════════════════════════════════════════════

    function test_default_policy_allows_small_tx() public view {
        (bool allowed, ) = policy.check(AGENT, RECIPIENT, 1 ether);
        assertTrue(allowed);
    }

    function test_default_policy_blocks_excessive_tx() public view {
        (bool allowed, ) = policy.check(AGENT, RECIPIENT, 100 ether);
        assertFalse(allowed);
    }

    function test_blocked_recipient() public {
        policy.setBlockedRecipient(RECIPIENT, true);
        (bool allowed, string memory reason) = policy.check(AGENT, RECIPIENT, 1 ether);
        assertFalse(allowed);
        assertEq(reason, "Recipient is blocklisted");
    }

    function test_rate_limit() public {
        (bool allowed1, ) = policy.check(AGENT, RECIPIENT, 1 ether);
        assertTrue(allowed1);
        policy.record(AGENT, RECIPIENT, 1 ether);

        (bool allowed2, ) = policy.check(AGENT, RECIPIENT, 1 ether);
        assertFalse(allowed2);
    }

    function test_pause_circuit_breaker() public {
        policy.pause();
        (bool allowed, string memory reason) = policy.check(AGENT, RECIPIENT, 1 ether);
        assertFalse(allowed);
        assertEq(reason, "Firewall is paused");
    }

    function test_agent_override() public {
        policy.setAgentPolicy(AGENT, 50 ether, 500 ether);
        (bool allowed, ) = policy.check(AGENT, RECIPIENT, 30 ether);
        assertTrue(allowed);
    }

    function test_allowlist_mode_blocks_unknown_contract() public {
        policy.setAllowedContract(DEX, true);
        policy.setAllowlistMode(true);

        (bool allowed, string memory reason) = policy.check(AGENT, RECIPIENT, 1 ether);
        assertFalse(allowed);
        assertEq(reason, "Contract not in allowlist");

        (bool allowed2, ) = policy.check(AGENT, DEX, 1 ether);
        assertTrue(allowed2);
    }

    function test_daily_limit_enforced_after_recording() public {
        policy.setGlobalPolicy(5 ether, 5 ether, 0);

        (bool allowed1, ) = policy.check(AGENT, RECIPIENT, 3 ether);
        assertTrue(allowed1);
        policy.record(AGENT, RECIPIENT, 3 ether);

        (bool allowed2, string memory reason) = policy.check(AGENT, RECIPIENT, 3 ether);
        assertFalse(allowed2);
        assertEq(reason, "Exceeds daily spending limit");
    }

    function test_unpause_restores_transactions() public {
        policy.pause();
        (bool blocked, ) = policy.check(AGENT, RECIPIENT, 1 ether);
        assertFalse(blocked);

        policy.unpause();
        (bool allowed, ) = policy.check(AGENT, RECIPIENT, 1 ether);
        assertTrue(allowed);
    }

    function test_NendoAudit_emits_blocked_event() public {
        vm.recordLogs();
        audit.logBlocked(AGENT, RECIPIENT, 5 ether, "Exceeds per-transaction cap");

        Vm.Log[] memory logs = vm.getRecordedLogs();
        assertEq(logs.length, 1);
        assertEq(
            logs[0].topics[0],
            keccak256("TransactionBlocked(address,address,uint256,string,uint256)")
        );
        assertEq(address(uint160(uint256(logs[0].topics[1]))), AGENT);
        assertEq(address(uint160(uint256(logs[0].topics[2]))), RECIPIENT);
    }

    // ═══════════════════════════════════════════════════════════════════
    // ICM CROSS-SUBNET TESTS
    // ═══════════════════════════════════════════════════════════════════

    function test_icm_untrusted_subnet_blocked() public {
        (bool allowed, string memory reason) = policy.checkCrossSubnet(AGENT, TEST_SUBNET, 1 ether);
        assertFalse(allowed);
        assertEq(reason, "Target subnet is not trusted");
    }

    function test_icm_trusted_subnet_allows() public {
        policy.setTrustedSubnet(TEST_SUBNET, "Payroll L1", true);
        (bool allowed, ) = policy.checkCrossSubnet(AGENT, TEST_SUBNET, 1 ether);
        assertTrue(allowed);
    }

    function test_icm_exceeds_cross_subnet_cap() public {
        policy.setTrustedSubnet(TEST_SUBNET, "Trading L1", true);
        // Default cross-subnet cap is 5 ether
        (bool allowed, string memory reason) = policy.checkCrossSubnet(AGENT, TEST_SUBNET, 10 ether);
        assertFalse(allowed);
        assertEq(reason, "Exceeds cross-subnet transfer cap");
    }

    function test_icm_per_subnet_cap_overrides_global() public {
        policy.setTrustedSubnet(TEST_SUBNET, "Gaming L1", true);
        policy.setSubnetCap(TEST_SUBNET, 2 ether);

        // 3 ether < global cap (5) but > subnet cap (2)
        (bool allowed, string memory reason) = policy.checkCrossSubnet(AGENT, TEST_SUBNET, 3 ether);
        assertFalse(allowed);
        assertEq(reason, "Exceeds cross-subnet transfer cap");
    }

    function test_icm_paused_blocks_cross_subnet() public {
        policy.setTrustedSubnet(TEST_SUBNET, "Payroll L1", true);
        policy.pause();

        (bool allowed, string memory reason) = policy.checkCrossSubnet(AGENT, TEST_SUBNET, 1 ether);
        assertFalse(allowed);
        assertEq(reason, "Firewall is paused");
    }

    function test_icm_audit_logs_cross_subnet_blocked() public {
        vm.recordLogs();
        audit.logCrossSubnetBlocked(AGENT, TEST_SUBNET, "Trading L1", 10 ether, "Exceeds cap");

        Vm.Log[] memory logs = vm.getRecordedLogs();
        assertEq(logs.length, 1);
        assertEq(
            logs[0].topics[0],
            keccak256("CrossSubnetIntentBlocked(address,bytes32,string,uint256,string,uint256)")
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    // STABLECOIN TESTS
    // ═══════════════════════════════════════════════════════════════════

    function test_stablecoin_not_supported_rejected() public {
        address USDC = address(0x1000);
        (bool allowed, string memory reason) = policy.checkToken(AGENT, RECIPIENT, USDC, 1000e6);
        assertFalse(allowed);
        assertEq(reason, "Token not supported");
    }

    function test_stablecoin_registration_and_check() public {
        // Deploy a mock ERC-20 for testing
        MockERC20 usdc = new MockERC20("USD Coin", "USDC", 6);
        policy.registerStablecoin(address(usdc), "USDC", 10000e6, 100000e6);

        (bool allowed, ) = policy.checkToken(AGENT, RECIPIENT, address(usdc), 5000e6);
        assertTrue(allowed);
    }

    function test_stablecoin_exceeds_cap() public {
        MockERC20 usdc = new MockERC20("USD Coin", "USDC", 6);
        policy.registerStablecoin(address(usdc), "USDC", 10000e6, 100000e6);

        (bool allowed, string memory reason) = policy.checkToken(AGENT, RECIPIENT, address(usdc), 15000e6);
        assertFalse(allowed);
        assertEq(reason, "Exceeds stablecoin per-transaction cap");
    }

    function test_stablecoin_daily_limit() public {
        MockERC20 usdc = new MockERC20("USD Coin", "USDC", 6);
        policy.registerStablecoin(address(usdc), "USDC", 10000e6, 10000e6);

        (bool allowed1, ) = policy.checkToken(AGENT, RECIPIENT, address(usdc), 6000e6);
        assertTrue(allowed1);
        policy.recordStablecoin(AGENT, address(usdc), 6000e6);

        (bool allowed2, string memory reason) = policy.checkToken(AGENT, RECIPIENT, address(usdc), 5000e6);
        assertFalse(allowed2);
        assertEq(reason, "Exceeds stablecoin daily limit");
    }

    function test_stablecoin_audit_events() public {
        MockERC20 usdt = new MockERC20("Tether USD", "USDT", 6);

        vm.recordLogs();
        audit.logStablecoinBlocked(AGENT, RECIPIENT, address(usdt), "USDT", 5000e6, "Exceeds cap");

        Vm.Log[] memory logs = vm.getRecordedLogs();
        assertEq(logs.length, 1);
        assertEq(
            logs[0].topics[0],
            keccak256("StablecoinTransferBlocked(address,address,address,string,uint256,string,uint256)")
        );
        assertEq(address(uint160(uint256(logs[0].topics[1]))), AGENT);
    }

    function test_stablecoin_paused_blocks_all() public {
        MockERC20 usdc = new MockERC20("USD Coin", "USDC", 6);
        policy.registerStablecoin(address(usdc), "USDC", 10000e6, 100000e6);
        policy.pause();

        (bool allowed, string memory reason) = policy.checkToken(AGENT, RECIPIENT, address(usdc), 1000e6);
        assertFalse(allowed);
        assertEq(reason, "Firewall is paused");
    }

    // ═══════════════════════════════════════════════════════════════════
    // DID REGISTRY TESTS
    // ═══════════════════════════════════════════════════════════════════

    function test_did_register_agent() public {
        string[] memory tags = new string[](2);
        tags[0] = "trading";
        tags[1] = "defi";

        registry.registerAgent(AGENT, "TradingBot v1", "trading", CONTROLLER, "ipfs://QmTest", tags);

        NendoRegistry.AgentRecord memory record = registry.resolveDID(AGENT);
        assertTrue(record.registered);
        assertEq(record.name, "TradingBot v1");
        assertEq(record.agentType, "trading");
        assertEq(record.controller, CONTROLLER);
    }

    function test_did_duplicate_registration_reverts() public {
        string[] memory tags = new string[](1);
        tags[0] = "test";

        registry.registerAgent(AGENT, "Bot", "test", CONTROLLER, "ipfs://QmTest", tags);

        vm.expectRevert(NendoRegistry.AgentAlreadyRegistered.selector);
        registry.registerAgent(AGENT, "Bot2", "test", CONTROLLER, "ipfs://QmTest2", tags);
    }

    function test_did_add_verification_method() public {
        string[] memory tags = new string[](1);
        tags[0] = "oracle";

        registry.registerAgent(AGENT, "OracleBot", "oracle", CONTROLLER, "ipfs://QmOracle", tags);

        bytes memory pubKey = hex"0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
        registry.addVerificationMethod(
            AGENT, "did:avax:0xABCD...#keys-1",
            NendoRegistry.VerificationMethodType.EcdsaSecp256k1RecoveryMethod2020,
            pubKey, CONTROLLER
        );

        NendoRegistry.VerificationMethod[] memory methods = registry.getVerificationMethods(AGENT);
        assertEq(methods.length, 1);
        assertEq(methods[0].controller, CONTROLLER);
        assertFalse(methods[0].revoked);
    }

    function test_did_revoke_verification_method() public {
        string[] memory tags = new string[](1);
        tags[0] = "test";

        registry.registerAgent(AGENT, "Bot", "test", CONTROLLER, "ipfs://QmTest", tags);

        bytes memory pubKey = hex"0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
        registry.addVerificationMethod(AGENT, "did:avax:0xABCD...#keys-1",
            NendoRegistry.VerificationMethodType.Ed25519VerificationKey2020, pubKey, CONTROLLER);

        registry.revokeVerificationMethod(AGENT, "did:avax:0xABCD...#keys-1");

        NendoRegistry.VerificationMethod[] memory methods = registry.getVerificationMethods(AGENT);
        assertTrue(methods[0].revoked);
    }

    function test_did_add_service_endpoint() public {
        string[] memory tags = new string[](1);
        tags[0] = "payment";

        registry.registerAgent(AGENT, "PaymentBot", "payment", CONTROLLER, "ipfs://QmPayment", tags);

        registry.addServiceEndpoint(AGENT, "did:avax:0xABCD...#agent-rpc",
            "NendoAgentService", "https://agent.example.com/rpc");

        NendoRegistry.ServiceEndpoint[] memory eps = registry.getServiceEndpoints(AGENT);
        assertEq(eps.length, 1);
        assertEq(eps[0].serviceType, "NendoAgentService");
        assertEq(eps[0].endpoint, "https://agent.example.com/rpc");
        assertTrue(eps[0].active);
    }

    function test_did_find_by_tag() public {
        string[] memory tags1 = new string[](1);
        tags1[0] = "trading";

        string[] memory tags2 = new string[](1);
        tags2[0] = "oracle";

        registry.registerAgent(AGENT, "Trader", "trading", CONTROLLER, "", tags1);
        registry.registerAgent(RECIPIENT, "Oracle", "oracle", CONTROLLER, "", tags2);

        address[] memory traders = registry.findAgentsByTag("trading");
        assertEq(traders.length, 1);
        assertEq(traders[0], AGENT);
    }

    function test_did_build_did_string() public {
        string memory did = registry.buildDID(AGENT);
        // DID format is did:avax:<hex address>
        assertTrue(_startsWith(did, "did:avax:0x"));
        assertTrue(bytes(did).length >= 50); // "did:avax:" + 42-char hex address
    }

    function _startsWith(string memory str, string memory prefix) internal pure returns (bool) {
        bytes memory strBytes = bytes(str);
        bytes memory prefixBytes = bytes(prefix);
        if (strBytes.length < prefixBytes.length) return false;
        for (uint256 i = 0; i < prefixBytes.length; i++) {
            if (strBytes[i] != prefixBytes[i]) return false;
        }
        return true;
    }

    function test_did_deactivate_agent() public {
        string[] memory tags = new string[](1);
        tags[0] = "test";

        registry.registerAgent(AGENT, "Bot", "test", CONTROLLER, "", tags);
        assertTrue(registry.resolveDID(AGENT).registered);

        registry.deactivateAgent(AGENT);

        // After deactivation, resolveDID reverts with AgentNotRegistered
        vm.expectRevert(NendoRegistry.AgentNotRegistered.selector);
        registry.resolveDID(AGENT);
    }

    function test_did_only_controller_can_update() public {
        string[] memory tags = new string[](1);
        tags[0] = "test";

        registry.registerAgent(AGENT, "Bot", "test", CONTROLLER, "", tags);

        // Non-controller cannot update
        vm.prank(address(0xB0B));
        vm.expectRevert(NendoRegistry.NotController.selector);
        registry.updateAgent(AGENT, "Hacked", "evil", "", tags);
    }

    function test_did_audit_events() public {
        vm.recordLogs();
        audit.logDIDAgentRegistered(AGENT, "did:avax:0xABCD", "TradingBot", "trading", CONTROLLER);

        Vm.Log[] memory logs = vm.getRecordedLogs();
        assertEq(logs.length, 1);
        assertEq(
            logs[0].topics[0],
            keccak256("DIDAgentRegistered(address,string,string,string,address,uint256)")
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    // FUZZ TESTS — property-based, catch overflow/underflow/edge cases
    // ═══════════════════════════════════════════════════════════════════

    /// Fuzz: No matter what value you throw at check(), if value == 0,
    /// it should always pass (assuming no other rules block it).
    function testFuzz_ZeroValueAlwaysPasses(address _agent, address _recipient) public view {
        vm.assume(_agent != address(0));
        vm.assume(_recipient != address(0));
        (bool allowed, ) = policy.check(_agent, _recipient, 0);
        assertTrue(allowed);
    }

    /// Fuzz: Any value above maxPerTx should always be blocked.
    function testFuzz_ExceedsCapAlwaysBlocked(uint256 _amount) public view {
        _amount = bound(_amount, 11 ether, type(uint128).max); // default cap is 10 ether
        (bool allowed, ) = policy.check(AGENT, RECIPIENT, _amount);
        assertFalse(allowed);
    }

    /// Fuzz: Any value at or below maxPerTx should pass (no other rules).
    function testFuzz_WithinCapPasses(uint256 _amount) public view {
        _amount = bound(_amount, 0, 10 ether);
        (bool allowed, ) = policy.check(AGENT, RECIPIENT, _amount);
        assertTrue(allowed);
    }

    /// Fuzz: Daily limit — record random amounts and verify limit enforcement.
    function testFuzz_DailyLimitOverflow(uint256 _spent, uint256 _new) public {
        _spent = bound(_spent, 0, 99 ether);
        _new = bound(_new, 0, 200 ether);
        policy.setGlobalPolicy(100 ether, 100 ether, 0);

        // Simulate already spent _spent
        policy.record(AGENT, RECIPIENT, _spent);

        (bool allowed, ) = policy.check(AGENT, RECIPIENT, _new);

        if (_spent + _new > 100 ether) {
            assertFalse(allowed);
        } else {
            assertTrue(allowed);
        }
    }

    /// Fuzz: Allowlist mode — random contracts should be blocked unless in list.
    function testFuzz_AllowlistRandomContracts(address _randomContract) public {
        _randomContract = address(uint160(bound(uint160(_randomContract), 1, type(uint160).max)));
        policy.setAllowlistMode(true);
        // Only DEX is in the allowlist
        policy.setAllowedContract(DEX, true);

        (bool allowed, ) = policy.check(AGENT, _randomContract, 1 ether);

        if (_randomContract == DEX) {
            assertTrue(allowed);
        } else {
            assertFalse(allowed);
        }
    }

    /// Edge case: Blocked recipient cannot be unblocked and then re-used
    /// without explicit re-allowing.
    function testEdge_BlockedRecipientPersists() public {
        policy.setBlockedRecipient(RECIPIENT, true);
        (bool blocked, ) = policy.check(AGENT, RECIPIENT, 1 ether);
        assertFalse(blocked);

        policy.setBlockedRecipient(RECIPIENT, false);
        (bool allowed, ) = policy.check(AGENT, RECIPIENT, 1 ether);
        assertTrue(allowed);
    }

    /// Edge case: Pause blocks even the owner.
    function testEdge_PauseBlocksOwner() public {
        policy.pause();
        address owner = policy.owner();
        (bool allowed, string memory reason) = policy.check(owner, RECIPIENT, 1 ether);
        assertFalse(allowed);
        assertEq(reason, "Firewall is paused");
    }

    /// Edge case: Zero-address recipient should be blocked if in blocklist.
    function testEdge_ZeroAddressInBlocklist() public {
        policy.setBlockedRecipient(address(0), true);
        (bool allowed, ) = policy.check(AGENT, address(0), 1 ether);
        assertFalse(allowed);
    }

    /// Edge case: Agent policy override with zero caps should block everything.
    function testEdge_AgentOverrideZeroCaps() public {
        policy.setAgentPolicy(AGENT, 0, 0);
        (bool allowed, string memory reason) = policy.check(AGENT, RECIPIENT, 1);
        assertFalse(allowed);
        assertEq(reason, "Exceeds per-transaction cap");
    }

    /// Fuzz: Rate limit — random intervals should respect the minimum.
    function testFuzz_RateLimitInterval(uint256 _interval) public {
        _interval = bound(_interval, 1, 60);
        policy.setGlobalPolicy(10 ether, 100 ether, _interval);

        (bool first, ) = policy.check(AGENT, RECIPIENT, 1 ether);
        assertTrue(first);
        policy.record(AGENT, RECIPIENT, 1 ether);

        (bool second, ) = policy.check(AGENT, RECIPIENT, 1 ether);
        // Second tx within same second should always be blocked
        assertFalse(second);
    }
}

// ═══════════════════════════════════════════════════════════════════
// MOCK ERC-20 for testing stablecoin integration
// ═══════════════════════════════════════════════════════════════════

contract MockERC20 is IERC20Metadata {
    string private _name;
    string private _symbol;
    uint8 private _decimals;

    constructor(string memory name_, string memory symbol_, uint8 decimals_) {
        _name = name_;
        _symbol = symbol_;
        _decimals = decimals_;
    }

    function name() external view returns (string memory) { return _name; }
    function symbol() external view returns (string memory) { return _symbol; }
    function decimals() external view returns (uint8) { return _decimals; }
    function totalSupply() external view returns (uint256) { return 0; }
    function balanceOf(address) external view returns (uint256) { return 0; }
    function transfer(address, uint256) external returns (bool) { return false; }
    function allowance(address, address) external view returns (uint256) { return 0; }
    function approve(address, uint256) external returns (bool) { return false; }
    function transferFrom(address, address, uint256) external returns (bool) { return false; }
}
