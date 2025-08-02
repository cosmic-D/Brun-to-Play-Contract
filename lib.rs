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
    pub fn initialize_game(ctx: Context<InitializeGame>, monthly_pass_cost: u64) -> Result<()> {
        instructions::initialize_game(ctx, monthly_pass_cost)
    }

    pub fn burn_to_play(ctx: Context<BurnToPlay>, amount: u64) -> Result<()> {
        instructions::burn_to_play(ctx, amount)
    }

    pub fn check_game_access(ctx: Context<CheckGameAccess>) -> Result<()> {
        instructions::check_game_access(ctx)
    }

    // Vault Management
    pub fn initialize_burn_vault(ctx: Context<InitializeBurnVault>, emergency_threshold: u64, minimum_balance: u64) -> Result<()> {
        instructions::initialize_burn_vault(ctx, emergency_threshold, minimum_balance)
    }

    pub fn initialize_prize_vault(ctx: Context<InitializePrizeVault>) -> Result<()> {
        instructions::initialize_prize_vault(ctx)
    }

    pub fn deposit_to_prize_vault(ctx: Context<DepositToPrizeVault>, amount: u64) -> Result<()> {
        instructions::deposit_to_prize_vault(ctx, amount)
    }

    // DAO Governance
    pub fn initialize_dao(ctx: Context<InitializeDAO>, members: Vec<Pubkey>) -> Result<()> {
        instructions::initialize_dao(ctx, members)
    }

    pub fn create_proposal(ctx: Context<CreateProposal>, proposal_type: ProposalType, amount: u64, description: String) -> Result<()> {
        instructions::create_proposal(ctx, proposal_type, amount, description)
    }

    pub fn vote_on_proposal(ctx: Context<VoteOnProposal>, proposal_id: u64, vote_for: bool) -> Result<()> {
        instructions::vote_on_proposal(ctx, proposal_id, vote_for)
    }

    pub fn execute_proposal(ctx: Context<ExecuteProposal>, proposal_id: u64) -> Result<()> {
        instructions::execute_proposal(ctx, proposal_id)
    }

    // Emergency Recovery
    pub fn initialize_emergency_recovery(ctx: Context<InitializeEmergencyRecovery>, members: Vec<Pubkey>) -> Result<()> {
        instructions::initialize_emergency_recovery(ctx, members)
    }

    pub fn emergency_unlock(ctx: Context<EmergencyUnlock>, amount: u64, percentage: u64) -> Result<()> {
        instructions::emergency_unlock(ctx, amount, percentage)
    }

    // Player Score Management
    pub fn update_player_score(ctx: Context<UpdatePlayerScore>, games_played: u32, games_won: u32, rating_change: i32) -> Result<()> {
        instructions::update_player_score(ctx, games_played, games_won, rating_change)
    }

    pub fn distribute_prize(ctx: Context<DistributePrize>, amount: u64) -> Result<()> {
        instructions::distribute_prize(ctx, amount)
    }

    // Monthly Reset
    pub fn monthly_reset(ctx: Context<MonthlyReset>) -> Result<()> {
        instructions::monthly_reset(ctx)
    }
} 