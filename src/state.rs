use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

use crate::errors::GameError;

// Constants
pub const MONTHLY_SECONDS: i64 = 30 * 24 * 60 * 60; // 30 days in seconds
pub const DAO_INACTIVITY_THRESHOLD: i64 = 6 * 30 * 24 * 60 * 60; // 6 months
pub const EMERGENCY_UNLOCK_DELAY: i64 = 7 * 24 * 60 * 60; // 7 days
pub const MINIMUM_BURN_AMOUNT: u64 = 1_000_000; // 1 WZN (assuming 6 decimals)
pub const MAX_EMERGENCY_UNLOCK_PERCENTAGE: u64 = 25; // 25% max emergency unlock
pub const DAO_QUORUM_PERCENTAGE: u64 = 60; // 60% quorum for DAO votes
pub const EMERGENCY_QUORUM_PERCENTAGE: u64 = 80; // 80% quorum for emergency votes

// Seeds for PDAs
pub const GAME_STATE_SEED: &[u8] = b"game_state";
pub const BURN_VAULT_SEED: &[u8] = b"burn_vault";
pub const PRIZE_VAULT_SEED: &[u8] = b"prize_vault";
pub const DAO_GOVERNANCE_SEED: &[u8] = b"dao_governance";
pub const EMERGENCY_RECOVERY_SEED: &[u8] = b"emergency_recovery";
pub const PLAYER_PASS_SEED: &[u8] = b"player_pass";
pub const PLAYER_SCORE_SEED: &[u8] = b"player_score";

#[account]
pub struct GameState {
    pub bump: u8,
    pub authority: Pubkey, // Initial authority (can be transferred to DAO)
    pub wzn_mint: Pubkey,
    pub monthly_pass_cost: u64, // Cost in WZN for 30-day pass
    pub is_initialized: bool,
    pub total_burned: u64,
    pub total_prizes_distributed: u64,
    pub last_monthly_reset: i64,
    pub emergency_mode: bool,
}

#[account]
pub struct BurnVault {
    pub bump: u8,
    pub total_locked: u64,
    pub total_unlocked: u64,
    pub last_dao_unlock: i64,
    pub emergency_unlock_threshold: u64, // 80% of supply
    pub minimum_balance_threshold: u64, // 10M WZN
    pub is_initialized: bool,
    pub unlock_delay: i64, // Delay before emergency unlock can execute
}

#[account]
pub struct PrizeVault {
    pub bump: u8,
    pub total_deposited: u64,
    pub total_distributed: u64,
    pub last_distribution: i64,
    pub is_initialized: bool,
}

#[account]
pub struct DAOGovernance {
    pub bump: u8,
    pub dao_members: Vec<Pubkey>, // DAO member addresses
    pub total_members: u32,
    pub quorum_percentage: u64,
    pub last_activity: i64,
    pub is_initialized: bool,
    pub pending_proposals: Vec<Proposal>,
}

#[account]
pub struct EmergencyRecovery {
    pub bump: u8,
    pub backup_members: Vec<Pubkey>, // 10+ trusted wallets
    pub total_members: u32,
    pub quorum_percentage: u64,
    pub last_activity: i64,
    pub is_initialized: bool,
    pub emergency_active: bool,
    pub emergency_start_time: i64,
}

#[account]
pub struct PlayerPass {
    pub bump: u8,
    pub player: Pubkey,
    pub pass_start_time: i64,
    pub pass_end_time: i64,
    pub is_active: bool,
    pub total_passes_purchased: u32,
    pub total_tokens_burned: u64,
}

#[account]
pub struct PlayerScore {
    pub bump: u8,
    pub player: Pubkey,
    pub total_games_played: u32,
    pub total_games_won: u32,
    pub current_rating: u32,
    pub highest_rating: u32,
    pub monthly_rank: u32,
    pub last_game_time: i64,
    pub total_prizes_earned: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Pubkey,
    pub proposal_type: ProposalType,
    pub amount: u64,
    pub description: String,
    pub votes_for: u32,
    pub votes_against: u32,
    pub total_votes: u32,
    pub is_executed: bool,
    pub created_at: i64,
    pub executed_at: Option<i64>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum ProposalType {
    UnlockBurnVault,
    DistributePrizes,
    UpdateMonthlyPassCost,
    EmergencyUnlock,
    UpdateDAO,
}

// Helper functions for PDA derivation
pub fn get_game_state_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[GAME_STATE_SEED], &crate::ID)
}

pub fn get_burn_vault_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[BURN_VAULT_SEED], &crate::ID)
}

pub fn get_prize_vault_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[PRIZE_VAULT_SEED], &crate::ID)
}

pub fn get_dao_governance_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[DAO_GOVERNANCE_SEED], &crate::ID)
}

pub fn get_emergency_recovery_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[EMERGENCY_RECOVERY_SEED], &crate::ID)
}

pub fn get_player_pass_pda(player: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[PLAYER_PASS_SEED, player.as_ref()], &crate::ID)
}

pub fn get_player_score_pda(player: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[PLAYER_SCORE_SEED, player.as_ref()], &crate::ID)
}

// Helper functions for validation
pub fn validate_token_account(
    account: &Account<TokenAccount>,
    expected_mint: &Pubkey,
    expected_owner: &Pubkey,
) -> Result<()> {
    require_keys_eq!(account.mint, *expected_mint, GameError::InvalidTokenMint);
    require_keys_eq!(account.owner, *expected_owner, GameError::InvalidTokenAccount);
    Ok(())
}

pub fn is_pass_active(pass: &PlayerPass) -> bool {
    let now = Clock::get().unwrap().unix_timestamp;
    pass.is_active && now >= pass.pass_start_time && now <= pass.pass_end_time
}

pub fn is_monthly_reset_needed(last_reset: i64) -> bool {
    let now = Clock::get().unwrap().unix_timestamp;
    now - last_reset >= MONTHLY_SECONDS
}

pub fn is_dao_inactive(last_activity: i64) -> bool {
    let now = Clock::get().unwrap().unix_timestamp;
    now - last_activity >= DAO_INACTIVITY_THRESHOLD
}

pub fn can_emergency_unlock(burn_vault: &BurnVault) -> bool {
    let now = Clock::get().unwrap().unix_timestamp;
    burn_vault.total_locked >= burn_vault.emergency_unlock_threshold &&
    burn_vault.total_locked <= burn_vault.minimum_balance_threshold &&
    now - burn_vault.last_dao_unlock >= EMERGENCY_UNLOCK_DELAY
}

pub fn calculate_emergency_unlock_amount(total_locked: u64, percentage: u64) -> u64 {
    (total_locked * percentage) / 100
} 