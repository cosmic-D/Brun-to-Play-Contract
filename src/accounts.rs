use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Burn, Mint};

use crate::state::*;
use crate::errors::GameError;

// Game Management
#[derive(Accounts)]
pub struct InitializeGame<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 1 + 32 + 32 + 8 + 1 + 8 + 8 + 8 + 1,
        seeds = [GAME_STATE_SEED],
        bump
    )]
    pub game_state: Account<'info, GameState>,
    
    pub authority: Signer<'info>,
    pub wzn_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct BurnToPlay<'info> {
    #[account(
        mut,
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump,
        constraint = game_state.is_initialized @ GameError::GameNotInitialized
    )]
    pub game_state: Account<'info, GameState>,
    
    #[account(
        mut,
        seeds = [BURN_VAULT_SEED],
        bump = burn_vault.bump,
        constraint = burn_vault.is_initialized @ GameError::BurnVaultNotInitialized
    )]
    pub burn_vault: Account<'info, BurnVault>,
    
    #[account(
        mut,
        init_if_needed,
        payer = player,
        space = 8 + 1 + 32 + 8 + 8 + 1 + 4 + 8,
        seeds = [PLAYER_PASS_SEED, player.key().as_ref()],
        bump
    )]
    pub player_pass: Account<'info, PlayerPass>,
    
    #[account(
        mut,
        constraint = player_token_account.owner == player.key() @ GameError::InvalidTokenAccount,
        constraint = player_token_account.mint == game_state.wzn_mint @ GameError::InvalidTokenMint
    )]
    pub player_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [BURN_VAULT_SEED],
        bump = burn_vault.bump
    )]
    /// CHECK: This is the burn vault token account
    pub burn_vault_token_account: UncheckedAccount<'info>,
    
    pub player: Signer<'info>,
    pub wzn_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CheckGameAccess<'info> {
    #[account(
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump,
        constraint = game_state.is_initialized @ GameError::GameNotInitialized
    )]
    pub game_state: Account<'info, GameState>,
    
    #[account(
        seeds = [PLAYER_PASS_SEED, player.key().as_ref()],
        bump = player_pass.bump,
        constraint = player_pass.player == player.key() @ GameError::NotAuthorized
    )]
    pub player_pass: Account<'info, PlayerPass>,
    
    pub player: Signer<'info>,
}

// Vault Management
#[derive(Accounts)]
pub struct InitializeBurnVault<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 1 + 8 + 8 + 8 + 8 + 8 + 1 + 8,
        seeds = [BURN_VAULT_SEED],
        bump
    )]
    pub burn_vault: Account<'info, BurnVault>,
    
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitializePrizeVault<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 1 + 8 + 8 + 8 + 1,
        seeds = [PRIZE_VAULT_SEED],
        bump
    )]
    pub prize_vault: Account<'info, PrizeVault>,
    
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct DepositToPrizeVault<'info> {
    #[account(
        mut,
        seeds = [PRIZE_VAULT_SEED],
        bump = prize_vault.bump,
        constraint = prize_vault.is_initialized @ GameError::PrizeVaultNotInitialized
    )]
    pub prize_vault: Account<'info, PrizeVault>,
    
    #[account(
        mut,
        constraint = from_token_account.owner == authority.key() @ GameError::InvalidTokenAccount,
        constraint = from_token_account.mint == game_state.wzn_mint @ GameError::InvalidTokenMint
    )]
    pub from_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [PRIZE_VAULT_SEED],
        bump = prize_vault.bump
    )]
    /// CHECK: This is the prize vault token account
    pub prize_vault_token_account: UncheckedAccount<'info>,
    
    #[account(
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump,
        constraint = game_state.is_initialized @ GameError::GameNotInitialized
    )]
    pub game_state: Account<'info, GameState>,
    
    pub authority: Signer<'info>,
    pub wzn_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

// DAO Governance
#[derive(Accounts)]
pub struct InitializeDAO<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 1 + 4 + 32 * 50 + 4 + 8 + 1 + 4 + 100 * 50, // Space for 50 members and 50 proposals
        seeds = [DAO_GOVERNANCE_SEED],
        bump
    )]
    pub dao_governance: Account<'info, DAOGovernance>,
    
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    #[account(
        mut,
        seeds = [DAO_GOVERNANCE_SEED],
        bump = dao_governance.bump,
        constraint = dao_governance.is_initialized @ GameError::DAONotInitialized
    )]
    pub dao_governance: Account<'info, DAOGovernance>,
    
    pub proposer: Signer<'info>,
}

#[derive(Accounts)]
pub struct VoteOnProposal<'info> {
    #[account(
        mut,
        seeds = [DAO_GOVERNANCE_SEED],
        bump = dao_governance.bump,
        constraint = dao_governance.is_initialized @ GameError::DAONotInitialized
    )]
    pub dao_governance: Account<'info, DAOGovernance>,
    
    pub voter: Signer<'info>,
}

