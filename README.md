# WZN Burn-to-Play Game on Solana

A crypto-native multiplayer card game built on Solana where players stake WZN tokens to unlock weekly play quotas. The system features a burn mechanism, staking rewards, and a recovery vault, all governed by a DAO with backup voting logic.

## 🎮 Game Overview

The WZN Burn-to-Play Game is designed with the following core principles:

- **Burn-to-Play**: Players must burn WZN tokens to participate in games
- **Staking Rewards**: Players earn ~4% APY by staking WZN tokens
- **Weekly Quotas**: Staking rewards unlock limited weekly play quotas
- **Non-custodial**: Users maintain control of their funds until used
- **DAO Governance**: Community-driven governance with backup recovery

## 🔧 Smart Contract Features

### Core Components

1. **Game Management**
   - Burn-to-play mechanism
   - Weekly quota system
   - Player state tracking

2. **Staking Pool**
   - 4% APY rewards
   - Weekly reward distribution
   - Permissioned withdrawals via DAO

3. **Vault System**
   - Secure token storage
   - PDA-based access control
   - Emergency recovery mechanisms

4. **Governance & Recovery**
   - DAO-controlled operations
   - Backup multisig for emergencies
   - Slashing mechanism for malicious actors

## 🏗️ Architecture

### Account Structure

- **GameState**: Global game configuration and statistics
- **StakingPool**: Staking pool management and rewards
- **RecoveryVault**: Emergency fund storage and recovery
- **PlayerState**: Individual player staking and game data
- **PlayerQuota**: Weekly play quota tracking

### PDA Seeds

- `game_state`: Global game state
- `staking_pool`: Staking pool management
- `recovery_vault`: Emergency vault
- `player_state`: Player-specific data
- `player_quota`: Weekly quota tracking
- `staking_vault`: Staked token storage
- `reward_vault`: Reward distribution
- `game_vault`: Game fee collection

## 🚀 Getting Started

### Prerequisites

- Rust 1.70+
- Solana CLI 1.17+
- Anchor Framework 0.29+

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd wzn-burn-play
```

2. Install dependencies:
```bash
cargo build
```

3. Build the program:
```bash
anchor build
```

4. Deploy to localnet:
```bash
anchor deploy
```

### Testing

Run the test suite:
```bash
anchor test
```

## 📋 Instructions

### Game Management

- `initialize_game`: Initialize the game with DAO authority and backup team
- `burn_to_play`: Burn WZN tokens to play a game
- `use_quota_play`: Use weekly quota for free play

### Staking Management

- `initialize_staking_pool`: Initialize the staking pool
- `stake_tokens`: Stake WZN tokens to earn rewards
- `claim_rewards`: Claim weekly staking rewards
- `withdraw_stake`: Withdraw staked tokens (DAO approval required)

### Vault Management

- `initialize_vault`: Initialize the recovery vault
- `deposit_to_vault`: Deposit tokens to the recovery vault
- `dao_withdraw`: DAO-controlled withdrawals
- `backup_withdraw`: Emergency withdrawals by backup team

### Governance

- `update_governance`: Update DAO authority
- `update_backup_team`: Update backup team members
- `slash_stake`: Slash player stakes for violations

## 🔐 Security Features

### Access Control

- **PDA Authorities**: All critical operations use program-derived addresses
- **DAO Governance**: Primary control mechanism for major operations
- **Backup Recovery**: Emergency access for backup team after inactivity
- **Slashing**: Penalty mechanism for malicious behavior

### Emergency Recovery

The system includes a sophisticated recovery mechanism:

1. **DAO Inactivity**: If DAO doesn't interact for 6 months
2. **Low Balance**: If vault balance drops below threshold
3. **Backup Access**: 3-member multisig can access emergency funds
4. **Limited Withdrawal**: Emergency withdrawals capped at 25% of vault

## 💰 Tokenomics

### Staking Rewards

- **APY**: 4% annual yield
- **Distribution**: Weekly rewards to player quota accounts
- **Usage**: Rewards can only be used for gameplay, not withdrawn

### Burn Mechanism

- **Game Cost**: Variable burn amount per game
- **Tracking**: All burns tracked on-chain
- **Deflationary**: Reduces total WZN supply

### Quota System

- **Weekly Reset**: Quotas reset every 7 days
- **Staking-Based**: Quota size based on staked amount
- **Anti-Farming**: Prevents spam and farming

## 🧪 Testing

The project includes comprehensive tests covering:

- Game initialization and configuration
- Staking and reward mechanisms
- Burn-to-play functionality
- Vault security and recovery
- Governance operations
- Emergency scenarios

## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## 📞 Support

For questions and support:
- Create an issue on GitHub
- Join our Discord community
- Check the documentation

## ⚠️ Disclaimer

This software is provided as-is without any guarantees. Users should conduct their own security audits and due diligence before using this code in production environments. 