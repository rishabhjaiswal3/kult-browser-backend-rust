// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

/**
 * Unified multi-game marketplace supporting USDT & USDC.
 * Items are keyed by gameId → category → itemId.
 * Owner can add games, categories, and items at any time.
 */
contract UnifiedGameMarketplace is Ownable, ReentrancyGuard {
    using SafeERC20 for IERC20;

    // ─── Types ────────────────────────────────────────────────────────────────

    struct Item {
        string  name;
        uint256 price;      // 6-decimal units (USDT/USDC standard)
        bool    enabled;
        string  itemType;   // "gun" | "car" | "skin" | "coin" | "gem" | …
        string  metadata;   // IPFS CID or JSON string for image/rarity/etc.
        bool    consumable; // if true, the same address can buy it again
    }

    // ─── Storage ──────────────────────────────────────────────────────────────

    // gameId → category → itemId → Item
    mapping(string => mapping(string => mapping(string => Item))) private _items;

    // buyer → gameId → category → itemId → quantity owned
    mapping(address => mapping(string => mapping(string => mapping(string => uint256)))) public owned;

    // Accepted payment tokens (USDT, USDC, …)
    mapping(address => bool) public acceptedTokens;

    // Where all payments go
    address public treasury;

    // ─── Events ───────────────────────────────────────────────────────────────

    event ItemPurchased(
        bytes32 indexed orderId,
        address indexed buyer,
        string  gameId,
        string  category,
        string  itemId,
        address paymentToken,
        uint256 price
    );

    event ItemSet(
        string  gameId,
        string  category,
        string  itemId,
        string  name,
        uint256 price,
        bool    enabled,
        bool    consumable
    );

    event TokenUpdated(address indexed token, bool accepted);
    event TreasuryUpdated(address indexed oldTreasury, address indexed newTreasury);

    // ─── Constructor ──────────────────────────────────────────────────────────

    constructor(
        address          _treasury,
        address[] memory _acceptedTokens
    ) Ownable(msg.sender) {
        require(_treasury != address(0), "Zero treasury");
        treasury = _treasury;

        for (uint256 i = 0; i < _acceptedTokens.length; i++) {
            require(_acceptedTokens[i] != address(0), "Zero token");
            acceptedTokens[_acceptedTokens[i]] = true;
            emit TokenUpdated(_acceptedTokens[i], true);
        }
    }

    // ─── Admin: items ─────────────────────────────────────────────────────────

    function setItem(
        string calldata gameId,
        string calldata category,
        string calldata itemId,
        string calldata name,
        uint256         price,
        bool            enabled,
        string calldata itemType,
        string calldata metadata,
        bool            consumable
    ) external onlyOwner {
        _items[gameId][category][itemId] = Item(name, price, enabled, itemType, metadata, consumable);
        emit ItemSet(gameId, category, itemId, name, price, enabled, consumable);
    }

    /// Batch-set items in a single transaction. Arrays must all be equal length.
    function setItemBatch(
        string[] calldata gameIds,
        string[] calldata categories,
        string[] calldata itemIds,
        string[] calldata names,
        uint256[] calldata prices,
        bool[] calldata enableds,
        string[] calldata itemTypes,
        string[] calldata metadatas,
        bool[] calldata consumables
    ) external onlyOwner {
        uint256 len = gameIds.length;
        require(
            len == categories.length &&
            len == itemIds.length    &&
            len == names.length      &&
            len == prices.length     &&
            len == enableds.length   &&
            len == itemTypes.length  &&
            len == metadatas.length  &&
            len == consumables.length,
            "Array length mismatch"
        );

        for (uint256 i = 0; i < len; i++) {
            _items[gameIds[i]][categories[i]][itemIds[i]] = Item(
                names[i], prices[i], enableds[i], itemTypes[i], metadatas[i], consumables[i]
            );
            emit ItemSet(gameIds[i], categories[i], itemIds[i], names[i], prices[i], enableds[i], consumables[i]);
        }
    }

    function setItemEnabled(
        string calldata gameId,
        string calldata category,
        string calldata itemId,
        bool            enabled
    ) external onlyOwner {
        _items[gameId][category][itemId].enabled = enabled;
    }

    // ─── Admin: tokens & treasury ─────────────────────────────────────────────

    function setAcceptedToken(address token, bool accepted) external onlyOwner {
        require(token != address(0), "Zero token");
        acceptedTokens[token] = accepted;
        emit TokenUpdated(token, accepted);
    }

    function setTreasury(address _treasury) external onlyOwner {
        require(_treasury != address(0), "Zero treasury");
        emit TreasuryUpdated(treasury, _treasury);
        treasury = _treasury;
    }

    // ─── Purchase ─────────────────────────────────────────────────────────────

    /**
     * @param gameId        Game identifier (e.g. "warzone", "racers")
     * @param category      Category (e.g. "guns", "cars", "skins", "coins")
     * @param itemId        Unique item ID (e.g. "awp", "ferrari", "gold_skin")
     * @param paymentToken  USDT or USDC contract address
     * @param orderId       Unique order hash generated by the frontend
     */
    function purchase(
        string  calldata gameId,
        string  calldata category,
        string  calldata itemId,
        address          paymentToken,
        bytes32          orderId
    ) external nonReentrant {
        require(acceptedTokens[paymentToken], "Token not accepted");

        Item storage item = _items[gameId][category][itemId];
        require(item.enabled, "Item not available");
        require(item.price > 0, "Item has no price");

        if (!item.consumable) {
            require(owned[msg.sender][gameId][category][itemId] == 0, "Already owned");
        }

        IERC20(paymentToken).safeTransferFrom(msg.sender, treasury, item.price);

        owned[msg.sender][gameId][category][itemId] += 1;

        emit ItemPurchased(orderId, msg.sender, gameId, category, itemId, paymentToken, item.price);
    }

    // ─── Views ────────────────────────────────────────────────────────────────

    function getItem(
        string calldata gameId,
        string calldata category,
        string calldata itemId
    ) external view returns (Item memory) {
        return _items[gameId][category][itemId];
    }

    function ownsItem(
        address         buyer,
        string calldata gameId,
        string calldata category,
        string calldata itemId
    ) external view returns (bool) {
        return owned[buyer][gameId][category][itemId] > 0;
    }

    function ownedQuantity(
        address         buyer,
        string calldata gameId,
        string calldata category,
        string calldata itemId
    ) external view returns (uint256) {
        return owned[buyer][gameId][category][itemId];
    }
}
