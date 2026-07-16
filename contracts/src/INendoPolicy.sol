// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title INendoPolicy — Interface for on-chain policy enforcement
/// @notice The Rust proxy calls these view functions (zero gas) to check
///         transactions against the active policy. Separated from the
///         implementation for testability and SDK integration.
interface INendoPolicy {
    // ── Errors ──
    error Paused();
    error RecipientBlocked();
    error ContractNotAllowed();
    error ExceedsMaxPerTx(uint256 requested, uint256 max);
    error ExceedsDailyLimit(uint256 total, uint256 max);
    error RateLimitExceeded(uint256 lastTx, uint256 minInterval);
    error SubnetNotTrusted(bytes32 subnetId);
    error ExceedsCrossSubnetCap(uint256 requested, uint256 max);
    error StablecoinNotSupported(address token);

    // ── View functions (zero gas, called by proxy via eth_call) ──

    /// Check if a native AVAX transaction passes all rules.
    function check(address from, address to, uint256 value)
        external view returns (bool allowed, string memory reason);

    /// Check if an ICM cross-subnet intent passes all rules.
    function checkCrossSubnet(address agent, bytes32 targetSubnetId, uint256 amount)
        external view returns (bool allowed, string memory reason);

    /// Check if an ERC-20 stablecoin transfer passes all rules.
    function checkToken(address from, address to, address token, uint256 amount)
        external view returns (bool allowed, string memory reason);

    // ── State-modifying functions (gas, called by proxy owner) ──

    /// Record a native AVAX transaction (updates daily spent + rate limit).
    function record(address from, address to, uint256 value) external;

    /// Record a stablecoin transfer.
    function recordStablecoin(address from, address token, uint256 amount) external;

    // ── Admin (owner-only) ──

    function pause() external;
    function unpause() external;
    function paused() external view returns (bool);
    function owner() external view returns (address);
}
