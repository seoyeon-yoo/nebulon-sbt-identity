import os
import httpx
import asyncio
import hashlib
import json
from fastapi import FastAPI, HTTPException, BackgroundTasks
from pydantic import BaseModel
from typing import Dict, List, Optional
from bs4 import BeautifulSoup
from dotenv import load_dotenv

# Solana Imports
from solana.rpc.async_api import AsyncClient
from solders.transaction import Transaction
from solders.keypair import Keypair
from solders.pubkey import Pubkey
from solders.instruction import Instruction, AccountMeta
from solders.system_program import ID as SYS_PROGRAM_ID
from solders.message import Message
import base58
import struct

load_dotenv()

app = FastAPI(title="Nebulon SBT Identity Backend")

# Constants
PROGRAM_ID = Pubkey.from_string("8AWzFHnngTCJQTGEQC2M1VHLEa52tXAkXKjzTMY2oxD1") # From lib.rs
RPC_URL = os.getenv("SOLANA_RPC_URL", "https://api.devnet.solana.com")
ADMIN_KEYPAIR_PATH = os.getenv("ADMIN_KEYPAIR_PATH", "../../mainnet-program-keypair.json") 

# Tier to Metadata URI Mapping (IPFS)
TIER_METADATA_MAP = {
    1: "https://ipfs.io/ipfs/QmYY1Dx83eZZFK5jYHfmoG8bCcZzJV5tndFxBkqR1qtTBS",
    2: "https://ipfs.io/ipfs/QmWnEPKLpACtaQzR99PSBReMgGTz4aSg2aYfcycLAWbaoE",
    3: "https://ipfs.io/ipfs/QmZeEzNvi2KzabVY6H8gpMJqe12yMDFaFw8xpVf6WcCcQK",
    4: "https://ipfs.io/ipfs/QmZR9kEMwKPCZ5tiDBEfGmy1ow2DqQv7o3JmXc7WLKn8pQ",
    5: "https://ipfs.io/ipfs/QmSqnQEfpuroog6VmLrx1byFGjGQdhR6z1pQVzRjAK2Bdx",
    6: "https://ipfs.io/ipfs/QmYr4SpuTR8N3meZC3UpJkpK1yM2ZxxSG62rZYjGqSsYdg",
    7: "https://ipfs.io/ipfs/QmeR1xvuMBdNXiQpLhyWwZSAi2jss9V5EqPjdJeU2h55vm",
    8: "https://ipfs.io/ipfs/QmbRRi252mYmpTpBLvWGhAP9Z93CEXXhzfFMLm29jyix7S",
    9: "https://ipfs.io/ipfs/QmfEiQSGBY447aSU1panm9EbuSufaJ2GQ6nmRUDuobMPLG",
    10: "https://ipfs.io/ipfs/QmVdjCRYhQSo8MQzAviNqotPu5PA7EXt75JQRcfgKZSSHT",
}

TIERS = {
    1: {"name": "Nebula Prime", "score_top": 5, "reward_share": 30.0},
    2: {"name": "Supernova", "score_top": 10, "reward_share": 20.0},
    3: {"name": "Quasar", "score_top": 20, "reward_share": 15.0},
    4: {"name": "Pulsar", "score_top": 30, "reward_share": 9.5},
    5: {"name": "Stellar", "score_top": 45, "reward_share": 8.5},
    6: {"name": "Orbit", "score_top": 60, "reward_share": 5.0},
    7: {"name": "Satellite", "score_top": 80, "reward_share": 5.0},
    8: {"name": "Drift", "score_top": 90, "reward_share": 5.0},
    9: {"name": "Void", "score_top": 99, "reward_share": 2.0},
    10: {"name": "Deadzone", "score_top": 100, "reward_share": 0.0},
}

class VerifyPostRequest(BaseModel):
    handle: str
    post_url: str
    account_type: str 

# Helper: Load Admin Keypair
def load_admin_keypair() -> Optional[Keypair]:
    try:
        with open(ADMIN_KEYPAIR_PATH, 'r') as f:
            data = json.load(f)
            return Keypair.from_bytes(data)
    except FileNotFoundError:
        print(f"Admin keypair not found at {ADMIN_KEYPAIR_PATH}")
        return None

admin_kp = load_admin_keypair()
solana_client = AsyncClient(RPC_URL)

def get_sighash(namespace: str, name: str) -> bytes:
    preimage = f"{namespace}:{name}".encode()
    return hashlib.sha256(preimage).digest()[:8]

# Helper: Find PDA
def find_global_state_pda():
    return Pubkey.find_program_address([b"global_state"], PROGRAM_ID)[0]

