// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import { Script, console } from "forge-std/Script.sol";
import { NendoPolicy } from "../src/NendoPolicy.sol";
import { NendoAudit } from "../src/NendoAudit.sol";
import { NendoRegistry } from "../src/NendoRegistry.sol";

contract DeployScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(deployerPrivateKey);

        NendoPolicy policy = new NendoPolicy();
        NendoAudit audit = new NendoAudit();
        NendoRegistry registry = new NendoRegistry();

        console.log("NendoPolicy deployed at:", address(policy));
        console.log("NendoAudit deployed at:", address(audit));
        console.log("NendoRegistry deployed at:", address(registry));

        vm.stopBroadcast();
    }
}
