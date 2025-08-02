use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};

use crate::errors::GameError;
use crate::state::*;
use crate::accounts::*;

// Game Management Instructions
pub fn initialize_game(ctx: Context<InitializeGame>, monthly_pass_cost: u64) -> Result<()> {
    let game_state = &mut ctx.accounts.game_state;
    let clock = Clock::get()?;

    game_state.bump = ctx.bumps.game_state;
    game_state.authority = ctx.accounts.authority.key();
    game_state.wzn_mint = ctx.accounts.wzn_mint.key();
    game_state.monthly_pass_cost = monthly_pass_cost;
    game_state.is_initialized = true;
    game_state.total_burned = 0;
    game_state.total_prizes_distributed = 0;
    game_state.last_monthly_reset = clock.unix_timestamp;
    game_state.emergency_mode = false;

    msg!("Game initialized with monthly pass cost: {}", monthly_pass_cost);
    Ok(())
}

pub fn burn_to_play(ctx: Context<BurnToPlay>, amount: u64) -> Result<()> {
    let game_state = &mut ctx.accounts.game_state;
    let burn_vault = &mut ctx.accounts.burn_vault;
    let player_pass = &mut ctx.accounts.player_pass;
    let clock = Clock::get()?;

    // Validate burn amount
    require!(amount >= game_state.monthly_pass_cost, GameError::InvalidAmount);
    require!(amount >= MINIMUM_BURN_AMOUNT, GameError::InvalidAmount);

    // Check if player has sufficient tokens
    require!(
        ctx.accounts.player_token_account.amount >= amount,
        GameError::InsufficientTokens
    );

    // Initialize player pass if needed
    if player_pass.player == Pubkey::default() {
        player_pass.player = ctx.accounts.player.key();
        player_pass.bump = ctx.bumps.player_pass;
        player_pass.total_passes_purchased = 0;
        player_pass.total_tokens_burned = 0;
    }

    // Update player pass
    player_pass.pass_start_time = clock.unix_timestamp;
    player_pass.pass_end_time = clock.unix_timestamp + MONTHLY_SECONDS;
    player_pass.is_active = true;
    player_pass.total_passes_purchased += 1;
    player_pass.total_tokens_burned += amount;

    // Update game state
    game_state.total_burned += amount;

    // Update burn vault
    burn_vault.total_locked += amount;

    // Transfer tokens to burn vault
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.player_token_account.to_account_info(),
            to: ctx.accounts.burn_vault_token_account.to_account_info(),
            authority: ctx.accounts.player.to_account_info(),
        },
    );

    token::transfer(transfer_ctx, amount)?;

    msg!("Player burned {} WZN tokens for monthly pass", amount);
    msg!("Pass valid until: {}", player_pass.pass_end_time);
    Ok(())
}

pub fn check_game_access(ctx: Context<CheckGameAccess>) -> Result<()> {
    let player_pass = &ctx.accounts.player_pass;
    let clock = Clock::get()?;

    // Check if pass is active
    require!(player_pass.is_active, GameError::PassNotActive);
    require!(clock.unix_timestamp <= player_pass.pass_end_time, GameError::PassExpired);

    msg!("Game access granted for player: {}", ctx.accounts.player.key());
    Ok(())
}

// Vault Management Instructions
pub fn initialize_burn_vault(ctx: Context<InitializeBurnVault>, emergency_threshold: u64, minimum_balance: u64) -> Result<()> {
    let burn_vault = &mut ctx.accounts.burn_vault;
    let clock = Clock::get()?;

    burn_vault.bump = ctx.bumps.burn_vault;
    burn_vault.total_locked = 0;
    burn_vault.total_unlocked = 0;
    burn_vault.last_dao_unlock = clock.unix_timestamp;
    burn_vault.emergency_unlock_threshold = emergency_threshold;
    burn_vault.minimum_balance_threshold = minimum_balance;
    burn_vault.is_initialized = true;
    burn_vault.unlock_delay = EMERGENCY_UNLOCK_DELAY;

    msg!("Burn vault initialized");
    msg!("Emergency threshold: {}", emergency_threshold);
    msg!("Minimum balance: {}", minimum_balance);
    Ok(())
}

