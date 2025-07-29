import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { WznBurnPlay } from "../target/types/wzn_burn_play";
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, createAccount, mintTo, getAccount } from "@solana/spl-token";
import { assert } from "chai";

describe("wzn-burn-play", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.WznBurnPlay as Program<WznBurnPlay>;

  // Test accounts
  const authority = Keypair.generate();
  const player = Keypair.generate();
  const backupTeam = [
    Keypair.generate().publicKey,
    Keypair.generate().publicKey,
    Keypair.generate().publicKey,
  ];

  // Token accounts
  let wznMint: PublicKey;
  let playerTokenAccount: PublicKey;
  let stakingVault: PublicKey;
  let rewardVault: PublicKey;
  let gameVault: PublicKey;
  let recoveryVault: PublicKey;

  // PDAs
  let gameStatePda: PublicKey;
  let stakingPoolPda: PublicKey;
  let recoveryVaultPda: PublicKey;
  let playerStatePda: PublicKey;
  let playerQuotaPda: PublicKey;

  before(async () => {
    // Airdrop SOL to test accounts
    await provider.connection.requestAirdrop(authority.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(player.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);

    // Create WZN token mint
    wznMint = await createMint(
      provider.connection,
      authority,
      authority.publicKey,
      null,
      6
    );

    // Create player token account
    playerTokenAccount = await createAccount(
      provider.connection,
      player,
      wznMint,
      player.publicKey
    );

    // Mint some WZN tokens to player
    await mintTo(
      provider.connection,
      authority,
      wznMint,
      playerTokenAccount,
      authority,
      1000000000 // 1000 WZN
    );

    // Derive PDAs
    [gameStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("game_state")],
      program.programId
    );

    [stakingPoolPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("staking_pool")],
      program.programId
    );

    [recoveryVaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("recovery_vault")],
      program.programId
    );

    [playerStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("player_state"), player.publicKey.toBuffer()],
      program.programId
    );

    [playerQuotaPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("player_quota"), player.publicKey.toBuffer()],
      program.programId
    );

    [stakingVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("staking_vault")],
      program.programId
    );

    [rewardVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("reward_vault")],
      program.programId
    );

    [gameVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("game_vault")],
      program.programId
    );

    [recoveryVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("recovery_vault")],
      program.programId
    );
  });

  it("Initializes the game", async () => {
    const weeklyQuota = 10;

    await program.methods
      .initializeGame(weeklyQuota)
      .accounts({
        gameState: gameStatePda,
        authority: authority.publicKey,
        wznMint: wznMint,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([authority])
      .rpc();

    const gameState = await program.account.gameState.fetch(gameStatePda);
    assert.equal(gameState.authority.toString(), authority.publicKey.toString());
    assert.equal(gameState.weeklyQuota, weeklyQuota);
    assert.equal(gameState.isInitialized, true);
  });

  it("Initializes the staking pool", async () => {
    await program.methods
      .initializeStakingPool()
      .accounts({
        stakingPool: stakingPoolPda,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([authority])
      .rpc();

    const stakingPool = await program.account.stakingPool.fetch(stakingPoolPda);
    assert.equal(stakingPool.isInitialized, true);
    assert.equal(stakingPool.totalStaked.toNumber(), 0);
  });

  it("Initializes the recovery vault", async () => {
    const backupThreshold = 100000000; // 100 WZN

    await program.methods
      .initializeVault(backupThreshold)
      .accounts({
        recoveryVault: recoveryVaultPda,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([authority])
      .rpc();

    const vault = await program.account.recoveryVault.fetch(recoveryVaultPda);
    assert.equal(vault.isInitialized, true);
    assert.equal(vault.backupThreshold.toNumber(), backupThreshold);
  });

  it("Allows player to stake tokens", async () => {
    const stakeAmount = 100000000; // 100 WZN

    await program.methods
      .stakeTokens(new anchor.BN(stakeAmount))
      .accounts({
        player: player.publicKey,
        stakingPool: stakingPoolPda,
        playerTokenAccount: playerTokenAccount,
        stakingVault: stakingVault,
        playerState: playerStatePda,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([player])
      .rpc();

    const stakingPool = await program.account.stakingPool.fetch(stakingPoolPda);
    const playerState = await program.account.playerState.fetch(playerStatePda);

    assert.equal(stakingPool.totalStaked.toNumber(), stakeAmount);
    assert.equal(playerState.totalStaked.toNumber(), stakeAmount);
  });

  it("Allows player to burn tokens to play", async () => {
    const burnAmount = 10000000; // 10 WZN

    await program.methods
      .burnToPlay(new anchor.BN(burnAmount))
      .accounts({
        player: player.publicKey,
        gameState: gameStatePda,
        playerTokenAccount: playerTokenAccount,
        wznMint: wznMint,
        playerState: playerStatePda,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([player])
      .rpc();

    const gameState = await program.account.gameState.fetch(gameStatePda);
    const playerState = await program.account.playerState.fetch(playerStatePda);

    assert.equal(gameState.totalBurned.toNumber(), burnAmount);
    assert.equal(playerState.weeklyPlaysUsed, 1);
  });

  it("Allows player to use quota play", async () => {
    // First, we need to create the player quota account
    // This would typically be done when claiming rewards
    // For this test, we'll simulate it

    await program.methods
      .useQuotaPlay()
      .accounts({
        gameState: gameStatePda,
        playerQuota: playerQuotaPda,
        player: player.publicKey,
      })
      .signers([player])
      .rpc();

    // Note: This will fail if playerQuota account doesn't exist
    // In a real scenario, this would be created during reward claiming
  });

  it("Allows DAO to update governance", async () => {
    const newDao = Keypair.generate().publicKey;

    await program.methods
      .updateGovernance(newDao)
      .accounts({
        gameState: gameStatePda,
        daoAuthority: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    const gameState = await program.account.gameState.fetch(gameStatePda);
    assert.equal(gameState.authority.toString(), newDao.toString());
  });

  it("Allows DAO to update backup team", async () => {
    const newBackupTeam = [
      Keypair.generate().publicKey,
      Keypair.generate().publicKey,
      Keypair.generate().publicKey,
    ];

    await program.methods
      .updateBackupTeam(newBackupTeam)
      .accounts({
        gameState: gameStatePda,
        daoAuthority: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    const gameState = await program.account.gameState.fetch(gameStatePda);
    assert.deepEqual(
      gameState.backupTeam.map(pk => pk.toString()),
      newBackupTeam.map(pk => pk.toString())
    );
  });
}); 