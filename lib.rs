pub mod errors;
pub mod state;
pub mod instructions;
pub mod accounts;

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};

declare_id!("WZNBurnPlay111111111111111111111111111111111");

use instructions::*;
use state::*;
use accounts::*;

#[program]
pub mod wzn_burn_play {
    use super::*;

    // Game Management
    pub fn initialize_game(ctx: Context<InitializeGame>, weekly_quota: u32) -> Result<()> {
        instructions::initialize_game(ctx, weekly_quota)
    }

    pub fn burn_to_play(ctx: Context<BurnToPlay>, amount: u64) -> Result<()> {
        instructions::burn_to_play(ctx, amount)
    }

    pub fn use_quota_play(ctx: Context<UseQuotaPlay>) -> Result<()> {
        instructions::use_quota_play(ctx)
    }

    // Staking Management
    pub fn initialize_staking_pool(ctx: Context<InitializeStakingPool>) -> Result<()> {
        instructions::initialize_staking_pool(ctx)
    }

    pub fn stake_tokens(ctx: Context<StakeTokens>, amount: u64) -> Result<()> {
        instructions::stake_tokens(ctx, amount)
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        instructions::claim_rewards(ctx)
    }

    pub fn withdraw_stake(ctx: Context<WithdrawStake>, amount: u64) -> Result<()> {
        instructions::withdraw_stake(ctx, amount)
    }

    // Vault Management
    pub fn initialize_vault(ctx: Context<InitializeVault>, backup_threshold: u64) -> Result<()> {
        instructions::initialize_vault(ctx, backup_threshold)
    }

    pub fn deposit_to_vault(ctx: Context<DepositToVault>, amount: u64) -> Result<()> {
        instructions::deposit_to_vault(ctx, amount)
    }

    pub fn dao_withdraw(ctx: Context<DAOWithdraw>, amount: u64) -> Result<()> {
        instructions::dao_withdraw(ctx, amount)
    }

    pub fn backup_withdraw(ctx: Context<BackupWithdraw>, amount: u64) -> Result<()> {
        instructions::backup_withdraw(ctx, amount)
    }

    // Governance
    pub fn update_governance(ctx: Context<UpdateGovernance>, new_dao: Pubkey) -> Result<()> {
        instructions::update_governance(ctx, new_dao)
    }

    pub fn update_backup_team(ctx: Context<UpdateBackupTeam>, new_team: [Pubkey; 3]) -> Result<()> {
        instructions::update_backup_team(ctx, new_team)
    }

    pub fn slash_stake(ctx: Context<SlashStake>, amount: u64) -> Result<()> {
        instructions::slash_stake(ctx, amount)
    }
} 