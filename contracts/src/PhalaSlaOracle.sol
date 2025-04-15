// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {Pausable} from "@openzeppelin/contracts/security/Pausable.sol";
import {ReentrancyGuard} from "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import {IPhalaSlaOracle} from "./interfaces/IPhalaSlaOracle.sol";
import {IPhalaServiceManager} from "./interfaces/IPhalaServiceManager.sol";

/**
 * @title Oracle for Phala Cloud AVS Operator SLA.
 * @notice Manages SLA challenges issued by the Tokenomic Manager and responses from Operators.
 *         Reports failures to the Service Manager upon expiry.
 */
contract PhalaSlaOracle is IPhalaSlaOracle, Ownable, Pausable, ReentrancyGuard {
    // --- Structs ---

    struct Challenge {
        address operator; // The operator being challenged
        bytes challengeData; // Data associated with the challenge
        uint256 responseWindowEndBlock; // Block number by which the operator must respond
        bool responded; // Flag indicating if the operator has responded
        bool reported; // Flag indicating if expiry has been reported
    }

    // --- State Variables ---

    /// @notice Address of the Phala Service Manager.
    IPhalaServiceManager public immutable serviceManager;

    /// @notice Address authorized to issue SLA challenges (Tokenomic Manager).
    address public challengeIssuer;

    /// @notice Counter for generating unique challenge IDs.
    uint256 public challengeCounter;

    /// @notice Mapping from challenge ID to the Challenge struct.
    mapping(uint256 => Challenge) public challenges;

    /// @notice The duration (in blocks) operators have to respond to a challenge.
    uint256 public responseWindowBlocks;

    /// @notice Whether the contract has been initialized.
    bool public initialized;

    // --- Events ---

    /// @notice Emitted when the Challenge Issuer address is updated.
    event ChallengeIssuerUpdated(address indexed newIssuer);

    /// @notice Emitted when the response window duration is updated.
    event ResponseWindowUpdated(uint256 newWindowBlocks);

    // --- Modifiers ---

    /// @notice Ensures the caller is the authorized Challenge Issuer.
    modifier onlyChallengeIssuer() {
        require(msg.sender == challengeIssuer, "PhalaSLA: Caller is not the Challenge Issuer");
        _;
    }

    /// @notice Ensures the contract has been initialized.
    modifier isInitialized() {
        require(initialized, "PhalaSLA: Contract is not initialized");
        _;
    }

    // --- Constructor ---

    constructor(address _serviceManager, uint256 _initialResponseWindowBlocks) {
        require(_serviceManager != address(0), "PhalaSLA: Zero address for Service Manager");
        require(_initialResponseWindowBlocks > 0, "PhalaSLA: Response window must be positive");
        serviceManager = IPhalaServiceManager(_serviceManager);
        responseWindowBlocks = _initialResponseWindowBlocks;
        // Owner is deployer, Challenge Issuer set via initialize
    }

    // --- Initialization ---

    /**
     * @notice Initializes the contract, setting the initial owner and challenge issuer.
     * @param _initialOwner The address to be set as the initial owner.
     * @param _initialChallengeIssuer The address authorized to issue challenges.
     */
    function initialize(address _initialOwner, address _initialChallengeIssuer) external {
        require(_initialOwner != address(0), "PhalaSLA: Zero address for owner");
        require(_initialChallengeIssuer != address(0), "PhalaSLA: Zero address for issuer");
        require(!initialized, "PhalaSLA: Contract is already initialized");
        challengeIssuer = _initialChallengeIssuer;
        initialized = true;
        emit ChallengeIssuerUpdated(_initialChallengeIssuer);
    }

    // --- Challenge Management ---

    /**
     * @notice Issues a new SLA challenge to a registered operator.
     * @dev Only callable by the authorized Challenge Issuer.
     * @param operator The address of the operator to challenge.
     * @param challengeData Arbitrary data describing the challenge.
     * @return challengeId The unique ID assigned to this challenge.
     */
    function issueSlaChallenge(
        address operator,
        bytes calldata challengeData
    ) external override onlyChallengeIssuer whenNotPaused isInitialized returns (uint256 challengeId) {
        require(serviceManager.isOperatorRegistered(operator), "PhalaSLA: Operator not registered");

        challengeId = ++challengeCounter;
        uint256 endBlock = block.number + responseWindowBlocks;

        challenges[challengeId] = Challenge({
            operator: operator,
            challengeData: challengeData,
            responseWindowEndBlock: endBlock,
            responded: false,
            reported: false
        });

        emit SlaChallengeIssued(challengeId, operator, challengeData, endBlock);
        return challengeId;
    }

    /**
     * @notice Allows an operator to respond to an active SLA challenge.
     * @dev Can only be called by the challenged operator before the window ends.
     * @param challengeId The ID of the challenge to respond to.
     * @param responseData Data proving the operator meets the challenged SLA.
     */
    function respondToSlaChallenge(
        uint256 challengeId,
        bytes calldata responseData
    ) external override whenNotPaused nonReentrant isInitialized {
        Challenge storage challenge = challenges[challengeId];
        require(challenge.operator != address(0), "PhalaSLA: Challenge does not exist");
        require(msg.sender == challenge.operator, "PhalaSLA: Caller is not the challenged operator");
        require(block.number <= challenge.responseWindowEndBlock, "PhalaSLA: Response window closed");
        require(!challenge.responded, "PhalaSLA: Challenge already responded to");

        challenge.responded = true;

        // The response data itself is just stored via the event for off-chain verification/logging.
        // The act of calling this function successfully is the proof of liveness for this mechanism.
        emit SlaChallengeResponded(challengeId, msg.sender, responseData);
    }

    /**
     * @notice Checks if a challenge has expired and reports the failure to the Service Manager.
     * @dev Can be called by anyone after the response window ends if not already reported.
     * @param challengeId The ID of the challenge to check.
     */
    function checkAndReportChallengeExpiry(uint256 challengeId) external override whenNotPaused nonReentrant isInitialized {
        Challenge storage challenge = challenges[challengeId];
        require(challenge.operator != address(0), "PhalaSLA: Challenge does not exist");
        require(block.number > challenge.responseWindowEndBlock, "PhalaSLA: Response window not yet closed");
        require(!challenge.responded, "PhalaSLA: Challenge was responded to");
        require(!challenge.reported, "PhalaSLA: Challenge expiry already reported");

        challenge.reported = true;

        // Report the failure to the Service Manager
        serviceManager.recordOperatorLivenessFailure(challenge.operator);

        emit SlaChallengeExpired(challengeId, challenge.operator);
    }

    // --- Admin Functions ---

    /**
     * @notice Updates the address of the Challenge Issuer.
     * @dev Only callable by the contract owner.
     * @param _newChallengeIssuer The new address authorized to issue challenges.
     */
    function setChallengeIssuer(address _newChallengeIssuer) external onlyOwner isInitialized {
        require(_newChallengeIssuer != address(0), "PhalaSLA: Cannot set zero address");
        challengeIssuer = _newChallengeIssuer;
        emit ChallengeIssuerUpdated(_newChallengeIssuer);
    }

    /**
     * @notice Updates the duration of the response window.
     * @dev Only callable by the contract owner.
     * @param _newResponseWindowBlocks The new duration in blocks.
     */
    function setResponseWindow(uint256 _newResponseWindowBlocks) external onlyOwner isInitialized {
        require(_newResponseWindowBlocks > 0, "PhalaSLA: Response window must be positive");
        responseWindowBlocks = _newResponseWindowBlocks;
        emit ResponseWindowUpdated(_newResponseWindowBlocks);
    }

    /**
     * @notice Pauses the contract, preventing new challenges and responses.
     * @dev Only callable by the contract owner.
     */
    function pause() external onlyOwner isInitialized {
        _pause();
    }

    /**
     * @notice Unpauses the contract.
     * @dev Only callable by the contract owner.
     */
    function unpause() external onlyOwner isInitialized {
        _unpause();
    }

    // --- View Functions ---

    /**
     * @notice Retrieves the details of a specific challenge.
     * @param challengeId The ID of the challenge.
     * @return operator The challenged operator address.
     * @return challengeData The data associated with the challenge.
     * @return responseWindowEndBlock The block number the response window ends.
     * @return responded True if the operator responded, false otherwise.
     * @return reported True if expiry has been reported, false otherwise.
     */
    function getChallengeDetails(uint256 challengeId) external view returns (
        address operator,
        bytes memory challengeData,
        uint256 responseWindowEndBlock,
        bool responded,
        bool reported
    ) {
        Challenge storage c = challenges[challengeId];
        return (
            c.operator,
            c.challengeData,
            c.responseWindowEndBlock,
            c.responded,
            c.reported
        );
    }
} 