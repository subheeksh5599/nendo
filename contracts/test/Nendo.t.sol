// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import { Test } from "forge-std/Test.sol";
import { Vm } from "forge-std/Vm.sol";
import { NendoPolicy } from "../src/NendoPolicy.sol";
import { NendoAudit } from "../src/NendoAudit.sol";

contract NendoTest is Test {
    NendoPolicy public policy;
    NendoAudit public audit;

    address constant AGENT = address(0xABCD);
    address constant RECIPIENT = address(0xDEAD);

    function setUp() public {
        policy = new NendoPolicy();
        audit = new NendoAudit();
    }

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
        // First tx should pass
        (bool allowed1, ) = policy.check(AGENT, RECIPIENT, 1 ether);
        assertTrue(allowed1);

        // Record it (simulates proxy forwarding the tx)
        policy.record(AGENT, RECIPIENT, 1 ether);

        // Second immediate tx should fail rate limit
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

    // ─── Additional tests for grant submission ───────────────────────

    function test_allowlist_mode_blocks_unknown_contract() public {
        address DEX = address(0xBEEF);
        policy.setAllowedContract(DEX, true);
        policy.setAllowlistMode(true);

        // Unlisted contract should be blocked
        (bool allowed, string memory reason) = policy.check(AGENT, RECIPIENT, 1 ether);
        assertFalse(allowed);
        assertEq(reason, "Contract not in allowlist");

        // Allowed contract should pass
        (bool allowed2, ) = policy.check(AGENT, DEX, 1 ether);
        assertTrue(allowed2);
    }

    function test_daily_limit_enforced_after_recording() public {
        // Set a very low daily limit for testing
        policy.setGlobalPolicy(5 ether, 5 ether, 0);

        // First tx: 3 AVAX should pass
        (bool allowed1, ) = policy.check(AGENT, RECIPIENT, 3 ether);
        assertTrue(allowed1);
        policy.record(AGENT, RECIPIENT, 3 ether);

        // Second tx: another 3 AVAX (total=6 > daily=5) should be blocked
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
        // topic[0] is the event signature hash
        assertEq(
            logs[0].topics[0],
            keccak256("TransactionBlocked(address,address,uint256,string,uint256)")
        );
        // topic[1] is the indexed agent address
        assertEq(address(uint160(uint256(logs[0].topics[1]))), AGENT);
        // topic[2] is the indexed recipient address
        assertEq(address(uint160(uint256(logs[0].topics[2]))), RECIPIENT);
    }
}