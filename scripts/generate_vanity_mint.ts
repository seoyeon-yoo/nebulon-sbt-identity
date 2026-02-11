import { Keypair } from "@solana/web3.js";
import fs from "fs";

// Function to find a vanity address ending with a specific suffix
function findVanityAddress(suffix: string): Keypair {
    let keypair = Keypair.generate();
    let count = 0;
    const start = Date.now();

    while (!keypair.publicKey.toBase58().endsWith(suffix)) {
        keypair = Keypair.generate();
        count++;
        if (count % 100000 === 0) {
            console.log(`Checked ${count} keys... (${(count / ((Date.now() - start) / 1000)).toFixed(0)} keys/sec)`);
        }
    }

    console.log(`Found vanity address: ${keypair.publicKey.toBase58()}`);
    return keypair;
}

const SUFFIX = "NEBU";
console.log(`Searching for mint address ending with "${SUFFIX}"...`);
const vanityKeypair = findVanityAddress(SUFFIX);

const secretKey = Array.from(vanityKeypair.secretKey);
const keypairPath = "vanity_mint_nebu.json";

fs.writeFileSync(keypairPath, JSON.stringify(secretKey));
console.log(`Saved keypair to ${keypairPath}`);
