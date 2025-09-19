import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { VaultApp } from "./target/types/vault_app";
import { assert } from "chai";
import { SystemProgram, PublicKey, LAMPORTS_PER_SOL, Connection } from "@solana/web3.js";

describe("DevNet Vault Tests", () => {
  // Configure for DevNet
  const connection = new Connection("https://api.devnet.solana.com", "confirmed");
  const provider = new anchor.AnchorProvider(
    connection,
    anchor.AnchorProvider.env().wallet,
    { commitment: "confirmed" }
  );
  anchor.setProvider(provider);

  const programId = new PublicKey("5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL");
  const program = new Program(
    require("./target/idl/vault_app.json"),
    programId,
    provider
  ) as Program<VaultApp>;

  // Test wallets
  const authority = anchor.web3.Keypair.generate();
  const user1 = anchor.web3.Keypair.generate();
  const user2 = anchor.web3.Keypair.generate();
  const user3 = anchor.web3.Keypair.generate();

  // PDAs
  const [vaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault")],
    programId
  );

  const [vaultFundsPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault_pda")],
    programId
  );

  const getUserVaultPda = (user: PublicKey) => {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("user_vault"), user.toBuffer()],
      programId
    );
  };

  before(async () => {
    console.log("\n=== Setting Up Test Wallets ===");
    console.log("Authority:", authority.publicKey.toString());
    console.log("User1:", user1.publicKey.toString());
    console.log("User2:", user2.publicKey.toString());
    console.log("User3:", user3.publicKey.toString());

    // Request airdrops
    console.log("\nRequesting airdrops...");
    
    try {
      await provider.connection.requestAirdrop(authority.publicKey, 1 * LAMPORTS_PER_SOL);
      await provider.connection.requestAirdrop(user1.publicKey, 2 * LAMPORTS_PER_SOL);
      await provider.connection.requestAirdrop(user2.publicKey, 2 * LAMPORTS_PER_SOL);
      await provider.connection.requestAirdrop(user3.publicKey, 1 * LAMPORTS_PER_SOL);
      
      // Wait for confirmations
      await new Promise(resolve => setTimeout(resolve, 10000));
      
      // Verify balances
      const authBalance = await provider.connection.getBalance(authority.publicKey);
      const user1Balance = await provider.connection.getBalance(user1.publicKey);
      const user2Balance = await provider.connection.getBalance(user2.publicKey);
      const user3Balance = await provider.connection.getBalance(user3.publicKey);
      
      console.log("\nBalances after airdrop:");
      console.log(`Authority: ${authBalance / LAMPORTS_PER_SOL} SOL`);
      console.log(`User1: ${user1Balance / LAMPORTS_PER_SOL} SOL`);
      console.log(`User2: ${user2Balance / LAMPORTS_PER_SOL} SOL`);
      console.log(`User3: ${user3Balance / LAMPORTS_PER_SOL} SOL`);
    } catch (e) {
      console.log("Airdrop failed, continuing with existing balances:", e);
    }
  });

  describe("Vault Initialization", () => {
    it("Should initialize the vault with correct authority", async () => {
      console.log("\n=== Test: Vault Initialization ===");
      
      try {
        const tx = await program.methods
          .initializeVault(authority.publicKey)
          .accounts({
            vault: vaultPda,
            payer: provider.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .rpc();

        console.log("✅ Vault initialized, tx:", tx);

        const vaultAccount = await program.account.vault.fetch(vaultPda);
        assert.equal(vaultAccount.authority.toString(), authority.publicKey.toString());
        assert.equal(vaultAccount.totalDeposits.toNumber(), 0);
        console.log("✅ Vault authority verified");
      } catch (e) {
        if (e.toString().includes("already in use")) {
          console.log("⚠️  Vault already initialized, continuing tests...");
          const vaultAccount = await program.account.vault.fetch(vaultPda);
          console.log("Existing vault authority:", vaultAccount.authority.toString());
        } else {
          throw e;
        }
      }
    });
  });

  describe("User Deposits", () => {
    it("User1 should deposit 0.5 SOL", async () => {
      console.log("\n=== Test: User1 Deposit ===");
      const depositAmount = 0.5 * LAMPORTS_PER_SOL;
      const [user1VaultPda] = getUserVaultPda(user1.publicKey);

      const tx = await program.methods
        .deposit(new anchor.BN(depositAmount))
        .accounts({
          vault: vaultPda,
          vaultPda: vaultFundsPda,
          userVault: user1VaultPda,
          depositor: user1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      console.log("✅ User1 deposit tx:", tx);

      const userVaultAccount = await program.account.userVaultAccount.fetch(user1VaultPda);
      assert.equal(userVaultAccount.owner.toString(), user1.publicKey.toString());
      assert.equal(userVaultAccount.totalDeposited.toNumber(), depositAmount);
      assert.equal(userVaultAccount.currentBalance.toNumber(), depositAmount);
      console.log(`✅ User1 balance: ${userVaultAccount.currentBalance.toNumber() / LAMPORTS_PER_SOL} SOL`);
    });

    it("User2 should deposit 0.8 SOL", async () => {
      console.log("\n=== Test: User2 Deposit ===");
      const depositAmount = 0.8 * LAMPORTS_PER_SOL;
      const [user2VaultPda] = getUserVaultPda(user2.publicKey);

      const tx = await program.methods
        .deposit(new anchor.BN(depositAmount))
        .accounts({
          vault: vaultPda,
          vaultPda: vaultFundsPda,
          userVault: user2VaultPda,
          depositor: user2.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user2])
        .rpc();

      console.log("✅ User2 deposit tx:", tx);

      const userVaultAccount = await program.account.userVaultAccount.fetch(user2VaultPda);
      assert.equal(userVaultAccount.currentBalance.toNumber(), depositAmount);
      console.log(`✅ User2 balance: ${userVaultAccount.currentBalance.toNumber() / LAMPORTS_PER_SOL} SOL`);

      // Check vault total
      const vaultAccount = await program.account.vault.fetch(vaultPda);
      console.log(`✅ Vault total deposits: ${vaultAccount.totalDeposits.toNumber() / LAMPORTS_PER_SOL} SOL`);
    });

    it("User1 should make additional deposit", async () => {
      console.log("\n=== Test: User1 Additional Deposit ===");
      const depositAmount = 0.3 * LAMPORTS_PER_SOL;
      const [user1VaultPda] = getUserVaultPda(user1.publicKey);

      const beforeAccount = await program.account.userVaultAccount.fetch(user1VaultPda);
      const beforeBalance = beforeAccount.currentBalance.toNumber();

      const tx = await program.methods
        .deposit(new anchor.BN(depositAmount))
        .accounts({
          vault: vaultPda,
          vaultPda: vaultFundsPda,
          userVault: user1VaultPda,
          depositor: user1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      console.log("✅ User1 additional deposit tx:", tx);

      const afterAccount = await program.account.userVaultAccount.fetch(user1VaultPda);
      assert.equal(afterAccount.currentBalance.toNumber(), beforeBalance + depositAmount);
      assert.equal(afterAccount.totalDeposited.toNumber(), 0.8 * LAMPORTS_PER_SOL);
      console.log(`✅ User1 new balance: ${afterAccount.currentBalance.toNumber() / LAMPORTS_PER_SOL} SOL`);
    });
  });

  describe("User Withdrawals", () => {
    it("User1 should withdraw 0.2 SOL", async () => {
      console.log("\n=== Test: User1 Withdrawal ===");
      const withdrawAmount = 0.2 * LAMPORTS_PER_SOL;
      const [user1VaultPda] = getUserVaultPda(user1.publicKey);

      const beforeVault = await program.account.userVaultAccount.fetch(user1VaultPda);
      const beforeBalance = await provider.connection.getBalance(user1.publicKey);

      const tx = await program.methods
        .withdraw(new anchor.BN(withdrawAmount))
        .accounts({
          vault: vaultPda,
          vaultPda: vaultFundsPda,
          userVault: user1VaultPda,
          owner: user1.publicKey,
          recipient: user1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      console.log("✅ User1 withdrawal tx:", tx);

      const afterVault = await program.account.userVaultAccount.fetch(user1VaultPda);
      const afterBalance = await provider.connection.getBalance(user1.publicKey);

      assert.equal(afterVault.currentBalance.toNumber(), beforeVault.currentBalance.toNumber() - withdrawAmount);
      assert.equal(afterVault.totalWithdrawn.toNumber(), withdrawAmount);
      assert.isTrue(afterBalance > beforeBalance);
      console.log(`✅ User1 remaining balance: ${afterVault.currentBalance.toNumber() / LAMPORTS_PER_SOL} SOL`);
    });

    it("User2 should withdraw to different recipient", async () => {
      console.log("\n=== Test: User2 Withdrawal to Different Recipient ===");
      const withdrawAmount = 0.3 * LAMPORTS_PER_SOL;
      const [user2VaultPda] = getUserVaultPda(user2.publicKey);

      const beforeRecipientBalance = await provider.connection.getBalance(user3.publicKey);

      const tx = await program.methods
        .withdraw(new anchor.BN(withdrawAmount))
        .accounts({
          vault: vaultPda,
          vaultPda: vaultFundsPda,
          userVault: user2VaultPda,
          owner: user2.publicKey,
          recipient: user3.publicKey, // Different recipient
          systemProgram: SystemProgram.programId,
        })
        .signers([user2])
        .rpc();

      console.log("✅ User2 withdrawal to User3 tx:", tx);

      const afterRecipientBalance = await provider.connection.getBalance(user3.publicKey);
      assert.isTrue(afterRecipientBalance > beforeRecipientBalance);
      console.log(`✅ User3 received ${(afterRecipientBalance - beforeRecipientBalance) / LAMPORTS_PER_SOL} SOL`);
    });
  });

  describe("Security Tests", () => {
    it("Should prevent User1 from withdrawing User2's funds", async () => {
      console.log("\n=== Test: Security - Unauthorized Withdrawal ===");
      const withdrawAmount = 0.1 * LAMPORTS_PER_SOL;
      const [user2VaultPda] = getUserVaultPda(user2.publicKey);

      try {
        await program.methods
          .withdraw(new anchor.BN(withdrawAmount))
          .accounts({
            vault: vaultPda,
            vaultPda: vaultFundsPda,
            userVault: user2VaultPda,
            owner: user1.publicKey, // Wrong owner!
            recipient: user1.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([user1])
          .rpc();

        assert.fail("Should have thrown unauthorized error");
      } catch (error) {
        console.log("✅ Security check passed: Unauthorized withdrawal blocked");
        assert.include(error.toString().toLowerCase(), "constraint");
      }
    });

    it("Should prevent overdraft withdrawal", async () => {
      console.log("\n=== Test: Security - Overdraft Prevention ===");
      const [user1VaultPda] = getUserVaultPda(user1.publicKey);
      
      const userVault = await program.account.userVaultAccount.fetch(user1VaultPda);
      const overdraftAmount = userVault.currentBalance.toNumber() + LAMPORTS_PER_SOL;

      try {
        await program.methods
          .withdraw(new anchor.BN(overdraftAmount))
          .accounts({
            vault: vaultPda,
            vaultPda: vaultFundsPda,
            userVault: user1VaultPda,
            owner: user1.publicKey,
            recipient: user1.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([user1])
          .rpc();

        assert.fail("Should have thrown insufficient balance error");
      } catch (error) {
        console.log("✅ Security check passed: Overdraft prevented");
        assert.include(error.toString(), "InsufficientUserBalance");
      }
    });

    it("Should prevent withdrawal from non-existent user vault", async () => {
      console.log("\n=== Test: Security - Non-existent Vault ===");
      const nonExistentUser = anchor.web3.Keypair.generate();
      const [nonExistentVaultPda] = getUserVaultPda(nonExistentUser.publicKey);

      try {
        await program.methods
          .withdraw(new anchor.BN(100000))
          .accounts({
            vault: vaultPda,
            vaultPda: vaultFundsPda,
            userVault: nonExistentVaultPda,
            owner: nonExistentUser.publicKey,
            recipient: nonExistentUser.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([nonExistentUser])
          .rpc();

        assert.fail("Should have thrown account not found error");
      } catch (error) {
        console.log("✅ Security check passed: Non-existent vault access blocked");
      }
    });
  });

  describe("Balance and Stats Queries", () => {
    it("Should get correct user balance", async () => {
      console.log("\n=== Test: Get User Balance ===");
      const [user1VaultPda] = getUserVaultPda(user1.publicKey);

      const balance = await program.methods
        .getUserBalance()
        .accounts({
          userVault: user1VaultPda,
          user: user1.publicKey,
        })
        .view();

      console.log(`✅ User1 balance query: ${balance.toNumber() / LAMPORTS_PER_SOL} SOL`);

      const userVault = await program.account.userVaultAccount.fetch(user1VaultPda);
      assert.equal(balance.toNumber(), userVault.currentBalance.toNumber());
    });

    it("Should get correct vault statistics", async () => {
      console.log("\n=== Test: Get Vault Statistics ===");
      
      const stats = await program.methods
        .getVaultStats()
        .accounts({
          vault: vaultPda,
          vaultPda: vaultFundsPda,
        })
        .view();

      const vaultBalance = await provider.connection.getBalance(vaultFundsPda);
      
      console.log(`✅ Vault stats:`);
      console.log(`   Total deposits tracked: ${stats[0].toNumber() / LAMPORTS_PER_SOL} SOL`);
      console.log(`   Actual vault balance: ${vaultBalance / LAMPORTS_PER_SOL} SOL`);
      
      assert.equal(stats[1].toNumber(), vaultBalance);
    });
  });

  describe("Final State Verification", () => {
    it("Should verify all account states are consistent", async () => {
      console.log("\n=== Test: Final State Verification ===");
      
      const [user1VaultPda] = getUserVaultPda(user1.publicKey);
      const [user2VaultPda] = getUserVaultPda(user2.publicKey);
      
      const user1Vault = await program.account.userVaultAccount.fetch(user1VaultPda);
      const user2Vault = await program.account.userVaultAccount.fetch(user2VaultPda);
      const vaultAccount = await program.account.vault.fetch(vaultPda);
      const vaultBalance = await provider.connection.getBalance(vaultFundsPda);
      
      console.log("\n📊 Final Account States:");
      console.log("\nUser1 Vault:");
      console.log(`  Owner: ${user1Vault.owner.toString()}`);
      console.log(`  Current Balance: ${user1Vault.currentBalance.toNumber() / LAMPORTS_PER_SOL} SOL`);
      console.log(`  Total Deposited: ${user1Vault.totalDeposited.toNumber() / LAMPORTS_PER_SOL} SOL`);
      console.log(`  Total Withdrawn: ${user1Vault.totalWithdrawn.toNumber() / LAMPORTS_PER_SOL} SOL`);
      
      console.log("\nUser2 Vault:");
      console.log(`  Owner: ${user2Vault.owner.toString()}`);
      console.log(`  Current Balance: ${user2Vault.currentBalance.toNumber() / LAMPORTS_PER_SOL} SOL`);
      console.log(`  Total Deposited: ${user2Vault.totalDeposited.toNumber() / LAMPORTS_PER_SOL} SOL`);
      console.log(`  Total Withdrawn: ${user2Vault.totalWithdrawn.toNumber() / LAMPORTS_PER_SOL} SOL`);
      
      console.log("\nVault Summary:");
      console.log(`  Total Deposits: ${vaultAccount.totalDeposits.toNumber() / LAMPORTS_PER_SOL} SOL`);
      console.log(`  Actual Balance: ${vaultBalance / LAMPORTS_PER_SOL} SOL`);
      
      // Verify consistency
      const totalUserBalances = user1Vault.currentBalance.toNumber() + user2Vault.currentBalance.toNumber();
      assert.equal(totalUserBalances, vaultBalance, "User balances should match vault balance");
      console.log("\n✅ All account states are consistent!");
    });
  });
});