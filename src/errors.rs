use anchor_lang::prelude::*;

#[error_code]
pub enum GameError {
    #[msg("Invalid amount to burn")]
    InvalidAmount,
    
    #[msg("Invalid token mint")]
    InvalidTokenMint,
    
    #[msg("Invalid token account")]
    InvalidTokenAccount,
    
    #[msg("Player pass not active")]
    PassNotActive,
    
    #[msg("Pass expired")]
    PassExpired,
    
    #[msg("Insufficient tokens for burn")]
    InsufficientTokens,
    
    #[msg("Game not initialized")]
    GameNotInitialized,
    
    #[msg("Burn vault not initialized")]
    BurnVaultNotInitialized,
    
    #[msg("Prize vault not initialized")]
    PrizeVaultNotInitialized,
    
    #[msg("DAO governance not initialized")]
    DAONotInitialized,
    
    #[msg("Emergency recovery not initialized")]
    EmergencyRecoveryNotInitialized,
    
    #[msg("Not authorized")]
    NotAuthorized,
    
    #[msg("DAO member not found")]
    DAOMemberNotFound,
    
    #[msg("Emergency member not found")]
    EmergencyMemberNotFound,
    
    #[msg("Proposal not found")]
    ProposalNotFound,
    
    #[msg("Proposal already executed")]
    ProposalAlreadyExecuted,
    
    #[msg("Insufficient votes for quorum")]
    InsufficientVotes,
    
    #[msg("Emergency conditions not met")]
    EmergencyConditionsNotMet,
    
    #[msg("Emergency unlock delay not met")]
    EmergencyUnlockDelayNotMet,
    
    #[msg("Invalid unlock percentage")]
    InvalidUnlockPercentage,
    
    #[msg("DAO not inactive enough for emergency")]
    DAONotInactiveEnough,
    
    #[msg("Player score not found")]
    PlayerScoreNotFound,
    
    #[msg("Invalid game result")]
    InvalidGameResult,
    
    #[msg("Monthly reset not ready")]
    MonthlyResetNotReady,
    
    #[msg("Invalid proposal type")]
    InvalidProposalType,
    
    #[msg("Proposal voting period ended")]
    ProposalVotingEnded,
    
    #[msg("Invalid emergency unlock amount")]
    InvalidEmergencyUnlockAmount,
} 