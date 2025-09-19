import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { VaultApp } from "../target/types/vault_app";
import { assert } from "chai";
import { SystemProgram, PublicKey, LAMPORTS_PER_SOL } from "@solana/web3.js";

describe("vault_app", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.VaultApp as Program<VaultApp>;
  const provider = anchor.getProvider();

  const authority = anchor.web3.Keypair.generate();
  const user1 = anchor.web3.Keypair.generate();
  const user2 = anchor.web3.Keypair.generate();

  const [vaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault")],
    program.programId
  );

  const [vaultFundsPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault_pda")],
    program.programId
  );

  const getUserVaultPda = (user: PublicKey) => {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("user_vault"), user.toBuffer()],
      program.programId
    );
  };

  before(async () => {
    const airdropSignature1 = await provider.connection.requestAirdrop(
      user1.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSignature1);

    const airdropSignature2 = await provider.connection.requestAirdrop(
      user2.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSignature2);

    const airdropSignature3 = await provider.connection.requestAirdrop(
      authority.publicKey,
      1 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSignature3);
  });

  it("Initializes the vault", async () => {
    const tx = await program.methods
      .initializeVault(authority.publicKey)
      .accounts({
        vault: vaultPda,
        payer: provider.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Vault initialized with transaction:", tx);

    const vaultAccount = await program.account.vault.fetch(vaultPda);
    assert.equal(vaultAccount.authority.toString(), authority.publicKey.toString());
    assert.equal(vaultAccount.totalDeposits.toNumber(), 0);
  });

  it("User1 deposits to the vault", async () => {
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

    console.log("User1 deposit transaction:", tx);

    const userVaultAccount = await program.account.userVaultAccount.fetch(user1VaultPda);
    assert.equal(userVaultAccount.owner.toString(), user1.publicKey.toString());
    assert.equal(userVaultAccount.totalDeposited.toNumber(), depositAmount);
    assert.equal(userVaultAccount.currentBalance.toNumber(), depositAmount);
    assert.equal(userVaultAccount.totalWithdrawn.toNumber(), 0);

    const vaultAccount = await program.account.vault.fetch(vaultPda);
    assert.equal(vaultAccount.totalDeposits.toNumber(), depositAmount);

    const vaultFundsBalance = await provider.connection.getBalance(vaultFundsPda);
    assert.equal(vaultFundsBalance, depositAmount);
  });

  it("User2 deposits to the vault", async () => {
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

    console.log("User2 deposit transaction:", tx);

    const userVaultAccount = await program.account.userVaultAccount.fetch(user2VaultPda);
    assert.equal(userVaultAccount.owner.toString(), user2.publicKey.toString());
    assert.equal(userVaultAccount.totalDeposited.toNumber(), depositAmount);
    assert.equal(userVaultAccount.currentBalance.toNumber(), depositAmount);

    const vaultAccount = await program.account.vault.fetch(vaultPda);
    assert.equal(vaultAccount.totalDeposits.toNumber(), 1.3 * LAMPORTS_PER_SOL);

    const vaultFundsBalance = await provider.connection.getBalance(vaultFundsPda);
    assert.equal(vaultFundsBalance, 1.3 * LAMPORTS_PER_SOL);
  });

  it("User1 withdraws from the vault", async () => {
    const withdrawAmount = 0.2 * LAMPORTS_PER_SOL;
    const [user1VaultPda] = getUserVaultPda(user1.publicKey);

    const initialBalance = await provider.connection.getBalance(user1.publicKey);

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

    console.log("User1 withdrawal transaction:", tx);

    const userVaultAccount = await program.account.userVaultAccount.fetch(user1VaultPda);
    assert.equal(userVaultAccount.currentBalance.toNumber(), 0.3 * LAMPORTS_PER_SOL);
    assert.equal(userVaultAccount.totalWithdrawn.toNumber(), withdrawAmount);

    const vaultAccount = await program.account.vault.fetch(vaultPda);
    assert.equal(vaultAccount.totalDeposits.toNumber(), 1.1 * LAMPORTS_PER_SOL);

    const vaultFundsBalance = await provider.connection.getBalance(vaultFundsPda);
    assert.equal(vaultFundsBalance, 1.1 * LAMPORTS_PER_SOL);

    const finalBalance = await provider.connection.getBalance(user1.publicKey);
    assert.isTrue(finalBalance > initialBalance);
  });

  it("User2 cannot withdraw more than their balance", async () => {
    const withdrawAmount = 1 * LAMPORTS_PER_SOL;
    const [user2VaultPda] = getUserVaultPda(user2.publicKey);

    try {
      await program.methods
        .withdraw(new anchor.BN(withdrawAmount))
        .accounts({
          vault: vaultPda,
          vaultPda: vaultFundsPda,
          userVault: user2VaultPda,
          owner: user2.publicKey,
          recipient: user2.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user2])
        .rpc();

      assert.fail("Should have thrown an error");
    } catch (error) {
      assert.include(error.toString(), "InsufficientUserBalance");
    }
  });

  it("User1 cannot withdraw from User2's account", async () => {
    const withdrawAmount = 0.1 * LAMPORTS_PER_SOL;
    const [user2VaultPda] = getUserVaultPda(user2.publicKey);

    try {
      await program.methods
        .withdraw(new anchor.BN(withdrawAmount))
        .accounts({
          vault: vaultPda,
          vaultPda: vaultFundsPda,
          userVault: user2VaultPda,
          owner: user1.publicKey,
          recipient: user1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      assert.fail("Should have thrown an error");
    } catch (error) {
      assert.include(error.toString(), "ConstraintHasOne");
    }
  });

  it("Gets user balance", async () => {
    const [user1VaultPda] = getUserVaultPda(user1.publicKey);

    const balance = await program.methods
      .getUserBalance()
      .accounts({
        userVault: user1VaultPda,
        user: user1.publicKey,
      })
      .view();

    console.log("User1 balance:", balance.toNumber());
    assert.equal(balance.toNumber(), 0.3 * LAMPORTS_PER_SOL);
  });

  it("Gets vault stats", async () => {
    const stats = await program.methods
      .getVaultStats()
      .accounts({
        vault: vaultPda,
        vaultPda: vaultFundsPda,
      })
      .view();

    console.log("Vault stats - Total deposits:", stats[0].toNumber());
    console.log("Vault stats - Current balance:", stats[1].toNumber());

    assert.equal(stats[0].toNumber(), 1.1 * LAMPORTS_PER_SOL);
    assert.equal(stats[1].toNumber(), 1.1 * LAMPORTS_PER_SOL);
  });

  it("User1 deposits again to existing account", async () => {
    const depositAmount = 0.3 * LAMPORTS_PER_SOL;
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

    console.log("User1 second deposit transaction:", tx);

    const userVaultAccount = await program.account.userVaultAccount.fetch(user1VaultPda);
    assert.equal(userVaultAccount.totalDeposited.toNumber(), 0.8 * LAMPORTS_PER_SOL);
    assert.equal(userVaultAccount.currentBalance.toNumber(), 0.6 * LAMPORTS_PER_SOL);

    const vaultAccount = await program.account.vault.fetch(vaultPda);
    assert.equal(vaultAccount.totalDeposits.toNumber(), 1.4 * LAMPORTS_PER_SOL);
  });
});