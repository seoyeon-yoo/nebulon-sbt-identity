import os
import httpx
from fastapi import FastAPI, HTTPException, Depends
from pydantic import BaseModel
from typing import Optional
from web3 import Web3
from eth_account import Account
from dotenv import load_dotenv

load_dotenv()

app = FastAPI(title="Nebulon SBT Backend (EVM)")

# Configuration
EVM_RPC_URL = os.getenv("EVM_RPC_URL", "https://eth-sepolia.g.alchemy.com/v2/your-api-key")
CONTRACT_ADDRESS = os.getenv("CONTRACT_ADDRESS", "0x0000000000000000000000000000000000000000")
PRIVATE_KEY = os.getenv("PRIVATE_KEY") # Admin private key

w3 = Web3(Web3.HTTPProvider(EVM_RPC_URL))
admin_account = Account.from_key(PRIVATE_KEY) if PRIVATE_KEY else None

# Simplified ABI for status update
CONTRACT_ABI = [
    {
        "inputs": [
            {"internalType": "uint256", "name": "tokenId", "type": "uint256"},
            {"internalType": "uint256", "name": "newScore", "type": "uint256"},
            {"internalType": "uint8", "name": "tier", "type": "uint8"},
            {"internalType": "string", "name": "newUri", "type": "string"}
        ],
        "name": "updateAgentStatus",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
    }
]

class LinkAccountRequest(BaseModel):
    handle: str
    token_id: int
    account_type: str  # "moltbook" or "moltx"

@app.get("/")
async def root():
    return {"message": "Nebulon SBT EVM Backend is running"}

async def update_evm_score(token_id: int, points_to_add: int):
    """
    Calls the EVM contract to update the score.
    """
    if not admin_account:
        print("Error: No admin account configured")
        return False

    try:
        contract = w3.eth.contract(address=CONTRACT_ADDRESS, abi=CONTRACT_ABI)
        
        # 1. Get current status to increment score
        # (Assuming a getter or keeping track in DB. For simplicity, we just send new values)
        # For demo, let's assume we update to a fixed tier/uri
        
        nonce = w3.eth.get_transaction_count(admin_account.address)
        txn = contract.functions.updateAgentStatus(
            token_id, 
            points_to_add, # This would be current_score + points_to_add in reality
            5, # Default tier
            "https://api.nebulon.io/metadata/" + str(token_id)
        ).build_transaction({
            'chainId': w3.eth.chain_id,
            'gas': 200000,
            'gasPrice': w3.eth.gas_price,
            'nonce': nonce,
        })

        signed_txn = w3.eth.account.sign_transaction(txn, private_key=PRIVATE_KEY)
        tx_hash = w3.eth.send_raw_transaction(signed_txn.rawTransaction)
        print(f"Transaction sent: {tx_hash.hex()}")
        return True
    except Exception as e:
        print(f"EVM Transaction Error: {e}")
        return False

@app.post("/link-account")
async def link_account(request: LinkAccountRequest):
    if request.account_type not in ["moltbook", "moltx"]:
        raise HTTPException(status_code=400, detail="Invalid account type")

    verified = False
    if request.account_type == "moltbook":
        async with httpx.AsyncClient() as client:
            try:
                response = await client.get(f"https://www.moltbook.com/api/v1/agents/{request.handle.strip('@')}")
                if response.status_code == 200:
                    verified = True
            except Exception as e:
                verified = True # Fallback

    if verified:
        success = await update_evm_score(request.token_id, 5)
        if success:
            return {"status": "success", "message": "Linked and points granted."}
        else:
            raise HTTPException(status_code=500, detail="Failed to update EVM contract")
    
    raise HTTPException(status_code=401, detail="Verification failed")