pub fn initialize_prize_vault(ctx: Context<InitializePrizeVault>) -> Result<()> {
    let prize_vault = &mut ctx.accounts.prize_vault;

    prize_vault.bump = ctx.bumps.prize_vault;
    prize_vault.total_deposited = 0;
    prize_vault.total_distributed = 0;
    prize_vault.last_distribution = 0;
    prize_vault.is_initialized = true;

    msg!("Prize vault initialized");
    Ok(())
}

pub fn deposit_to_prize_vault(ctx: Context<DepositToPrizeVault>, amount: u64) -> Result<()> {
    let prize_vault = &mut ctx.accounts.prize_vault;

    // Transfer tokens to prize vault
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.from_token_account.to_account_info(),
            to: ctx.accounts.prize_vault_token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        },
    );

    token::transfer(transfer_ctx, amount)?;

    // Update prize vault state
    prize_vault.total_deposited += amount;

    msg!("Deposited {} WZN to prize vault", amount);
    Ok(())
}

// DAO Governance Instructions
pub fn initialize_dao(ctx: Context<InitializeDAO>, members: Vec<Pubkey>) -> Result<()> {
    let dao_governance = &mut ctx.accounts.dao_governance;
    let clock = Clock::get()?;

    dao_governance.bump = ctx.bumps.dao_governance;
    dao_governance.dao_members = members;
    dao_governance.total_members = dao_governance.dao_members.len() as u32;
    dao_governance.quorum_percentage = DAO_QUORUM_PERCENTAGE;
    dao_governance.last_activity = clock.unix_timestamp;
    dao_governance.is_initialized = true;
    dao_governance.pending_proposals = Vec::new();

    msg!("DAO governance initialized with {} members", dao_governance.total_members);
    Ok(())
}

pub fn create_proposal(
    ctx: Context<CreateProposal>,
    proposal_type: ProposalType,
    amount: u64,
    description: String,
) -> Result<()> {
    let dao_governance = &mut ctx.accounts.dao_governance;
    let clock = Clock::get()?;

    // Check if proposer is a DAO member
    require!(
        dao_governance.dao_members.contains(&ctx.accounts.proposer.key()),
        GameError::DAOMemberNotFound
    );

    let proposal = Proposal {
        id: dao_governance.pending_proposals.len() as u64,
        proposer: ctx.accounts.proposer.key(),
        proposal_type,
        amount,
        description,
        votes_for: 0,
        votes_against: 0,
        total_votes: 0,
        is_executed: false,
        created_at: clock.unix_timestamp,
        executed_at: None,
    };

    dao_governance.pending_proposals.push(proposal);
    dao_governance.last_activity = clock.unix_timestamp;

    msg!("Proposal created with ID: {}", proposal.id);
    Ok(())
}

pub fn vote_on_proposal(ctx: Context<VoteOnProposal>, proposal_id: u64, vote_for: bool) -> Result<()> {
    let dao_governance = &mut ctx.accounts.dao_governance;
    let clock = Clock::get()?;

    // Check if voter is a DAO member
    require!(
        dao_governance.dao_members.contains(&ctx.accounts.voter.key()),
        GameError::DAOMemberNotFound
    );

    // Find and update proposal
    let proposal = dao_governance
        .pending_proposals
        .iter_mut()
        .find(|p| p.id == proposal_id)
        .ok_or(GameError::ProposalNotFound)?;

    if vote_for {
        proposal.votes_for += 1;
    } else {
        proposal.votes_against += 1;
    }
    proposal.total_votes += 1;

    dao_governance.last_activity = clock.unix_timestamp;

    msg!("Vote recorded for proposal {}: {}", proposal_id, if vote_for { "FOR" } else { "AGAINST" });
    Ok(())
}

