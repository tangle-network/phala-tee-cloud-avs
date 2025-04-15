// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

/**
 * @title Interface for the Phala Service Manager.
 * @notice Defines functions needed by the SLA Oracle to interact with the Service Manager.
 */
interface IPhalaServiceManager {
    /**
     * @notice Checks if an operator address is registered with the Service Manager.
     * @param operator The address to check.
     * @return bool True if the operator is registered, false otherwise.
     */
    function isOperatorRegistered(address operator) external view returns (bool);

    /**
     * @notice Called by the SLA Oracle to report that an operator failed to respond to a challenge.
     * @param operator The address of the operator who failed the liveness check.
     */
    function recordOperatorLivenessFailure(address operator) external;
} 