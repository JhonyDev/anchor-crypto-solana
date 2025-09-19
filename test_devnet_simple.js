const anchor = require("@coral-xyz/anchor");
const { SystemProgram, PublicKey, LAMPORTS_PER_SOL, Connection, Keypair } = require("@solana/web3.js");
const fs = require('fs');

// Program ID on DevNet
const PROGRAM_ID = new PublicKey("5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL");

// Test results
let testResults = {
  passed: 0,
  failed: 0,
  details: []
};

// Helper functions
function printHeader(text) {
  console.log("\n" + "=".repeat(60));
  console.log(text.padStart((60 + text.length) / 2).padEnd(60));
  console.log("=".repeat(60));
}

function printTest(name, passed, details = "") {
  if (passed) {
    testResults.passed++;
    console.log(`✅ PASSED | ${name}`);
  } else {
    testResults.failed++;
    console.log(`❌ FAILED | ${name}`);
  }
  if (details) {
    console.log(`         ${details}`);
  }
  testResults.details.push({ test: name, passed, details });
}

async function main() {
  console.log("\n╔════════════════════════════════════════════════════════╗");
  console.log("║     DEVNET VAULT MULTI-USER FUNCTIONALITY TESTS       ║");
  console.log("╚════════════════════════════════════════════════════════╝\n");

  // Setup connection
  const connection = new Connection("https://api.devnet.solana.com", "confirmed");
  
  // Load wallet from default location
  const walletKeypair = anchor.web3.Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync("/home/jhonydev/.config/solana/id.json", "utf-8")))
  );
  const wallet = new anchor.Wallet(walletKeypair);
  const provider = new anchor.AnchorProvider(connection, wallet, { commitment: "confirmed" });
  anchor.setProvider(provider);

  // Load IDL
  const idl = JSON.parse(fs.readFileSync("target/idl/vault_app.json", "utf-8"));
  const program = new anchor.Program(idl, PROGRAM_ID, provider);

  // Generate test keypairs
  const authority = Keypair.generate();
  const user1 = Keypair.generate();
  const user2 = Keypair.generate();
  const user3 = Keypair.generate();

  console.log("Test Wallets:");
  console.log(`Authority: ${authority.publicKey.toString()}`);
  console.log(`User 1: ${user1.publicKey.toString()}`);
  console.log(`User 2: ${user2.publicKey.toString()}`);
  console.log(`User 3: ${user3.publicKey.toString()}`);

  // Derive PDAs
  const [vaultPda] = PublicKey.findProgramAddressSync([Buffer.from("vault")], PROGRAM_ID);
  const [vaultFundsPda] = PublicKey.findProgramAddressSync([Buffer.from("vault_pda")], PROGRAM_ID);
  
  const getUserVaultPda = (user) => {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("user_vault"), user.toBuffer()],
      PROGRAM_ID
    );
  };

  // Request airdrops
  console.log("\nRequesting airdrops...");
  try {
    await connection.requestAirdrop(user1.publicKey, 0.5 * LAMPORTS_PER_SOL);
    await connection.requestAirdrop(user2.publicKey, 0.5 * LAMPORTS_PER_SOL);
    await connection.requestAirdrop(user3.publicKey, 0.3 * LAMPORTS_PER_SOL);
    console.log("Waiting for confirmations...");
    await new Promise(resolve => setTimeout(resolve, 10000));
  } catch (e) {
    console.log("Airdrop might be rate limited, continuing...");
  }

  // TEST 1: Check vault state
  printHeader("TEST: Vault State Check");
  try {
    const vaultAccount = await program.account.vault.fetch(vaultPda);
    printTest("Vault exists", true, `Authority: ${vaultAccount.authority.toString()}`);
    console.log(`  Total deposits: ${vaultAccount.totalDeposits.toNumber() / LAMPORTS_PER_SOL} SOL`);
  } catch (e) {
    printTest("Vault exists", false, e.message);
  }

  // TEST 2: User deposits
  printHeader("TEST: Multi-User Deposits");
  
  // User 1 deposit
  try {
    const [user1VaultPda] = getUserVaultPda(user1.publicKey);
    const depositAmount = 0.1 * LAMPORTS_PER_SOL;
    
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
    
    const userVault = await program.account.userVaultAccount.fetch(user1VaultPda);
    printTest("User 1 deposit", true, `Balance: ${userVault.currentBalance.toNumber() / LAMPORTS_PER_SOL} SOL`);
  } catch (e) {
    printTest("User 1 deposit", false, e.message);
  }

  // User 2 deposit
  try {
    const [user2VaultPda] = getUserVaultPda(user2.publicKey);
    const depositAmount = 0.15 * LAMPORTS_PER_SOL;
    
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
    
    const userVault = await program.account.userVaultAccount.fetch(user2VaultPda);
    printTest("User 2 deposit", true, `Balance: ${userVault.currentBalance.toNumber() / LAMPORTS_PER_SOL} SOL`);
  } catch (e) {
    printTest("User 2 deposit", false, e.message);
  }

  // TEST 3: User withdrawals
  printHeader("TEST: User Withdrawals");
  
  try {
    const [user1VaultPda] = getUserVaultPda(user1.publicKey);
    const withdrawAmount = 0.05 * LAMPORTS_PER_SOL;
    
    const beforeBalance = await connection.getBalance(user1.publicKey);
    
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
    
    const afterBalance = await connection.getBalance(user1.publicKey);
    const userVault = await program.account.userVaultAccount.fetch(user1VaultPda);
    
    printTest("User 1 withdrawal", true, 
      `Remaining: ${userVault.currentBalance.toNumber() / LAMPORTS_PER_SOL} SOL`);
  } catch (e) {
    printTest("User 1 withdrawal", false, e.message);
  }

  // TEST 4: Security - Unauthorized withdrawal
  printHeader("TEST: Security Features");
  
  try {
    const [user2VaultPda] = getUserVaultPda(user2.publicKey);
    
    await program.methods
      .withdraw(new anchor.BN(0.01 * LAMPORTS_PER_SOL))
      .accounts({
        vault: vaultPda,
        vaultPda: vaultFundsPda,
        userVault: user2VaultPda,  // User 2's vault
        owner: user1.publicKey,     // User 1 trying to withdraw!
        recipient: user1.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([user1])
      .rpc();
    
    printTest("Unauthorized withdrawal prevention", false, "Security breach!");
  } catch (e) {
    if (e.toString().includes("ConstraintHasOne") || e.toString().includes("constraint")) {
      printTest("Unauthorized withdrawal prevention", true, "Access denied as expected");
    } else {
      printTest("Unauthorized withdrawal prevention", false, e.message);
    }
  }

  // TEST 5: Overdraft prevention
  try {
    const [user1VaultPda] = getUserVaultPda(user1.publicKey);
    const userVault = await program.account.userVaultAccount.fetch(user1VaultPda);
    const overdraftAmount = userVault.currentBalance.toNumber() + LAMPORTS_PER_SOL;
    
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
    
    printTest("Overdraft prevention", false, "Overdraft allowed!");
  } catch (e) {
    if (e.toString().includes("InsufficientUserBalance")) {
      printTest("Overdraft prevention", true, "Overdraft blocked as expected");
    } else {
      printTest("Overdraft prevention", false, e.message);
    }
  }

  // TEST 6: Balance queries
  printHeader("TEST: Balance and Stats Queries");
  
  try {
    const [user1VaultPda] = getUserVaultPda(user1.publicKey);
    const balance = await program.methods
      .getUserBalance()
      .accounts({
        userVault: user1VaultPda,
        user: user1.publicKey,
      })
      .view();
    
    printTest("User balance query", true, `${balance.toNumber() / LAMPORTS_PER_SOL} SOL`);
  } catch (e) {
    printTest("User balance query", false, e.message);
  }

  try {
    const stats = await program.methods
      .getVaultStats()
      .accounts({
        vault: vaultPda,
        vaultPda: vaultFundsPda,
      })
      .view();
    
    printTest("Vault stats query", true, 
      `Total: ${stats[0].toNumber() / LAMPORTS_PER_SOL} SOL, Balance: ${stats[1].toNumber() / LAMPORTS_PER_SOL} SOL`);
  } catch (e) {
    printTest("Vault stats query", false, e.message);
  }

  // Print summary
  printHeader("TEST SUMMARY");
  const totalTests = testResults.passed + testResults.failed;
  const successRate = totalTests > 0 ? (testResults.passed / totalTests * 100) : 0;
  
  console.log(`Passed: ${testResults.passed}`);
  console.log(`Failed: ${testResults.failed}`);
  console.log(`Success Rate: ${successRate.toFixed(1)}%`);
  
  if (testResults.failed === 0) {
    console.log("\n🎉 ALL TESTS PASSED! 🎉");
  } else {
    console.log("\n⚠️  Some tests failed. Review the details above.");
  }

  // Save test report
  fs.writeFileSync("test_report.json", JSON.stringify(testResults, null, 2));
  console.log("\nTest report saved to test_report.json");
}

main().catch(console.error);