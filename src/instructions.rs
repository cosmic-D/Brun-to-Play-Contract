use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};

use crate::errors::GameError;
use crate::state::*;
use crate::accounts::*;

// Game Management Instructions

pub fn initialize_game(ctx: Context<InitializeGame>, weekly_quota: u32) -> Result<()> {
    let game_state = &mut ctx.accounts.game_state;
    let clock = Clock::get()?;

    game_state.bump = ctx.bumps.game_state;
    game_state.authority = ctx.accounts.authority.key();
    // Initialize with default backup team (can be updated later)
    game_state.backup_team = [Pubkey::default(); 3];
    game_state.wzn_mint = ctx.accounts.wzn_mint.key();
    game_state.weekly_quota = weekly_quota;
    game_state.total_burned = 0;
    game_state.last_weekly_reset = clock.unix_timestamp;
    game_state.is_initialized = true;
    game_state.emergency_mode = false;
    game_state.emergency_threshold = 1_000_000_000; // 1000 WZN
    game_state.last_dao_activity = clock.unix_timestamp;

    Ok(())
}

pub fn burn_to_play(ctx: Context<BurnToPlay>, amount: u64) -> Result<()> {
    require!(amount > 0, GameError::InvalidAmount);
    require!(ctx.accounts.game_state.is_initialized, GameError::GameNotInitialized);

    // Burn tokens from player's account
    let burn_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Burn {
            mint: ctx.accounts.wzn_mint.to_account_info(),
            from: ctx.accounts.player_token_account.to_account_info(),
            authority: ctx.accounts.player.to_account_info(),
        },
    );
    token::burn(burn_ctx, amount)?;

    // Update game state
    ctx.accounts.game_state.total_burned += amount;

    // Update player state
    let player_state = &mut ctx.accounts.player_state;
    if player_state.player == Pubkey::default() {
        player_state.player = ctx.accounts.player.key();
        player_state.bump = ctx.bumps.player_state;
    }
    player_state.weekly_plays_used += 1;

    Ok(())
}

pub fn use_quota_play(ctx: Context<UseQuotaPlay>) -> Result<()> {
    require!(ctx.accounts.game_state.is_initialized, GameError::GameNotInitialized);

    let player_quota = &mut ctx.accounts.player_quota;
    let clock = Clock::get()?;

    // Initialize player_quota if needed
    if player_quota.player == Pubkey::default() {
        player_quota.player = ctx.accounts.player.key();
        player_quota.bump = ctx.bumps.player_quota;
        player_quota.weekly_quota = ctx.accounts.game_state.weekly_quota;
        player_quota.plays_used_this_week = 0;
        player_quota.last_reset_time = clock.unix_timestamp;
        player_quota.total_plays_used = 0;
    }

    // Check if weekly reset is needed
    if is_weekly_reset_needed(player_quota.last_reset_time) {
        player_quota.plays_used_this_week = 0;
        player_quota.last_reset_time = clock.unix_timestamp;
    }

    // Check quota
    require!(
        player_quota.plays_used_this_week < player_quota.weekly_quota,
        GameError::QuotaExceeded
    );

    // Use quota
    player_quota.plays_used_this_week += 1;
    player_quota.total_plays_used += 1;

    Ok(())
}

// Staking Management Instructions

pub fn initialize_staking_pool(ctx: Context<InitializeStakingPool>) -> Result<()> {
    let staking_pool = &mut ctx.accounts.staking_pool;
    let clock = Clock::get()?;

    staking_pool.bump = ctx.bumps.staking_pool;
    staking_pool.total_staked = 0;
    staking_pool.total_rewards_distributed = 0;
    staking_pool.last_distribution = clock.unix_timestamp;
    staking_pool.apy_rate = ANNUAL_REWARD_RATE;
    staking_pool.minimum_stake = MINIMUM_STAKE_AMOUNT;
    staking_pool.is_initialized = true;

    Ok(())
}

pub fn stake_tokens(ctx: Context<StakeTokens>, amount: u64) -> Result<()> {
    require!(ctx.accounts.staking_pool.is_initialized, GameError::StakingPoolNotInitialized);
    require!(amount >= MINIMUM_STAKE_AMOUNT, GameError::InvalidAmount);

    // Transfer tokens to staking vault
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.player_token_account.to_account_info(),
            to: ctx.accounts.staking_vault.to_account_info(),
            authority: ctx.accounts.player.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, amount)?;

    // Update staking pool
    ctx.accounts.staking_pool.total_staked += amount;

    // Update player state
    let player_state = &mut ctx.accounts.player_state;
    if player_state.player == Pubkey::default() {
        player_state.player = ctx.accounts.player.key();
        player_state.bump = ctx.bumps.player_state;
    }
    player_state.total_staked += amount;

    Ok(())
}

pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
    require!(ctx.accounts.staking_pool.is_initialized, GameError::StakingPoolNotInitialized);
    require!(!ctx.accounts.player_state.is_slashed, GameError::OperationNotAllowed);

    let player_state = &mut ctx.accounts.player_state;
    let clock = Clock::get()?;

    // Check if enough time has passed since last claim
    require!(
        clock.unix_timestamp - player_state.last_claim_time >= WEEKLY_SECONDS,
        GameError::TooEarlyToClaim
    );

    // Calculate rewards
    let reward_amount = calculate_weekly_reward(player_state.total_staked);
    require!(reward_amount > 0, GameError::NoRewards);

    // Transfer rewards to player's quota account
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.reward_vault.to_account_info(),
            to: ctx.accounts.player_quota_account.to_account_info(),
            authority: ctx.accounts.staking_authority.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, reward_amount)?;

    // Update state
    player_state.total_rewards_claimed += reward_amount;
    player_state.last_claim_time = clock.unix_timestamp;
    ctx.accounts.staking_pool.total_rewards_distributed += reward_amount;

    Ok(())
}

pub fn withdraw_stake(ctx: Context<WithdrawStake>, amount: u64) -> Result<()> {
    require!(ctx.accounts.staking_pool.is_initialized, GameError::StakingPoolNotInitialized);
    require!(amount > 0, GameError::InvalidAmount);
    require!(amount <= ctx.accounts.player_state.total_staked, GameError::InsufficientStake);

    // Check if player is authorized to withdraw (DAO approval required)
    require!(
        ctx.accounts.dao_authority.key() == ctx.accounts.game_state.authority,
        GameError::DAONotAuthorized
    );

    // Transfer tokens back to player
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.staking_vault.to_account_info(),
            to: ctx.accounts.player_token_account.to_account_info(),
            authority: ctx.accounts.staking_authority.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, amount)?;

    // Update state
    ctx.accounts.staking_pool.total_staked -= amount;
    ctx.accounts.player_state.total_staked -= amount;

    Ok(())
}

// Vault Management Instructions

pub fn initialize_vault(ctx: Context<InitializeVault>, backup_threshold: u64) -> Result<()> {
    let recovery_vault = &mut ctx.accounts.recovery_vault;
    let clock = Clock::get()?;

    recovery_vault.bump = ctx.bumps.recovery_vault;
    recovery_vault.total_deposited = 0;
    recovery_vault.total_withdrawn = 0;
    recovery_vault.last_dao_withdrawal = clock.unix_timestamp;
    recovery_vault.backup_threshold = backup_threshold;
    recovery_vault.emergency_mode = false;
    recovery_vault.is_initialized = true;

    Ok(())
}

pub fn deposit_to_vault(ctx: Context<DepositToVault>, amount: u64) -> Result<()> {
    require!(ctx.accounts.recovery_vault.is_initialized, GameError::VaultNotInitialized);
    require!(amount > 0, GameError::InvalidAmount);

    // Transfer tokens to recovery vault
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.from_account.to_account_info(),
            to: ctx.accounts.vault_account.to_account_info(),
            authority: ctx.accounts.from.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, amount)?;

    // Update vault state
    ctx.accounts.recovery_vault.total_deposited += amount;

    Ok(())
}

pub fn dao_withdraw(ctx: Context<DAOWithdraw>, amount: u64) -> Result<()> {
    require!(ctx.accounts.recovery_vault.is_initialized, GameError::VaultNotInitialized);
    require!(amount > 0, GameError::InvalidAmount);
    require!(
        amount <= ctx.accounts.recovery_vault.total_deposited - ctx.accounts.recovery_vault.total_withdrawn,
        GameError::Overdraw
    );

    // Check DAO authorization
    require!(
        ctx.accounts.dao_authority.key() == ctx.accounts.game_state.authority,
        GameError::DAONotAuthorized
    );

    // Transfer tokens from vault
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.vault_account.to_account_info(),
            to: ctx.accounts.recipient.to_account_info(),
            authority: ctx.accounts.vault_authority.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, amount)?;

    // Update state
    ctx.accounts.recovery_vault.total_withdrawn += amount;
    ctx.accounts.recovery_vault.last_dao_withdrawal = Clock::get()?.unix_timestamp;
    ctx.accounts.game_state.last_dao_activity = Clock::get()?.unix_timestamp;

    Ok(())
}

