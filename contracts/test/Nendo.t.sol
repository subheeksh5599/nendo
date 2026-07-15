// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import { Test } from "forge-std/Test.sol";
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
}