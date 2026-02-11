import { ethers } from 'ethers';

const SBT_ABI = [
  "function issueIdentity(string handle, bytes hexId, string uri) payable returns (uint256)",
  "function getIdentity(uint256 tokenId) view returns (string handle, uint256 score, uint8 tier, bool isActive, uint256 recommendations, uint256 reports)",
  "function calculateFee() view returns (uint256)"
];

async function main() {
  const provider = new ethers.JsonRpcProvider(process.env.EVM_RPC_URL);
  const wallet = new ethers.Wallet(process.env.PRIVATE_KEY, provider);
  const contractAddress = process.env.CONTRACT_ADDRESS;

  const sbtContract = new ethers.Contract(contractAddress, SBT_ABI, wallet);

  console.log("Issuing identity for @nebulon_test...");
  
  const fee = await sbtContract.calculateFee();
  const hexId = ethers.hexlify(ethers.randomBytes(512));
  
  const tx = await sbtContract.issueIdentity(
    "@nebulon_test",
    hexId,
    "https://api.nebulon.io/metadata/test",
    { value: fee }
  );

  console.log("Transaction sent:", tx.hash);
  const receipt = await tx.wait();
  console.log("Identity issued in block:", receipt.blockNumber);
}

main().catch(console.error);
