use anchor_lang::prelude::*;

#[error_code]
pub enum GameError {
    #[msg("Invalid amount to burn")]
    InvalidAmount,
    #[msg("Player already played this round")]
    AlreadyPlayed,
    #[msg("Weekly quota exceeded")]
    QuotaExceeded,
    #[msg("No rewards available")]
    NoRewards,
    #[msg("Insufficient stake")]
    InsufficientStake,
    #[msg("Too early to claim rewards")]
    TooEarlyToClaim,
    #[msg("Too early to withdraw")]
    TooEarlyToWithdraw,
    #[msg("DAO not authorized")]
    DAONotAuthorized,
    #[msg("Backup team not authorized")]
    BackupNotAuthorized,
    #[msg("Vault not in emergency state")]
    NotEmergencyState,
    #[msg("Amount exceeds available balance")]
    Overdraw,
    #[msg("Invalid governance update")]
    InvalidGovernanceUpdate,
    #[msg("Game not initialized")]
    GameNotInitialized,
    #[msg("Staking pool not initialized")]
    StakingPoolNotInitialized,
    #[msg("Vault not initialized")]
    VaultNotInitialized,
    #[msg("Invalid token mint")]
    InvalidTokenMint,
    #[msg("Invalid token account")]
    InvalidTokenAccount,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Operation not allowed in current state")]
    OperationNotAllowed,
    #[msg("Invalid backup team configuration")]
    InvalidBackupTeam,
    #[msg("Slashing amount too high")]
    SlashingAmountTooHigh,
    #[msg("Weekly reset not ready")]
    WeeklyResetNotReady,
} 