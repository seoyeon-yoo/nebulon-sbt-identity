// SPDX-License-Identifier: MIT
pragma version ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Permit.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title NebulonToken
 * @dev ERC20 token for Nebulon with Transfer Fee and Interest-bearing features.
 */
contract NebulonToken is ERC20, ERC20Permit, Ownable {
    uint256 public transferFeeBasisPoints = 100; // 1%
    uint256 public constant MAX_FEE_BASIS_POINTS = 500; // 5%
    address public feeCollector;

    // Interest bearing simulation: Balance increases over time via a multiplier
    uint256 private _baseMultiplier = 1e18;
    uint256 public lastUpdateTimestamp;
    uint256 public interestRatePerSecond = 317097919; // ~1% per year (1e18 scale)

    mapping(address => uint256) private _rawBalances;
    uint256 private _totalRawSupply;

    constructor(address initialOwner) 
        ERC20("Nebulon", "NEBU") 
        ERC20Permit("Nebulon")
        Ownable(initialOwner)
    {
        feeCollector = initialOwner;
        lastUpdateTimestamp = block.timestamp;
        _mint(initialOwner, 1_000_000_000 * 10**decimals());
    }

    function setTransferFee(uint256 feeBasisPoints) external onlyOwner {
        require(feeBasisPoints <= MAX_FEE_BASIS_POINTS, "Fee too high");
        transferFeeBasisPoints = feeBasisPoints;
    }

    function setFeeCollector(address _feeCollector) external onlyOwner {
        feeCollector = _feeCollector;
    }

    /**
     * @dev Overridden transfer to include fee logic.
     */
    function _update(address from, address to, uint256 amount) internal virtual override {
        if (from == address(0) || to == address(0) || from == owner() || to == owner()) {
            super._update(from, to, amount);
        } else {
            uint256 fee = (amount * transferFeeBasisPoints) / 10000;
            uint256 amountAfterFee = amount - fee;
            super._update(from, feeCollector, fee);
            super._update(from, to, amountAfterFee);
        }
    }

    /**
     * @dev Simple interest bearing logic: increase the global multiplier.
     * In a real production contract, this would be more complex to ensure precision.
     */
    function currentMultiplier() public view returns (uint256) {
        uint256 timePassed = block.timestamp - lastUpdateTimestamp;
        return _baseMultiplier + (timePassed * interestRatePerSecond);
    }

    // Note: To implement full interest-bearing where balances auto-increase, 
    // we'd need to override balanceOf and transfer logic significantly.
    // For this conversion, we provide the logic to 'accrue' interest.

    function accrueInterest() public {
        _baseMultiplier = currentMultiplier();
        lastUpdateTimestamp = block.timestamp;
    }

    function decimals() public view virtual override returns (uint8) {
        return 9; // Matching Solana Token-2022 default for many tokens
    }
}
