// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import {ServiceManagerBase, IAVSDirectory, IStakeRegistry, IPermissionController, IAllocationManager} from "@eigenlayer-middleware/src/ServiceManagerBase.sol";
import {IRegistryCoordinator} from "@eigenlayer-middleware/src/interfaces/IRegistryCoordinator.sol";
import {IRewardsCoordinator} from "eigenlayer-contracts/src/contracts/interfaces/IRewardsCoordinator.sol";
import "./interfaces/IPhalaSlaOracle.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

/**
 * @title Service Manager for the Phala Cloud AVS.
 * @notice Manages operator registration, reward distribution proposals, and interacts with the SLA Oracle.
 */
contract PhalaServiceManager is ServiceManagerBase {
    using SafeERC20 for IERC20;

    // --- State Variables ---

    /// @notice Address of the Phala SLA Oracle contract.
    IPhalaSlaOracle public immutable phalaSlaOracle;

    /// @notice Address authorized to submit reward proposals and manage tokenomics.
    address public tokenomicManager;

    /// @notice Address of the PHA token used for staking (if required by rewards/slashing).
    IERC20 public immutable phaToken;

    /// @notice Address of vPHA token used for rewards.
    IERC20 public immutable vPhaToken;

    /// @notice Mapping to track registered operators and potentially store attestation metadata hash.
    mapping(address => bytes32) public registeredOperatorAttestationHash;

    /// @notice Whether the contract has been initialized.
    bool public initialized;

    // --- Events ---

    /// @notice Emitted when an operator submits an attestation for registration.
    event OperatorAttestationReceived(address indexed operator, bytes attestationData);

    /// @notice Emitted when an operator is successfully registered after attestation submission.
    event OperatorRegistered(address indexed operator, bytes32 attestationHash);

    /// @notice Emitted when an operator is deregistered.
    event OperatorDeregistered(address indexed operator);

    /// @notice Emitted when the Tokenomic Manager address is updated.
    event TokenomicManagerUpdated(address indexed newManager);

    /// @notice Emitted when a liveness failure is recorded from the SLA Oracle.
    event OperatorLivenessFailureRecorded(address indexed operator);

    // --- Modifiers ---

    /// @notice Ensures the caller is the authorized Tokenomic Manager.
    modifier onlyTokenomicManager() {
        require(msg.sender == tokenomicManager, "PhalaSM: Caller is not the Tokenomic Manager");
        _;
    }

    /// @notice Ensures the caller is the Phala SLA Oracle contract.
    modifier onlySlaOracle() {
        require(msg.sender == address(phalaSlaOracle), "PhalaSM: Caller is not the SLA Oracle");
        _;
    }

    /// @notice Ensures the contract is initialized.
    modifier isInitialized() {
        require(initialized, "PhalaSM: Contract is not initialized");
        _;
    }

    // --- Constructor ---

    constructor(
        IAVSDirectory _avsDirectory,
        IRegistryCoordinator _registryCoordinator,
        IStakeRegistry _stakeRegistry,
        IRewardsCoordinator _rewardsCoordinator,
        address _phalaSlaOracle,
        address _phaToken,
        address _vPhaToken,
        IPermissionController _permissionController,
        IAllocationManager _allocationManager
    ) ServiceManagerBase(
            _avsDirectory,
            _rewardsCoordinator,
            _registryCoordinator,
            _stakeRegistry,
            _permissionController,
            _allocationManager
          )
    {
        require(_phalaSlaOracle != address(0), "PhalaSM: Zero address for SLA Oracle");
        require(_phaToken != address(0), "PhalaSM: Zero address for PHA token");
        require(_vPhaToken != address(0), "PhalaSM: Zero address for vPHA token");
        phalaSlaOracle = IPhalaSlaOracle(_phalaSlaOracle);
        phaToken = IERC20(_phaToken);
        vPhaToken = IERC20(_vPhaToken);
        // Tokenomic Manager is set via initialize
    }

    // --- Initialization ---

    /// @notice Initializes the contract owner, rewards initiator, and Tokenomic Manager.
    /// @param _initialOwner The address to be set as the initial owner.
    /// @param _rewardsInitiator The address authorized to initiate reward cycles.
    /// @param _initialTokenomicManager The address authorized for tokenomic management.
    function initialize(
        address _initialOwner,
        address _rewardsInitiator,
        address _initialTokenomicManager
    ) external {
        require(_initialTokenomicManager != address(0), "PhalaSM: Zero address for Tokenomic Manager");
        require(!initialized, "PhalaSM: Contract is already initialized");
        __ServiceManagerBase_init(_initialOwner, _rewardsInitiator);
        tokenomicManager = _initialTokenomicManager;
        emit TokenomicManagerUpdated(_initialTokenomicManager);
        initialized = true;
    }

    // --- Reward Management ---

    /**
     * @notice Submits operator-directed reward proposals to the EigenLayer Rewards Coordinator.
     * @dev Only callable by the Tokenomic Manager.
     *      Requires this contract to hold sufficient PHA tokens and approve the RewardsCoordinator.
     * @param operatorDirectedRewardsSubmissions A list of reward submissions detailing operator and staker rewards.
     */
    function submitOperatorDirectedRewardProposal(
        IRewardsCoordinator.OperatorDirectedRewardsSubmission[] calldata operatorDirectedRewardsSubmissions
    ) external onlyTokenomicManager isInitialized {
        uint256 totalPhaRequired = _calculateTotalPhaRequired(operatorDirectedRewardsSubmissions);
        
        require(phaToken.balanceOf(address(this)) >= totalPhaRequired, "PhalaSM: Insufficient PHA balance");

        // Approve the RewardsCoordinator to spend the required PHA amount
        // Using safeApprove with 0 allowance first to prevent front-running/double-spending vulnerabilities
        phaToken.safeApprove(address(_rewardsCoordinator), 0);
        phaToken.safeApprove(address(_rewardsCoordinator), totalPhaRequired);

        // Submit the proposal to the Rewards Coordinator, the ServiceManagerBase already has the correct permissions
        createOperatorDirectedAVSRewardsSubmission(
            operatorDirectedRewardsSubmissions
        );
    }

    /**
     * @notice Internal function to calculate the total PHA needed for a batch of reward submissions.
     * @dev Assumes PHA is the reward token specified within the submissions.
     * @param submissions The array of reward submissions.
     * @return totalPha The total amount of PHA required.
     */
    function _calculateTotalPhaRequired(
        IRewardsCoordinator.OperatorDirectedRewardsSubmission[] calldata submissions
    ) internal view returns (uint256 totalPha) {
        address _phaToken = address(phaToken);
        for (uint i = 0; i < submissions.length; ++i) {
            // Ensure the submission is using the correct PHA token
            require(address(submissions[i].token) == _phaToken, "PhalaSM: Reward token must be PHA");

            IRewardsCoordinator.OperatorReward[] calldata operatorRewards = submissions[i].operatorRewards;
            for (uint j = 0; j < operatorRewards.length; ++j) {
                totalPha += operatorRewards[j].amount;
            }
            // Note: Staker rewards are distributed proportionally by the RewardsCoordinator based on 
            // the total operator reward and delegation, not double-counted here.
        }
        return totalPha;
    }

    // --- Slashing / Liveness ---

    /**
     * @notice Records that an operator has failed a liveness check, as reported by the SLA Oracle.
     * @dev Only callable by the Phala SLA Oracle.
     *      Currently only emits an event. Future implementation may trigger slashing.
     * @param operator The address of the operator who failed the check.
     */
    function recordOperatorLivenessFailure(address operator) external onlySlaOracle isInitialized {
        require(registeredOperatorAttestationHash[operator] != bytes32(0), "PhalaSM: Operator not registered");
        // TODO: Implement slashing logic integration when EigenLayer Slasher is finalized.
        // slasher.freezeOperator(operator); 
        emit OperatorLivenessFailureRecorded(operator);
    }

    // --- Admin Functions ---

    /**
     * @notice Updates the address of the Tokenomic Manager.
     * @dev Only callable by the contract owner.
     * @param _newTokenomicManager The new address for the Tokenomic Manager.
     */
    function setTokenomicManager(address _newTokenomicManager) external onlyOwner isInitialized {
        require(_newTokenomicManager != address(0), "PhalaSM: Cannot set zero address");
        tokenomicManager = _newTokenomicManager;
        emit TokenomicManagerUpdated(_newTokenomicManager);
    }

     // --- View Functions ---

    /**
     * @notice Checks if an operator is registered.
     * @param operator The address of the operator.
     * @return bool True if the operator is registered, false otherwise.
     */
    function isOperatorRegistered(address operator) public view returns (bool) {
        return registeredOperatorAttestationHash[operator] != bytes32(0);
    }
} 