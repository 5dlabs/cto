import { Connection, Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { readFileSync } from "fs";

// Load our keypair
const keypairData = JSON.parse(readFileSync("/Users/jonathon/.config/solana/id.json", "utf8"));
const keypair = Keypair.fromSecretKey(new Uint8Array(keypairData));
const pubkey = keypair.publicKey;

console.log(`Wallet: ${pubkey.toBase58()}`);

// List of devnet RPC endpoints to try
const endpoints = [
  { name: "Devnet Public", url: "https://api.devnet.solana.com" },
  { name: "Devnet Public 2", url: "https://devnet.solana.com" },
];

// Check if we have a Helius key
if (process.env.HELIUS_API_KEY) {
  endpoints.unshift({
    name: "Helius Devnet",
    url: `https://devnet.helius-rpc.com/?api-key=${process.env.HELIUS_API_KEY}`,
  });
}

async function tryAirdrop(endpoint: { name: string; url: string }, amount: number): Promise<boolean> {
  try {
    const connection = new Connection(endpoint.url, "confirmed");
    console.log(`\n[${endpoint.name}] Requesting ${amount} SOL...`);

    const sig = await connection.requestAirdrop(pubkey, amount * LAMPORTS_PER_SOL);
    console.log(`[${endpoint.name}] Tx: ${sig}`);

    // Wait for confirmation
    const latestBlockhash = await connection.getLatestBlockhash();
    await connection.confirmTransaction({
      signature: sig,
      blockhash: latestBlockhash.blockhash,
      lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
    }, "confirmed");

    const balance = await connection.getBalance(pubkey);
    console.log(`[${endpoint.name}] SUCCESS! Balance: ${balance / LAMPORTS_PER_SOL} SOL`);
    return true;
  } catch (e: any) {
    console.log(`[${endpoint.name}] Failed: ${e.message?.slice(0, 100)}`);
    return false;
  }
}

async function main() {
  let totalSuccess = 0;

  // Try each endpoint with decreasing amounts
  for (const endpoint of endpoints) {
    for (const amount of [2, 1, 0.5]) {
      const success = await tryAirdrop(endpoint, amount);
      if (success) {
        totalSuccess++;
        // Wait a bit before next request
        await Bun.sleep(2000);
        // Try again from same endpoint
        const success2 = await tryAirdrop(endpoint, amount);
        if (success2) totalSuccess++;
        break; // Move to next endpoint
      }
      // Small delay between attempts
      await Bun.sleep(1000);
    }
  }

  // Final balance check
  const connection = new Connection("https://api.devnet.solana.com", "confirmed");
  const balance = await connection.getBalance(pubkey);
  console.log(`\n========================================`);
  console.log(`Final Balance: ${balance / LAMPORTS_PER_SOL} SOL`);
  console.log(`Successful airdrops: ${totalSuccess}`);
  console.log(`Wallet: ${pubkey.toBase58()}`);
  console.log(`========================================`);
}

main().catch(console.error);
