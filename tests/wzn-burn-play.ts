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
  const daoMember1 = Keypair.generate();
  const daoMember2 = Keypair.generate();
  const emergencyMember1 = Keypair.generate();
  const emergencyMember2 = Keypair.generate();

  // Token accounts
  let wznMint: PublicKey;
  let playerTokenAccount: PublicKey;
  let authorityTokenAccount: PublicKey;
  let burnVaultTokenAccount: PublicKey;
  let prizeVaultTokenAccount: PublicKey;

  // PDAs
  let gameStatePda: PublicKey;
  let burnVaultPda: PublicKey;
  let prizeVaultPda: PublicKey;
  let daoGovernancePda: PublicKey;
  let emergencyRecoveryPda: PublicKey;
  let playerPassPda: PublicKey;
  let playerScorePda: PublicKey;

  before(async () => {
    // Airdrop SOL to test accounts
    await provider.connection.requestAirdrop(authority.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(player.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(daoMember1.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(daoMember2.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);

    // Create WZN token mint
    wznMint = await createMint(
      provider.connection,
      authority,
      authority.publicKey,
      null,
      6
    );

    // Create token accounts
    playerTokenAccount = await createAccount(
      provider.connection,
      player,
      wznMint,
      player.publicKey
    );

    authorityTokenAccount = await createAccount(
      provider.connection,
      authority,
      wznMint,
      authority.publicKey
    );

    // Mint some WZN tokens
    await mintTo(
      provider.connection,
      authority,
      wznMint,
      playerTokenAccount,
      authority,
      1000000000 // 1000 WZN
    );

    await mintTo(
      provider.connection,
      authority,
      wznMint,
      authorityTokenAccount,
      authority,
      1000000000 // 1000 WZN
    );

    // Derive PDAs
    [gameStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("game_state")],
      program.programId
    );

    [burnVaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("burn_vault")],
      program.programId
    );

    [prizeVaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("prize_vault")],
      program.programId
    );

    [daoGovernancePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("dao_governance")],
      program.programId
    );

    [emergencyRecoveryPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("emergency_recovery")],
      program.programId
    );

    [playerPassPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("player_pass"), player.publicKey.toBuffer()],
      program.programId
    );

    [playerScorePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("player_score"), player.publicKey.toBuffer()],
      program.programId
    );

    [burnVaultTokenAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("burn_vault")],
      program.programId
    );

    [prizeVaultTokenAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("prize_vault")],
      program.programId
    );
  });

  it("Initializes the game", async () => {
    const monthlyPassCost = 10000000; // 10 WZN

    await program.methods
      .initializeGame(new anchor.BN(monthlyPassCost))
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
    assert.equal(gameState.monthlyPassCost.toNumber(), monthlyPassCost);
    assert.equal(gameState.isInitialized, true);
  });

  it("Initializes the burn vault", async () => {
    const emergencyThreshold = 800000000000; // 80% of 1B supply
    const minimumBalance = 10000000000; // 10M WZN

    await program.methods
      .initializeBurnVault(new anchor.BN(emergencyThreshold), new anchor.BN(minimumBalance))
      .accounts({
        burnVault: burnVaultPda,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([authority])
      .rpc();

    const burnVault = await program.account.burnVault.fetch(burnVaultPda);
    assert.equal(burnVault.isInitialized, true);
    assert.equal(burnVault.totalLocked.toNumber(), 0);
    assert.equal(burnVault.emergencyUnlockThreshold.toNumber(), emergencyThreshold);
  });

  it("Initializes the prize vault", async () => {
    await program.methods
      .initializePrizeVault()
      .accounts({
        prizeVault: prizeVaultPda,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([authority])
      .rpc();

    const prizeVault = await program.account.prizeVault.fetch(prizeVaultPda);
    assert.equal(prizeVault.isInitialized, true);
    assert.equal(prizeVault.totalDeposited.toNumber(), 0);
  });

  it("Initializes DAO governance", async () => {
    const daoMembers = [daoMember1.publicKey, daoMember2.publicKey];

    await program.methods
      .initializeDao(daoMembers)
      .accounts({
        daoGovernance: daoGovernancePda,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([authority])
      .rpc();

    const daoGovernance = await program.account.daoGovernance.fetch(daoGovernancePda);
    assert.equal(daoGovernance.isInitialized, true);
    assert.equal(daoGovernance.totalMembers, 2);
    assert.equal(daoGovernance.daoMembers.length, 2);
  });

  it("Initializes emergency recovery", async () => {
    const emergencyMembers = [emergencyMember1.publicKey, emergencyMember2.publicKey];

    await program.methods
      .initializeEmergencyRecovery(emergencyMembers)
      .accounts({
        emergencyRecovery: emergencyRecoveryPda,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([authority])
      .rpc();

    const emergencyRecovery = await program.account.emergencyRecovery.fetch(emergencyRecoveryPda);
    assert.equal(emergencyRecovery.isInitialized, true);
    assert.equal(emergencyRecovery.totalMembers, 2);
    assert.equal(emergencyRecovery.backupMembers.length, 2);
  });

  it("Allows player to burn tokens for monthly pass", async () => {
    const burnAmount = 10000000; // 10 WZN

    await program.methods
      .burnToPlay(new anchor.BN(burnAmount))
      .accounts({
        gameState: gameStatePda,
        burnVault: burnVaultPda,
        playerPass: playerPassPda,
        playerTokenAccount: playerTokenAccount,
        burnVaultTokenAccount: burnVaultTokenAccount,
        player: player.publicKey,
        wznMint: wznMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([player])
      .rpc();

    const gameState = await program.account.gameState.fetch(gameStatePda);
    const burnVault = await program.account.burnVault.fetch(burnVaultPda);
    const playerPass = await program.account.playerPass.fetch(playerPassPda);

    assert.equal(gameState.totalBurned.toNumber(), burnAmount);
    assert.equal(burnVault.totalLocked.toNumber(), burnAmount);
    assert.equal(playerPass.isActive, true);
    assert.equal(playerPass.totalPassesPurchased, 1);
  });

  it("Allows checking game access", async () => {
    await program.methods
      .checkGameAccess()
      .accounts({
        gameState: gameStatePda,
        playerPass: playerPassPda,
        player: player.publicKey,
      })
      .signers([player])
      .rpc();

    // If we reach here, access was granted
    assert(true);
  });

  it("Allows depositing to prize vault", async () => {
    const depositAmount = 50000000; // 50 WZN

    await program.methods
      .depositToPrizeVault(new anchor.BN(depositAmount))
      .accounts({
        prizeVault: prizeVaultPda,
        fromTokenAccount: authorityTokenAccount,
        prizeVaultTokenAccount: prizeVaultTokenAccount,
        gameState: gameStatePda,
        authority: authority.publicKey,
        wznMint: wznMint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([authority])
      .rpc();

    const prizeVault = await program.account.prizeVault.fetch(prizeVaultPda);
    assert.equal(prizeVault.totalDeposited.toNumber(), depositAmount);
  });

  it("Allows creating DAO proposal", async () => {
    await program.methods
      .createProposal(
        { updateMonthlyPassCost: {} },
        new anchor.BN(15000000), // 15 WZN
        "Update monthly pass cost"
      )
      .accounts({
        daoGovernance: daoGovernancePda,
        proposer: daoMember1.publicKey,
      })
      .signers([daoMember1])
      .rpc();

    const daoGovernance = await program.account.daoGovernance.fetch(daoGovernancePda);
    assert.equal(daoGovernance.pendingProposals.length, 1);
  });

  it("Allows voting on proposal", async () => {
    await program.methods
      .voteOnProposal(0, true) // Vote FOR proposal 0
      .accounts({
        daoGovernance: daoGovernancePda,
        voter: daoMember1.publicKey,
      })
      .signers([daoMember1])
      .rpc();

    await program.methods
      .voteOnProposal(0, true) // Vote FOR proposal 0
      .accounts({
        daoGovernance: daoGovernancePda,
        voter: daoMember2.publicKey,
      })
      .signers([daoMember2])
      .rpc();

    const daoGovernance = await program.account.daoGovernance.fetch(daoGovernancePda);
    const proposal = daoGovernance.pendingProposals[0];
    assert.equal(proposal.totalVotes, 2);
    assert.equal(proposal.votesFor, 2);
  });

  it("Allows executing proposal", async () => {
    await program.methods
      .executeProposal(0)
      .accounts({
        daoGovernance: daoGovernancePda,
        burnVault: burnVaultPda,
        prizeVault: prizeVaultPda,
        gameState: gameStatePda,
        executor: daoMember1.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([daoMember1])
      .rpc();

    const gameState = await program.account.gameState.fetch(gameStatePda);
    assert.equal(gameState.monthlyPassCost.toNumber(), 15000000); // Updated to 15 WZN
  });

  it("Allows updating player score", async () => {
    await program.methods
      .updatePlayerScore(5, 3, 50) // 5 games played, 3 won, +50 rating
      .accounts({
        playerScore: playerScorePda,
        playerPass: playerPassPda,
        player: player.publicKey,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([player])
      .rpc();

    const playerScore = await program.account.playerScore.fetch(playerScorePda);
    assert.equal(playerScore.totalGamesPlayed, 5);
    assert.equal(playerScore.totalGamesWon, 3);
    assert.equal(playerScore.currentRating, 1050); // 1000 + 50
  });

  it("Allows distributing prizes", async () => {
    const prizeAmount = 10000000; // 10 WZN

    await program.methods
      .distributePrize(new anchor.BN(prizeAmount))
      .accounts({
        prizeVault: prizeVaultPda,
        prizeVaultTokenAccount: prizeVaultTokenAccount,
        recipientTokenAccount: playerTokenAccount,
        playerScore: playerScorePda,
        gameState: gameStatePda,
        recipient: player.publicKey,
        wznMint: wznMint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([player])
      .rpc();

    const prizeVault = await program.account.prizeVault.fetch(prizeVaultPda);
    const playerScore = await program.account.playerScore.fetch(playerScorePda);
    const gameState = await program.account.gameState.fetch(gameStatePda);

    assert.equal(prizeVault.totalDistributed.toNumber(), prizeAmount);
    assert.equal(playerScore.totalPrizesEarned.toNumber(), prizeAmount);
    assert.equal(gameState.totalPrizesDistributed.toNumber(), prizeAmount);
  });

  it("Allows monthly reset", async () => {
    await program.methods
      .monthlyReset()
      .accounts({
        gameState: gameStatePda,
        authority: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    const gameState = await program.account.gameState.fetch(gameStatePda);
    assert(gameState.lastMonthlyReset > 0);
  });
}); 