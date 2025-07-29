use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::state::*;

// Game Management Accounts

#[derive(Accounts)]
pub struct InitializeGame<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 1 + 32 + 96 + 32 + 4 + 8 + 8 + 1 + 1 + 8 + 8,
        seeds = [GAME_STATE_SEED],
        bump
    )]
    pub game_state: Account<'info, GameState>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub wzn_mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct BurnToPlay<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    #[account(
        mut,
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump,
        constraint = game_state.is_initialized
    )]
    pub game_state: Account<'info, GameState>,

    #[account(
        mut,
        constraint = player_token_account.owner == player.key(),
        constraint = player_token_account.mint == game_state.wzn_mint
    )]
    pub player_token_account: Account<'info, TokenAccount>,

    pub wzn_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = player,
        space = 8 + 1 + 32 + 8 + 8 + 8 + 4 + 8 + 1 + 8,
        seeds = [PLAYER_STATE_SEED, player.key().as_ref()],
        bump
    )]
    pub player_state: Account<'info, PlayerState>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UseQuotaPlay<'info> {
    #[account(
        mut,
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump,
        constraint = game_state.is_initialized
    )]
    pub game_state: Account<'info, GameState>,

    #[account(
        mut,
        init_if_needed,
        payer = player,
        space = 8 + 1 + 32 + 4 + 4 + 8 + 8,
        seeds = [PLAYER_QUOTA_SEED, player.key().as_ref()],
        bump
    )]
    pub player_quota: Account<'info, PlayerQuota>,

    pub player: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

// Staking Management Accounts

#[derive(Accounts)]
pub struct InitializeStakingPool<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 1 + 8 + 8 + 8 + 8 + 8 + 1,
        seeds = [STAKING_POOL_SEED],
        bump
    )]
    pub staking_pool: Account<'info, StakingPool>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    #[account(
        mut,
        seeds = [STAKING_POOL_SEED],
        bump = staking_pool.bump,
        constraint = staking_pool.is_initialized
    )]
    pub staking_pool: Account<'info, StakingPool>,

    #[account(
        mut,
        constraint = player_token_account.owner == player.key()
    )]
    pub player_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [STAKING_VAULT_SEED],
        bump
    )]
    pub staking_vault: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = player,
        space = 8 + 1 + 32 + 8 + 8 + 8 + 4 + 8 + 1 + 8,
        seeds = [PLAYER_STATE_SEED, player.key().as_ref()],
        bump
    )]
    pub player_state: Account<'info, PlayerState>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(
        mut,
        seeds = [STAKING_POOL_SEED],
        bump = staking_pool.bump,
        constraint = staking_pool.is_initialized
    )]
    pub staking_pool: Account<'info, StakingPool>,

    #[account(
        mut,
        seeds = [PLAYER_STATE_SEED, player.key().as_ref()],
        bump = player_state.bump
    )]
    pub player_state: Account<'info, PlayerState>,

    #[account(
        mut,
        seeds = [REWARD_VAULT_SEED],
        bump
    )]
    pub reward_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [PLAYER_QUOTA_SEED, player.key().as_ref()],
        bump,
        constraint = player_quota_account.owner == player.key()
    )]
    pub player_quota_account: Account<'info, TokenAccount>,

    #[account(
        seeds = [STAKING_VAULT_SEED],
        bump
    )]
    pub staking_authority: AccountInfo<'info>,

    pub player: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct WithdrawStake<'info> {
    #[account(
        mut,
        seeds = [STAKING_POOL_SEED],
        bump = staking_pool.bump,
        constraint = staking_pool.is_initialized
    )]
    pub staking_pool: Account<'info, StakingPool>,

    #[account(
        mut,
        seeds = [PLAYER_STATE_SEED, player.key().as_ref()],
        bump = player_state.bump
    )]
    pub player_state: Account<'info, PlayerState>,

    #[account(
        mut,
        constraint = player_token_account.owner == player.key()
    )]
    pub player_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [STAKING_VAULT_SEED],
        bump
    )]
    pub staking_vault: Account<'info, TokenAccount>,

    #[account(
        seeds = [STAKING_VAULT_SEED],
        bump
    )]
    pub staking_authority: AccountInfo<'info>,

    #[account(
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump
    )]
    pub game_state: Account<'info, GameState>,

    pub dao_authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

