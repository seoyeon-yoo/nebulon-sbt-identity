// SPDX-License-Identifier: MIT
pragma version ^0.8.20;

import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/token/ERC721/extensions/ERC721URIStorage.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/Strings.sol";

/**
 * @title NebulonSBTIdentity
 * @dev Soulbound Token (SBT) for Nebulon Identity on EVM.
 * Matches features: Vanity-like naming, Tier system, Dynamic metadata, Admin management.
 */
contract NebulonSBTIdentity is ERC721, ERC721URIStorage, AccessControl {
    using Strings for uint256;

    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");

    struct Identity {
        string handle;      // @handle
        bytes hexId;        // 512 bytes equivalent
        uint256 score;
        uint8 tier;
        bool isActive;
        uint256 lastClaimTs;
        mapping(string => string) sns;
        bytes privateVault;
        uint256 recommendations;
        uint256 reports;
    }

    uint256 private _nextTokenId;
    uint256 public totalAgents;
    uint256 public totalScore;

    mapping(uint256 => Identity) private _identities;
    mapping(string => uint256) private _handleToTokenId;
    
    // Fee constants
    uint256 public constant BASE_FEE = 0.01 ether;
    uint256 public constant INCREMENT = 0.00001 ether;
    uint256 public constant MAX_FEE = 0.02 ether;

    event IdentityIssued(address indexed owner, uint256 tokenId, string handle);
    event StatusUpdated(uint256 tokenId, uint256 score, uint8 tier, string uri);
    event SNSRecordUpdated(uint256 tokenId, string platform, string handle, bool removed);

    constructor() ERC721("Nebulon Identity", "NEBU-ID") {
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(ADMIN_ROLE, msg.sender);
    }

    /**
     * @dev Validates handle format (@lowercase)
     */
    function _validateHandle(string memory handle) internal pure returns (bool) {
        bytes memory b = bytes(handle);
        if (b.length < 2 || b[0] != "@") return false;
        for (uint i = 1; i < b.length; i++) {
            if (!(b[i] >= 0x61 && b[i] <= 0x7A) && !(b[i] >= 0x30 && b[i] <= 0x39)) return false;
        }
        return true;
    }

    function calculateFee() public view returns (uint256) {
        uint256 fee = BASE_FEE + (totalAgents * INCREMENT);
        return fee > MAX_FEE ? MAX_FEE : fee;
    }

    /**
     * @dev Issue a new SBT Identity.
     */
    function issueIdentity(
        string memory handle,
        bytes memory hexId,
        string memory uri
    ) public payable returns (uint256) {
        require(_validateHandle(handle), "Invalid handle format");
        require(_handleToTokenId[handle] == 0, "Handle already taken");
        require(msg.value >= calculateFee(), "Insufficient fee");

        uint256 tokenId = ++_nextTokenId;
        _safeMint(msg.sender, tokenId);
        _setTokenURI(tokenId, uri);

        Identity storage id = _identities[tokenId];
        id.handle = handle;
        id.hexId = hexId;
        id.isActive = true;
        id.tier = 10;
        id.lastClaimTs = block.timestamp;

        _handleToTokenId[handle] = tokenId;
        totalAgents++;

        emit IdentityIssued(msg.sender, tokenId, handle);
        return tokenId;
    }

    /**
     * @dev Admin updates score, tier and metadata URI.
     */
    function updateAgentStatus(
        uint256 tokenId,
        uint256 newScore,
        uint8 tier,
        string memory newUri
    ) public onlyRole(ADMIN_ROLE) {
        require(_ownerOf(tokenId) != address(0), "Identity does not exist");
        require(tier >= 1 && tier <= 10, "Invalid tier");

        Identity storage id = _identities[tokenId];
        totalScore = totalScore - id.score + newScore;
        id.score = newScore;
        id.tier = tier;
        
        _setTokenURI(tokenId, newUri);
        emit StatusUpdated(tokenId, newScore, tier, newUri);
    }

    function updateSns(
        uint256 tokenId,
        string memory platform,
        string memory handle,
        bool remove
    ) public onlyRole(ADMIN_ROLE) {
        require(_ownerOf(tokenId) != address(0), "Identity does not exist");
        Identity storage id = _identities[tokenId];
        if (remove) {
            delete id.sns[platform];
        } else {
            id.sns[platform] = handle;
        }
        emit SNSRecordUpdated(tokenId, platform, handle, remove);
    }

    function updatePrivateVault(uint256 tokenId, bytes memory encryptedData) public {
        require(ownerOf(tokenId) == msg.sender, "Not identity owner");
        _identities[tokenId].privateVault = encryptedData;
    }

    // Soulbound implementation: override transfers to revert
    function transferFrom(address from, address to, uint256 tokenId) public virtual override(ERC721, IERC721) {
        revert("Soulbound: Transfer not allowed");
    }

    function safeTransferFrom(address from, address to, uint256 tokenId, bytes memory data) public virtual override(ERC721, IERC721) {
        revert("Soulbound: Transfer not allowed");
    }

    // Getters
    function getIdentity(uint256 tokenId) public view returns (
        string memory handle,
        uint256 score,
        uint8 tier,
        bool isActive,
        uint256 recommendations,
        uint256 reports
    ) {
        Identity storage id = _identities[tokenId];
        return (id.handle, id.score, id.tier, id.isActive, id.recommendations, id.reports);
    }

    function getSnsHandle(uint256 tokenId, string memory platform) public view returns (string memory) {
        return _identities[tokenId].sns[platform];
    }

    function withdraw() public onlyRole(DEFAULT_ADMIN_ROLE) {
        payable(msg.sender).transfer(address(this).balance);
    }

    // Required overrides for ERC721URIStorage
    function tokenURI(uint256 tokenId) public view override(ERC721, ERC721URIStorage) returns (string memory) {
        return super.tokenURI(tokenId);
    }

    function supportsInterface(bytes4 interfaceId) public view override(ERC721, ERC721URIStorage, AccessControl) returns (bool) {
        return super.supportsInterface(interfaceId);
    }
}