def find_identity_pda(handle: str):
    return Pubkey.find_program_address([b"identity", handle.encode()], PROGRAM_ID)[0]

async def send_update_tx(handle: str, score: int, tier: int, uri: str):
    if not admin_kp:
        print("Skipping on-chain update: Admin keypair missing")
        return

    global_state = find_global_state_pda()
    identity = find_identity_pda(handle)
    
    # Construct Instruction Data
    # update_agent_status args: new_score: u64, tier: u8, new_uri: String
    # Discriminator (8) + u64 (8) + u8 (1) + String (4 + len)
    discriminator = get_sighash("global", "update_agent_status")
    
    uri_bytes = uri.encode("utf-8")
    data = discriminator + struct.pack("<Q", score) + struct.pack("<B", tier) + struct.pack("<I", len(uri_bytes)) + uri_bytes
    
    ix = Instruction(
        PROGRAM_ID,
        data,
        [
            AccountMeta(pubkey=global_state, is_signer=False, is_writable=False),
            AccountMeta(pubkey=identity, is_signer=False, is_writable=True),
            AccountMeta(pubkey=admin_kp.pubkey(), is_signer=True, is_writable=False),
        ]
    )
    
    try:
        recent_blockhash = await solana_client.get_latest_blockhash()
        msg = Message.new_with_blockhash(
            [ix],
            admin_kp.pubkey(),
            recent_blockhash.value.blockhash
        )
        tx = Transaction.new_unsigned(msg)
        tx.sign([admin_kp], recent_blockhash.value.blockhash)
        
        resp = await solana_client.send_transaction(tx)
        print(f"On-chain update for {handle}: {resp.value}")
    except Exception as e:
        print(f"Failed to send transaction for {handle}: {str(e)}")

@app.get("/")
async def root():
    return {
        "message": "Nebulon Verification & Analytics Backend",
        "program_id": str(PROGRAM_ID),
        "admin_loaded": admin_kp is not None
    }

@app.get("/tiers")
async def get_tiers():
    return TIERS

@app.post("/verify-linking")
async def verify_linking(request: VerifyPostRequest):
    if request.account_type not in ["moltbook", "moltx", "twitter", "x"]:
        raise HTTPException(status_code=400, detail="Invalid account type.")

    # Logic to fetch URL
    headers = {"User-Agent": "NebulonBot/1.0"}
    async with httpx.AsyncClient() as client:
        try:
            response = await client.get(request.post_url, headers=headers, follow_redirects=True)
            if response.status_code != 200:
                raise HTTPException(status_code=404, detail="Could not access the post.")
            
            # Simple text extraction
            page_text = BeautifulSoup(response.text, 'html.parser').get_text()
        except Exception as e:
            raise HTTPException(status_code=500, detail=f"Fetch error: {str(e)}")

    verification_pattern = f"NEBULON-LINK-{request.handle}"
    
    if verification_pattern in page_text:
        # In a real scenario, we might issue an on-chain verification here via update_sns
        # For now, we return success and let the client handle the on-chain submission or admin does it later
        return {
            "status": "success",
            "message": f"Verified {request.account_type}! Handle {request.handle} linked.",
            "handle": request.handle
        }
    else:
        raise HTTPException(status_code=401, detail=f"Verification pattern '{verification_pattern}' not found in the post.")

@app.post("/calculate-tiers")
async def calculate_tiers(agent_scores: Dict[str, int], background_tasks: BackgroundTasks):
    """
    Receives a map of {handle: score}.
    Calculates tiers based on percentile.
    Triggers background on-chain updates.
    """
    if not agent_scores:
        return {}

    sorted_agents = sorted(agent_scores.items(), key=lambda x: x[1], reverse=True)
    total_count = len(sorted_agents)
    
    results = {}
    for i, (handle, score) in enumerate(sorted_agents):
        rank_percentile = ((i + 1) / total_count) * 100
        
        assigned_tier = 10
        for tier_id, data in TIERS.items():
            if rank_percentile <= data["score_top"]:
                assigned_tier = tier_id
                break
        
        metadata_uri = TIER_METADATA_MAP[assigned_tier]
        
        # Add on-chain update task
        background_tasks.add_task(send_update_tx, handle, score, assigned_tier, metadata_uri)
        
        results[handle] = {
            "score": score,
            "rank": i + 1,
            "percentile": rank_percentile,
            "tier_id": assigned_tier,
            "tier_name": TIERS[assigned_tier]["name"],
            "metadata_uri": metadata_uri
        }
        
    return results

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
