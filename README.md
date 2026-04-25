# TeachLink Contract

TeachLink: Decentralized Knowledge-Sharing on Stellar

## Overview

TeachLink is a platform for technocrats where knowledge, skills, ideas, and information that can bring about development and improvement can be shared, dissected, and you can also earn from it.

## Features

- **Rewards System**: Secure reward distribution with overflow protection
- **Escrow Services**: Multi-signature escrow contracts
- **Bridge Protocol**: Cross-chain asset transfers
- **Liquidity Pools**: DeFi liquidity provision
- **Atomic Swaps**: Trustless token exchanges
- **Mobile Platform**: Mobile app integration

## Security

This contract includes comprehensive security measures:

- **Overflow Protection**: All arithmetic operations use checked arithmetic
- **Reentrancy Guards**: Protection against recursive calls
- **Input Validation**: Comprehensive parameter validation
- **Access Control**: Role-based permissions

## Recent Updates

### Overflow Protection Fixes (#235)

- ✅ Added checked arithmetic operations in rewards calculation
- ✅ Implemented input range validation for rewards and rates
- ✅ Added graceful overflow handling with proper error messages
- ✅ Added comprehensive overflow test cases
- ✅ Introduced MAX_REWARD_RATE constant for rate validation
- ✅ Enhanced RewardsValidator with reward-specific validation
- ✅ Added utility functions for safe arithmetic operations

## Installation

```bash
cargo build --release
```

## Testing

```bash
cargo test
```

## License

This project is licensed under the MIT License.
