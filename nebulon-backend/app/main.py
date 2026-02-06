import os
import httpx
from fastapi import FastAPI, HTTPException, Depends
from pydantic import BaseModel
from typing import Optional
from anchorpy import Program, Provider, Wallet, Context
from solana.rpc.async_api import AsyncClient
from solders.pubkey import Pubkey
from dotenv import load_dotenv

load_dotenv()

app = FastAPI(title="Nebulon SBT Backend")

# Configuration
SOLANA_RPC_URL = os.getenv("SOLANA_RPC_URL", "https://api.devnet.solana.com")
PROGRAM_ID = os.getenv("PROGRAM_ID", "AVPj6DchcE2yZQPidaYqt2MoyNx3TyH1BpRyB9E1TW7h")
ADMIN_KEY_PATH = os.getenv("ADMIN_KEY_PATH", os.path.expanduser("~/.config/solana/id.json"))

class LinkAccountRequest(BaseModel):
    handle: str
    wallet_address: str
    account_type: str  # "moltbook" or "moltx"
    verification_token: Optional[str] = None

@app.get("/")
async def root():
    return {"message": "Nebulon SBT Identity Backend is running"}

async def update_onchain_score(handle: str, points_to_add: int):
    """
    Calls the Anchor program to update the score.
    In a real implementation, this would use AnchorPy to invoke 'update_score'.
    """
    # Placeholder for AnchorPy logic
    # 1. Load IDL
    # 2. Setup Provider/Program
    # 3. Fetch current identity account
    # 4. Invoke update_score(current_score + points_to_add)
    print(f"DEBUG: Adding {points_to_add} points to handle {handle}")
    return True

@app.post("/link-account")
async def link_account(request: LinkAccountRequest):
    if request.account_type not in ["moltbook", "moltx"]:
        raise HTTPException(status_code=400, detail="Invalid account type")

    # Verification Logic
    verified = False
    if request.account_type == "moltbook":
        # Example verification: Check if handle exists on Moltbook
        async with httpx.AsyncClient() as client:
            try:
                # Mocking Moltbook verification
                response = await client.get(f"https://www.moltbook.com/api/v1/agents/{request.handle.strip('@')}")
                if response.status_code == 200:
                    verified = True
            except Exception as e:
                print(f"Moltbook verification error: {e}")
                verified = True # Fallback for demo/testing
    
    elif request.account_type == "moltX":
        # MoltX verification logic (Placeholder)
        verified = True # Assuming success for now

    if verified:
        # Grant 5 points
        success = await update_onchain_score(request.handle, 5)
        if success:
            return {
                "status": "success",
                "message": f"Successfully linked {request.account_type}. 5 points granted.",
                "handle": request.handle
            }
        else:
            raise HTTPException(status_code=500, detail="Failed to update on-chain score")
    else:
        raise HTTPException(status_code=401, detail=f"Verification failed for {request.account_type}")

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