#[derive(Accounts)]
pub struct ExecuteProposal<'info> {
    #[account(
        mut,
        seeds = [DAO_GOVERNANCE_SEED],
        bump = dao_governance.bump,
        constraint = dao_governance.is_initialized @ GameError::DAONotInitialized
    )]
    pub dao_governance: Account<'info, DAOGovernance>,
    
    #[account(
        mut,
        seeds = [BURN_VAULT_SEED],
        bump = burn_vault.bump,
        constraint = burn_vault.is_initialized @ GameError::BurnVaultNotInitialized
    )]
    pub burn_vault: Account<'info, BurnVault>,
    
    #[account(
        mut,
        seeds = [PRIZE_VAULT_SEED],
        bump = prize_vault.bump,
        constraint = prize_vault.is_initialized @ GameError::PrizeVaultNotInitialized
    )]
    pub prize_vault: Account<'info, PrizeVault>,
    
    #[account(
        mut,
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump,
        constraint = game_state.is_initialized @ GameError::GameNotInitialized
    )]
    pub game_state: Account<'info, GameState>,
    
    pub executor: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

// Emergency Recovery
#[derive(Accounts)]
pub struct InitializeEmergencyRecovery<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 1 + 4 + 32 * 20 + 4 + 8 + 8 + 1 + 1 + 8, // Space for 20 backup members
        seeds = [EMERGENCY_RECOVERY_SEED],
        bump
    )]
    pub emergency_recovery: Account<'info, EmergencyRecovery>,
    
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct EmergencyUnlock<'info> {
    #[account(
        mut,
        seeds = [EMERGENCY_RECOVERY_SEED],
        bump = emergency_recovery.bump,
        constraint = emergency_recovery.is_initialized @ GameError::EmergencyRecoveryNotInitialized
    )]
    pub emergency_recovery: Account<'info, EmergencyRecovery>,
    
    #[account(
        mut,
        seeds = [BURN_VAULT_SEED],
        bump = burn_vault.bump,
        constraint = burn_vault.is_initialized @ GameError::BurnVaultNotInitialized
    )]
    pub burn_vault: Account<'info, BurnVault>,
    
    #[account(
        mut,
        seeds = [BURN_VAULT_SEED],
        bump = burn_vault.bump
    )]
    /// CHECK: This is the burn vault token account
    pub burn_vault_token_account: UncheckedAccount<'info>,
    
    #[account(
        mut,
        constraint = recipient_token_account.owner == recipient.key() @ GameError::InvalidTokenAccount,
        constraint = recipient_token_account.mint == game_state.wzn_mint @ GameError::InvalidTokenMint
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,
    
    #[account(
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump,
        constraint = game_state.is_initialized @ GameError::GameNotInitialized
    )]
    pub game_state: Account<'info, GameState>,
    
    pub recipient: Signer<'info>,
    pub wzn_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

// Player Score Management
#[derive(Accounts)]
pub struct UpdatePlayerScore<'info> {
    #[account(
        mut,
        init_if_needed,
        payer = player,
        space = 8 + 1 + 32 + 4 + 4 + 4 + 4 + 4 + 8 + 8,
        seeds = [PLAYER_SCORE_SEED, player.key().as_ref()],
        bump
    )]
    pub player_score: Account<'info, PlayerScore>,
    
    #[account(
        seeds = [PLAYER_PASS_SEED, player.key().as_ref()],
        bump = player_pass.bump,
        constraint = player_pass.player == player.key() @ GameError::NotAuthorized
    )]
    pub player_pass: Account<'info, PlayerPass>,
    
    pub player: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct DistributePrize<'info> {
    #[account(
        mut,
        seeds = [PRIZE_VAULT_SEED],
        bump = prize_vault.bump,
        constraint = prize_vault.is_initialized @ GameError::PrizeVaultNotInitialized
    )]
    pub prize_vault: Account<'info, PrizeVault>,
    
    #[account(
        mut,
        seeds = [PRIZE_VAULT_SEED],
        bump = prize_vault.bump
    )]
    /// CHECK: This is the prize vault token account
    pub prize_vault_token_account: UncheckedAccount<'info>,
    
    #[account(
        mut,
        constraint = recipient_token_account.owner == recipient.key() @ GameError::InvalidTokenAccount,
        constraint = recipient_token_account.mint == game_state.wzn_mint @ GameError::InvalidTokenMint
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [PLAYER_SCORE_SEED, recipient.key().as_ref()],
        bump = player_score.bump,
        constraint = player_score.player == recipient.key() @ GameError::NotAuthorized
    )]
    pub player_score: Account<'info, PlayerScore>,
    
    #[account(
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump,
        constraint = game_state.is_initialized @ GameError::GameNotInitialized
    )]
    pub game_state: Account<'info, GameState>,
    
    pub recipient: Signer<'info>,
    pub wzn_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

// Monthly Reset
#[derive(Accounts)]
pub struct MonthlyReset<'info> {
    #[account(
        mut,
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump,
        constraint = game_state.is_initialized @ GameError::GameNotInitialized
    )]
    pub game_state: Account<'info, GameState>,
    
    pub authority: Signer<'info>,
} 