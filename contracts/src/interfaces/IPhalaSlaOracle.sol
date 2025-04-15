// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

/**
 * @title Interface for the Phala SLA Oracle.
 * @notice Defines the functions for interacting with the SLA challenge and response mechanism.
 */
interface IPhalaSlaOracle {
    /**
     * @notice Emitted when a new SLA challenge is issued for an operator.
     * @param challengeId Unique identifier for the challenge.
     * @param operator The address of the operator being challenged.
     * @param challengeData Data associated with the challenge (e.g., timestamp, type).
     * @param responseWindowEndBlock The block number by which the operator must respond.
     */
    event SlaChallengeIssued(
        uint256 indexed challengeId,
        address indexed operator,
        bytes challengeData,
        uint256 responseWindowEndBlock
    );

    /**
     * @notice Emitted when an operator successfully responds to an SLA challenge.
     * @param challengeId The ID of the challenge being responded to.
     * @param operator The address of the responding operator.
     * @param responseData Data provided by the operator as proof of liveness/SLA.
     */
    event SlaChallengeResponded(uint256 indexed challengeId, address indexed operator, bytes responseData);

    /**
     * @notice Emitted when an operator fails to respond to an SLA challenge within the window.
     * @param challengeId The ID of the challenge that expired.
     * @param operator The address of the operator who failed to respond.
     */
    event SlaChallengeExpired(uint256 indexed challengeId, address indexed operator);

    /**
     * @notice Issues a new SLA challenge to an operator.
     * @dev Typically called by the authorized Tokenomic Manager.
     * @param operator The address of the operator to challenge.
     * @param challengeData Arbitrary data describing the challenge.
     * @return challengeId The unique ID assigned to this challenge.
     */
    function issueSlaChallenge(address operator, bytes calldata challengeData) external returns (uint256 challengeId);

    /**
     * @notice Allows an operator to respond to an active SLA challenge.
     * @param challengeId The ID of the challenge to respond to.
     * @param responseData Data proving the operator meets the challenged SLA (e.g., signed heartbeat, TEE proof).
     */
    function respondToSlaChallenge(uint256 challengeId, bytes calldata responseData) external;

    /**
     * @notice Checks if a challenge has expired and reports the failure if necessary.
     * @dev Can be called by anyone after the response window has passed.
     *      Calls the Service Manager to record the failure.
     * @param challengeId The ID of the challenge to check.
     */
    function checkAndReportChallengeExpiry(uint256 challengeId) external;
} 