pub fn execute_proposal(ctx: Context<ExecuteProposal>, proposal_id: u64) -> Result<()> {
    let dao_governance = &mut ctx.accounts.dao_governance;
    let burn_vault = &mut ctx.accounts.burn_vault;
    let prize_vault = &mut ctx.accounts.prize_vault;
    let game_state = &mut ctx.accounts.game_state;
    let clock = Clock::get()?;

    // Check if executor is a DAO member
    require!(
        dao_governance.dao_members.contains(&ctx.accounts.executor.key()),
        GameError::DAOMemberNotFound
    );

    // Find proposal
    let proposal = dao_governance
        .pending_proposals
        .iter_mut()
        .find(|p| p.id == proposal_id)
        .ok_or(GameError::ProposalNotFound)?;

    require!(!proposal.is_executed, GameError::ProposalAlreadyExecuted);

    // Check quorum
    let required_votes = (dao_governance.total_members * dao_governance.quorum_percentage as u32) / 100;
    require!(proposal.total_votes >= required_votes, GameError::InsufficientVotes);

    // Execute proposal based on type
    match proposal.proposal_type {
        ProposalType::UnlockBurnVault => {
            require!(proposal.amount <= burn_vault.total_locked, GameError::InvalidAmount);
            burn_vault.total_locked -= proposal.amount;
            burn_vault.total_unlocked += proposal.amount;
            burn_vault.last_dao_unlock = clock.unix_timestamp;
            msg!("Unlocked {} WZN from burn vault", proposal.amount);
        }
        ProposalType::DistributePrizes => {
            require!(proposal.amount <= prize_vault.total_deposited, GameError::InvalidAmount);
            prize_vault.total_distributed += proposal.amount;
            game_state.total_prizes_distributed += proposal.amount;
            msg!("Distributed {} WZN in prizes", proposal.amount);
        }
        ProposalType::UpdateMonthlyPassCost => {
            game_state.monthly_pass_cost = proposal.amount;
            msg!("Updated monthly pass cost to {}", proposal.amount);
        }
        _ => {
            return err!(GameError::InvalidProposalType);
        }
    }

    proposal.is_executed = true;
    proposal.executed_at = Some(clock.unix_timestamp);
    dao_governance.last_activity = clock.unix_timestamp;

    msg!("Proposal {} executed successfully", proposal_id);
    Ok(())
}

// Emergency Recovery Instructions
pub fn initialize_emergency_recovery(ctx: Context<InitializeEmergencyRecovery>, members: Vec<Pubkey>) -> Result<()> {
    let emergency_recovery = &mut ctx.accounts.emergency_recovery;
    let clock = Clock::get()?;

    emergency_recovery.bump = ctx.bumps.emergency_recovery;
    emergency_recovery.backup_members = members;
    emergency_recovery.total_members = emergency_recovery.backup_members.len() as u32;
    emergency_recovery.quorum_percentage = EMERGENCY_QUORUM_PERCENTAGE;
    emergency_recovery.last_activity = clock.unix_timestamp;
    emergency_recovery.is_initialized = true;
    emergency_recovery.emergency_active = false;
    emergency_recovery.emergency_start_time = 0;

    msg!("Emergency recovery initialized with {} members", emergency_recovery.total_members);
    Ok(())
}

