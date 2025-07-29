use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

use crate::errors::GameError;

// Constants
pub const WEEKLY_SECONDS: i64 = 7 * 24 * 60 * 60; // 7 days in seconds
pub const ANNUAL_REWARD_RATE: u64 = 4; // 4% APY
pub const WEEKLY_REWARD_RATE: u64 = ANNUAL_REWARD_RATE * 100 / 52; // ~0.0769% weekly
pub const MINIMUM_STAKE_AMOUNT: u64 = 1_000_000; // 1 WZN (assuming 6 decimals)
pub const MAX_SLASH_PERCENTAGE: u64 = 50; // 50% max slashing

// Seeds for PDAs
pub const GAME_STATE_SEED: &[u8] = b"game_state";
pub const STAKING_POOL_SEED: &[u8] = b"staking_pool";
pub const RECOVERY_VAULT_SEED: &[u8] = b"recovery_vault";
pub const PLAYER_STATE_SEED: &[u8] = b"player_state";
pub const PLAYER_QUOTA_SEED: &[u8] = b"player_quota";
pub const STAKING_VAULT_SEED: &[u8] = b"staking_vault";
pub const REWARD_VAULT_SEED: &[u8] = b"reward_vault";
pub const GAME_VAULT_SEED: &[u8] = b"game_vault";

#[account]
pub struct GameState {
    pub bump: u8,
    pub authority: Pubkey, // DAO authority
    pub backup_team: [Pubkey; 3], // 3-member backup multisig
    pub wzn_mint: Pubkey,
    pub weekly_quota: u32,
    pub total_burned: u64,
    pub last_weekly_reset: i64,
    pub is_initialized: bool,
    pub emergency_mode: bool,
    pub emergency_threshold: u64,
    pub last_dao_activity: i64,
}

#[account]
pub struct StakingPool {
    pub bump: u8,
    pub total_staked: u64,
    pub total_rewards_distributed: u64,
    pub last_distribution: i64,
    pub apy_rate: u64,
    pub minimum_stake: u64,
    pub is_initialized: bool,
}

#[account]
pub struct RecoveryVault {
    pub bump: u8,
    pub total_deposited: u64,
    pub total_withdrawn: u64,
    pub last_dao_withdrawal: i64,
    pub backup_threshold: u64,
    pub emergency_mode: bool,
    pub is_initialized: bool,
}

#[account]
pub struct PlayerState {
    pub bump: u8,
    pub player: Pubkey,
    pub total_staked: u64,
    pub total_rewards_claimed: u64,
    pub last_claim_time: i64,
    pub weekly_plays_used: u32,
    pub last_weekly_reset: i64,
    pub is_slashed: bool,
    pub slash_amount: u64,
}

#[account]
pub struct PlayerQuota {
    pub bump: u8,
    pub player: Pubkey,
    pub weekly_quota: u32,
    pub plays_used_this_week: u32,
    pub last_reset_time: i64,
    pub total_plays_used: u64,
}

// Helper functions for PDA derivation
pub fn get_game_state_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[GAME_STATE_SEED], &crate::ID)
}

pub fn get_staking_pool_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[STAKING_POOL_SEED], &crate::ID)
}

pub fn get_recovery_vault_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[RECOVERY_VAULT_SEED], &crate::ID)
}

pub fn get_player_state_pda(player: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[PLAYER_STATE_SEED, player.as_ref()], &crate::ID)
}

pub fn get_player_quota_pda(player: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[PLAYER_QUOTA_SEED, player.as_ref()], &crate::ID)
}

pub fn get_staking_vault_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[STAKING_VAULT_SEED], &crate::ID)
}

pub fn get_reward_vault_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[REWARD_VAULT_SEED], &crate::ID)
}

pub fn get_game_vault_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[GAME_VAULT_SEED], &crate::ID)
}

// Helper functions for validation
pub fn validate_token_account(
    account: &Account<TokenAccount>,
    expected_mint: &Pubkey,
    expected_owner: &Pubkey,
) -> Result<()> {
    require_keys_eq!(account.mint, *expected_mint, crate::errors::GameError::InvalidTokenMint);
    require_keys_eq!(account.owner, *expected_owner, crate::errors::GameError::InvalidTokenAccount);
    Ok(())
}

pub fn is_weekly_reset_needed(last_reset: i64) -> bool {
    let now = Clock::get().unwrap().unix_timestamp;
    now - last_reset >= WEEKLY_SECONDS
}

pub fn calculate_weekly_reward(staked_amount: u64) -> u64 {
    (staked_amount * WEEKLY_REWARD_RATE) / 10000 // Divide by 10000 for percentage
} 