pub fn backup_withdraw(ctx: Context<BackupWithdraw>, amount: u64) -> Result<()> {
    require!(ctx.accounts.recovery_vault.is_initialized, GameError::VaultNotInitialized);
    require!(amount > 0, GameError::InvalidAmount);

    let recovery_vault = &mut ctx.accounts.recovery_vault;
    let game_state = &mut ctx.accounts.game_state;
    let clock = Clock::get()?;

    // Check emergency conditions
    let dao_inactive = clock.unix_timestamp - game_state.last_dao_activity > WEEKLY_SECONDS * 26; // 6 months
    let low_balance = recovery_vault.total_deposited - recovery_vault.total_withdrawn < recovery_vault.backup_threshold;

    require!(dao_inactive || low_balance, GameError::NotEmergencyState);

    // Check backup team authorization
    let is_backup_member = game_state.backup_team.contains(&ctx.accounts.backup_authority.key());
    require!(is_backup_member, GameError::BackupNotAuthorized);

    // Limit withdrawal amount
    let max_withdrawal = (recovery_vault.total_deposited - recovery_vault.total_withdrawn) / 4;
    require!(amount <= max_withdrawal, GameError::Overdraw);

    // Transfer tokens
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.vault_account.to_account_info(),
            to: ctx.accounts.recipient.to_account_info(),
            authority: ctx.accounts.vault_authority.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, amount)?;

    // Update state
    recovery_vault.total_withdrawn += amount;
    recovery_vault.emergency_mode = true;

    Ok(())
}

// Governance Instructions

pub fn update_governance(ctx: Context<UpdateGovernance>, new_dao: Pubkey) -> Result<()> {
    require!(ctx.accounts.game_state.is_initialized, GameError::GameNotInitialized);
    require!(new_dao != Pubkey::default(), GameError::InvalidGovernanceUpdate);

    // Check current DAO authorization
    require!(
        ctx.accounts.dao_authority.key() == ctx.accounts.game_state.authority,
        GameError::DAONotAuthorized
    );

    // Update governance
    ctx.accounts.game_state.authority = new_dao;
    ctx.accounts.game_state.last_dao_activity = Clock::get()?.unix_timestamp;

    Ok(())
}

pub fn update_backup_team(ctx: Context<UpdateBackupTeam>, new_team: [Pubkey; 3]) -> Result<()> {
    require!(ctx.accounts.game_state.is_initialized, GameError::GameNotInitialized);

    // Validate backup team
    for member in &new_team {
        require!(*member != Pubkey::default(), GameError::InvalidBackupTeam);
    }

    // Check DAO authorization
    require!(
        ctx.accounts.dao_authority.key() == ctx.accounts.game_state.authority,
        GameError::DAONotAuthorized
    );

    // Update backup team
    ctx.accounts.game_state.backup_team = new_team;
    ctx.accounts.game_state.last_dao_activity = Clock::get()?.unix_timestamp;

    Ok(())
}

pub fn slash_stake(ctx: Context<SlashStake>, amount: u64) -> Result<()> {
    require!(ctx.accounts.staking_pool.is_initialized, GameError::StakingPoolNotInitialized);
    require!(amount > 0, GameError::InvalidAmount);

    let player_state = &mut ctx.accounts.player_state;
    let max_slash = (player_state.total_staked * MAX_SLASH_PERCENTAGE) / 100;
    require!(amount <= max_slash, GameError::SlashingAmountTooHigh);

    // Check DAO authorization
    require!(
        ctx.accounts.dao_authority.key() == ctx.accounts.game_state.authority,
        GameError::DAONotAuthorized
    );

    // Transfer slashed tokens to game vault
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.staking_vault.to_account_info(),
            to: ctx.accounts.game_vault.to_account_info(),
            authority: ctx.accounts.staking_authority.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, amount)?;

    // Update state
    player_state.total_staked -= amount;
    player_state.slash_amount += amount;
    player_state.is_slashed = true;
    ctx.accounts.staking_pool.total_staked -= amount;

    Ok(())
} 