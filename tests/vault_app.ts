import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { VaultApp } from "../target/types/vault_app";
import { PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { assert } from "chai";

describe("vault_app", () => {
  // Configure the client to use devnet
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.VaultApp as Program<VaultApp>;
  
  // Test accounts
  const authority = provider.wallet.publicKey;
  let vaultPda: PublicKey;
  let vaultBump: number;
  let vaultFundsPda: PublicKey;
  let vaultFundsBump: number;

  before(async () => {
    // Derive PDAs
    [vaultPda, vaultBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault")],
      program.programId
    );

    [vaultFundsPda, vaultFundsBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault_pda")],
      program.programId
    );

    console.log("Program ID:", program.programId.toString());
    console.log("Vault PDA:", vaultPda.toString());
    console.log("Vault Funds PDA:", vaultFundsPda.toString());
    console.log("Authority:", authority.toString());
  });

  it("Initializes the vault", async () => {
    try {
      // Initialize vault
      const tx = await program.methods
        .initializeVault(authority)
        .accounts({
          vault: vaultPda,
          payer: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      console.log("✅ Vault initialized!");
      console.log("Transaction signature:", tx);

      // Fetch vault account
      const vaultAccount = await program.account.vault.fetch(vaultPda);
      
      assert.ok(vaultAccount.authority.equals(authority));
      assert.equal(vaultAccount.totalDeposits.toNumber(), 0);
      console.log("Vault authority:", vaultAccount.authority.toString());
      console.log("Total deposits:", vaultAccount.totalDeposits.toNumber());
    } catch (error) {
      if (error.toString().includes("already in use")) {
        console.log("Vault already initialized, continuing with tests...");
      } else {
        throw error;
      }
    }
  });

  it("Deposits SOL into the vault", async () => {
    const depositAmount = 0.1 * LAMPORTS_PER_SOL; // 0.1 SOL

    // Get initial balances
    const initialVaultBalance = await provider.connection.getBalance(vaultFundsPda);
    const initialUserBalance = await provider.connection.getBalance(provider.wallet.publicKey);
    
    console.log("Initial vault balance:", initialVaultBalance / LAMPORTS_PER_SOL, "SOL");
    console.log("Initial user balance:", initialUserBalance / LAMPORTS_PER_SOL, "SOL");

    // Deposit SOL
    const tx = await program.methods
      .deposit(new anchor.BN(depositAmount))
      .accounts({
        vault: vaultPda,
        vaultPda: vaultFundsPda,
        depositor: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("✅ Deposited 0.1 SOL!");
    console.log("Transaction signature:", tx);

    // Check balances after deposit
    const finalVaultBalance = await provider.connection.getBalance(vaultFundsPda);
    const finalUserBalance = await provider.connection.getBalance(provider.wallet.publicKey);
    
    console.log("Final vault balance:", finalVaultBalance / LAMPORTS_PER_SOL, "SOL");
    console.log("Final user balance:", finalUserBalance / LAMPORTS_PER_SOL, "SOL");

    // Verify the deposit
    assert.equal(
      finalVaultBalance - initialVaultBalance,
      depositAmount,
      "Vault balance should increase by deposit amount"
    );

    // Fetch and check vault account
    const vaultAccount = await program.account.vault.fetch(vaultPda);
    console.log("Total deposits recorded:", vaultAccount.totalDeposits.toNumber() / LAMPORTS_PER_SOL, "SOL");
  });

  it("Deposits additional SOL", async () => {
    const depositAmount = 0.05 * LAMPORTS_PER_SOL; // 0.05 SOL

    const initialVaultBalance = await provider.connection.getBalance(vaultFundsPda);

    // Deposit more SOL
    const tx = await program.methods
      .deposit(new anchor.BN(depositAmount))
      .accounts({
        vault: vaultPda,
        vaultPda: vaultFundsPda,
        depositor: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("✅ Deposited additional 0.05 SOL!");
    console.log("Transaction signature:", tx);

    const finalVaultBalance = await provider.connection.getBalance(vaultFundsPda);
    console.log("Vault balance after second deposit:", finalVaultBalance / LAMPORTS_PER_SOL, "SOL");

    // Verify the deposit
    assert.equal(
      finalVaultBalance - initialVaultBalance,
      depositAmount,
      "Vault balance should increase by second deposit amount"
    );
  });

  it("Withdraws SOL from the vault (as authority)", async () => {
    const withdrawAmount = 0.05 * LAMPORTS_PER_SOL; // 0.05 SOL

    const initialVaultBalance = await provider.connection.getBalance(vaultFundsPda);
    const initialUserBalance = await provider.connection.getBalance(provider.wallet.publicKey);

    console.log("Vault balance before withdrawal:", initialVaultBalance / LAMPORTS_PER_SOL, "SOL");

    // Withdraw SOL
    const tx = await program.methods
      .withdraw(new anchor.BN(withdrawAmount))
      .accounts({
        vault: vaultPda,
        vaultPda: vaultFundsPda,
        authority: provider.wallet.publicKey,
        recipient: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("✅ Withdrew 0.05 SOL!");
    console.log("Transaction signature:", tx);

    const finalVaultBalance = await provider.connection.getBalance(vaultFundsPda);
    const finalUserBalance = await provider.connection.getBalance(provider.wallet.publicKey);

    console.log("Vault balance after withdrawal:", finalVaultBalance / LAMPORTS_PER_SOL, "SOL");
    console.log("User balance after withdrawal:", finalUserBalance / LAMPORTS_PER_SOL, "SOL");

    // Verify the withdrawal
    assert.equal(
      initialVaultBalance - finalVaultBalance,
      withdrawAmount,
      "Vault balance should decrease by withdrawal amount"
    );
  });

  it("Fails to withdraw with unauthorized account", async () => {
    // Generate a new keypair to simulate unauthorized user
    const unauthorizedUser = anchor.web3.Keypair.generate();
    
    console.log("Testing unauthorized withdrawal (should fail)...");

    try {
      // This should fail because unauthorizedUser is not the authority
      await program.methods
        .withdraw(new anchor.BN(0.01 * LAMPORTS_PER_SOL))
        .accounts({
          vault: vaultPda,
          vaultPda: vaultFundsPda,
          authority: unauthorizedUser.publicKey, // Wrong authority
          recipient: unauthorizedUser.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([unauthorizedUser])
        .rpc();

      // If we get here, the test failed
      assert.fail("Unauthorized withdrawal should have failed");
    } catch (error) {
      console.log("✅ Correctly rejected unauthorized withdrawal");
      console.log("Error message:", error.toString().slice(0, 100), "...");
      assert.ok(error.toString().includes("unknown signer") || 
                error.toString().includes("signature verification failed") ||
                error.toString().includes("Unauthorized"));
    }
  });

  it("Checks final vault state", async () => {
    // Get final vault info
    const vaultAccount = await program.account.vault.fetch(vaultPda);
    const vaultBalance = await provider.connection.getBalance(vaultFundsPda);

    console.log("\n=== Final Vault State ===");
    console.log("Authority:", vaultAccount.authority.toString());
    console.log("Total deposits tracked:", vaultAccount.totalDeposits.toNumber() / LAMPORTS_PER_SOL, "SOL");
    console.log("Actual vault balance:", vaultBalance / LAMPORTS_PER_SOL, "SOL");
    console.log("========================\n");

    // Verify vault still has funds
    assert.isAbove(vaultBalance, 0, "Vault should still have funds");
  });
});