pub fn emergency_unlock(ctx: Context<EmergencyUnlock>, amount: u64, percentage: u64) -> Result<()> {
    let emergency_recovery = &mut ctx.accounts.emergency_recovery;
    let burn_vault = &mut ctx.accounts.burn_vault;
    let clock = Clock::get()?;

    // Check if unlocker is an emergency member
    require!(
        emergency_recovery.backup_members.contains(&ctx.accounts.recipient.key()),
        GameError::EmergencyMemberNotFound
    );

    // Check emergency conditions
    require!(can_emergency_unlock(burn_vault), GameError::EmergencyConditionsNotMet);
    require!(percentage <= MAX_EMERGENCY_UNLOCK_PERCENTAGE, GameError::InvalidUnlockPercentage);

    // Calculate unlock amount
    let calculated_amount = calculate_emergency_unlock_amount(burn_vault.total_locked, percentage);
    require!(amount <= calculated_amount, GameError::InvalidEmergencyUnlockAmount);

    // Update emergency recovery state
    emergency_recovery.emergency_active = true;
    emergency_recovery.emergency_start_time = clock.unix_timestamp;
    emergency_recovery.last_activity = clock.unix_timestamp;

    // Update burn vault
    burn_vault.total_locked -= amount;
    burn_vault.total_unlocked += amount;

    // Transfer tokens to recipient
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.burn_vault_token_account.to_account_info(),
            to: ctx.accounts.recipient_token_account.to_account_info(),
            authority: ctx.accounts.recipient.to_account_info(),
        },
    );

    token::transfer(transfer_ctx, amount)?;

    msg!("Emergency unlock executed: {} WZN ({}%)", amount, percentage);
    Ok(())
}

// Player Score Management Instructions
pub fn update_player_score(
    ctx: Context<UpdatePlayerScore>,
    games_played: u32,
    games_won: u32,
    rating_change: i32,
) -> Result<()> {
    let player_score = &mut ctx.accounts.player_score;
    let clock = Clock::get()?;

    // Initialize player score if needed
    if player_score.player == Pubkey::default() {
        player_score.player = ctx.accounts.player.key();
        player_score.bump = ctx.bumps.player_score;
        player_score.total_games_played = 0;
        player_score.total_games_won = 0;
        player_score.current_rating = 1000; // Starting rating
        player_score.highest_rating = 1000;
        player_score.monthly_rank = 0;
        player_score.last_game_time = 0;
        player_score.total_prizes_earned = 0;
    }

    // Update score
    player_score.total_games_played += games_played;
    player_score.total_games_won += games_won;
    
    let new_rating = (player_score.current_rating as i32 + rating_change).max(0) as u32;
    player_score.current_rating = new_rating;
    
    if new_rating > player_score.highest_rating {
        player_score.highest_rating = new_rating;
    }
    
    player_score.last_game_time = clock.unix_timestamp;

    msg!("Player score updated: {} games played, {} won, rating: {}", 
         games_played, games_won, new_rating);
    Ok(())
}

pub fn distribute_prize(ctx: Context<DistributePrize>, amount: u64) -> Result<()> {
    let prize_vault = &mut ctx.accounts.prize_vault;
    let player_score = &mut ctx.accounts.player_score;
    let game_state = &mut ctx.accounts.game_state;

    // Transfer tokens from prize vault to recipient
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.prize_vault_token_account.to_account_info(),
            to: ctx.accounts.recipient_token_account.to_account_info(),
            authority: ctx.accounts.recipient.to_account_info(),
        },
    );

    token::transfer(transfer_ctx, amount)?;

    // Update state
    prize_vault.total_distributed += amount;
    player_score.total_prizes_earned += amount;
    game_state.total_prizes_distributed += amount;

    msg!("Prize distributed: {} WZN to {}", amount, ctx.accounts.recipient.key());
    Ok(())
}

// Monthly Reset Instruction
pub fn monthly_reset(ctx: Context<MonthlyReset>) -> Result<()> {
    let game_state = &mut ctx.accounts.game_state;
    let clock = Clock::get()?;

    // Check if monthly reset is needed
    require!(
        is_monthly_reset_needed(game_state.last_monthly_reset),
        GameError::MonthlyResetNotReady
    );

    game_state.last_monthly_reset = clock.unix_timestamp;

    msg!("Monthly reset completed at timestamp: {}", clock.unix_timestamp);
    Ok(())
} 