// Vault Management Accounts

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 1 + 8 + 8 + 8 + 8 + 1 + 1,
        seeds = [RECOVERY_VAULT_SEED],
        bump
    )]
    pub recovery_vault: Account<'info, RecoveryVault>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct DepositToVault<'info> {
    #[account(mut)]
    pub from: Signer<'info>,

    #[account(
        mut,
        constraint = from_account.owner == from.key()
    )]
    pub from_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [RECOVERY_VAULT_SEED],
        bump = recovery_vault.bump,
        constraint = recovery_vault.is_initialized
    )]
    pub recovery_vault: Account<'info, RecoveryVault>,

    #[account(
        mut,
        seeds = [RECOVERY_VAULT_SEED],
        bump
    )]
    pub vault_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct DAOWithdraw<'info> {
    #[account(
        mut,
        seeds = [RECOVERY_VAULT_SEED],
        bump = recovery_vault.bump,
        constraint = recovery_vault.is_initialized
    )]
    pub recovery_vault: Account<'info, RecoveryVault>,

    #[account(
        mut,
        seeds = [RECOVERY_VAULT_SEED],
        bump
    )]
    pub vault_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub recipient: Account<'info, TokenAccount>,

    #[account(
        seeds = [RECOVERY_VAULT_SEED],
        bump
    )]
    pub vault_authority: AccountInfo<'info>,

    #[account(
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump
    )]
    pub game_state: Account<'info, GameState>,

    pub dao_authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BackupWithdraw<'info> {
    #[account(
        mut,
        seeds = [RECOVERY_VAULT_SEED],
        bump = recovery_vault.bump,
        constraint = recovery_vault.is_initialized
    )]
    pub recovery_vault: Account<'info, RecoveryVault>,

    #[account(
        mut,
        seeds = [RECOVERY_VAULT_SEED],
        bump
    )]
    pub vault_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub recipient: Account<'info, TokenAccount>,

    #[account(
        seeds = [RECOVERY_VAULT_SEED],
        bump
    )]
    pub vault_authority: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump
    )]
    pub game_state: Account<'info, GameState>,

    pub backup_authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

// Governance Accounts

#[derive(Accounts)]
pub struct UpdateGovernance<'info> {
    #[account(
        mut,
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump,
        constraint = game_state.is_initialized
    )]
    pub game_state: Account<'info, GameState>,

    pub dao_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateBackupTeam<'info> {
    #[account(
        mut,
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump,
        constraint = game_state.is_initialized
    )]
    pub game_state: Account<'info, GameState>,

    pub dao_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SlashStake<'info> {
    #[account(
        mut,
        seeds = [STAKING_POOL_SEED],
        bump = staking_pool.bump,
        constraint = staking_pool.is_initialized
    )]
    pub staking_pool: Account<'info, StakingPool>,

    #[account(
        mut,
        seeds = [PLAYER_STATE_SEED, player.key().as_ref()],
        bump = player_state.bump
    )]
    pub player_state: Account<'info, PlayerState>,

    #[account(mut)]
    pub player: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [STAKING_VAULT_SEED],
        bump
    )]
    pub staking_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [GAME_VAULT_SEED],
        bump
    )]
    pub game_vault: Account<'info, TokenAccount>,

    #[account(
        seeds = [STAKING_VAULT_SEED],
        bump
    )]
    pub staking_authority: AccountInfo<'info>,

    #[account(
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump
    )]
    pub game_state: Account<'info, GameState>,

    pub dao_authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
} 