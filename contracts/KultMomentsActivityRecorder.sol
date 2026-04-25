// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// @title Kult Moments Activity Recorder
/// @notice Records verified Kult Moments user activity through an authorized backend relayer.
contract KultMomentsActivityRecorder {
    enum ActivityType {
        MomentCreated,
        MomentUpdated,
        MomentDeleted,
        MomentLiked,
        CommentCreated,
        CommentDeleted,
        ReplyCreated,
        SocialPostSubmitted,
        SocialPostValidated,
        AssetMigratedTo0G
    }

    address public owner;
    mapping(address => bool) public activityWriters;
    mapping(bytes32 => bool) public recordedActivities;

    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    event ActivityWriterUpdated(address indexed writer, bool enabled);
    event ActivityRecorded(
        bytes32 indexed activityId,
        address indexed user,
        ActivityType indexed activityType,
        bytes32 momentIdHash,
        bytes32 entityIdHash,
        bytes32 metadataHash,
        uint256 timestamp
    );

    error NotOwner();
    error NotActivityWriter();
    error AlreadyRecorded(bytes32 activityId);
    error ZeroAddress();

    constructor(address initialOwner) {
        if (initialOwner == address(0)) revert ZeroAddress();
        owner = initialOwner;
        emit OwnershipTransferred(address(0), initialOwner);
    }

    modifier onlyOwner() {
        if (msg.sender != owner) revert NotOwner();
        _;
    }

    modifier onlyActivityWriter() {
        if (!activityWriters[msg.sender]) revert NotActivityWriter();
        _;
    }

    function transferOwnership(address newOwner) external onlyOwner {
        if (newOwner == address(0)) revert ZeroAddress();
        emit OwnershipTransferred(owner, newOwner);
        owner = newOwner;
    }

    function setActivityWriter(address writer, bool enabled) external onlyOwner {
        if (writer == address(0)) revert ZeroAddress();
        activityWriters[writer] = enabled;
        emit ActivityWriterUpdated(writer, enabled);
    }

    function recordActivity(
        bytes32 activityId,
        address user,
        ActivityType activityType,
        bytes32 momentIdHash,
        bytes32 entityIdHash,
        bytes32 metadataHash,
        uint256 timestamp
    ) external onlyActivityWriter {
        if (recordedActivities[activityId]) revert AlreadyRecorded(activityId);

        recordedActivities[activityId] = true;

        emit ActivityRecorded(
            activityId,
            user,
            activityType,
            momentIdHash,
            entityIdHash,
            metadataHash,
            timestamp
        );
    }
}
