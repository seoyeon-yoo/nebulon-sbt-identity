import os
import httpx
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from typing import Dict, List, Optional
from bs4 import BeautifulSoup
from dotenv import load_dotenv

load_dotenv()

app = FastAPI(title="Nebulon SBT Identity Backend")

# Simulation of on-chain state
PROGRAM_ID = "AVPj6DchcE2yZQPidaYqt2MoyNx3TyH1BpRyB9E1TW7h"

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

class VerifyPostRequest(BaseModel):
    handle: str
    post_url: str
    account_type: str 

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

@app.get("/")
async def root():
    return {"message": "Nebulon Verification & Analytics Backend is running"}

@app.get("/tiers")
async def get_tiers():
    return TIERS

@app.post("/verify-linking")
async def verify_linking(request: VerifyPostRequest):
    if request.account_type not in ["moltbook", "moltx"]:
        raise HTTPException(status_code=400, detail="Invalid account type.")

    async with httpx.AsyncClient() as client:
        try:
            response = await client.get(request.post_url, follow_redirects=True)
            if response.status_code != 200:
                raise HTTPException(status_code=404, detail="Could not access the post.")
            
            page_text = BeautifulSoup(response.text, 'html.parser').get_text()
        except Exception as e:
            raise HTTPException(status_code=500, detail=f"Fetch error: {str(e)}")

    verification_pattern = f"NEBULON-LINK-{request.handle}"
    
    if verification_pattern in page_text:
        # Simulate on-chain call
        print(f"ON-CHAIN ACTION: Link {request.account_type} to {request.handle}")
        return {
            "status": "success",
            "message": f"Verified {request.account_type}! 5 points granted on-chain.",
            "handle": request.handle
        }
    else:
        raise HTTPException(status_code=401, detail="Verification pattern not found.")

@app.post("/calculate-tiers")
async def calculate_tiers(agent_scores: Dict[str, int]):
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
        
        # New Feature: Link Metadata URI to Tier
        metadata_uri = TIER_METADATA_MAP[assigned_tier]
        
        print(f"TRIGGER: Updating Agent {handle} to Tier {assigned_tier} with URI {metadata_uri}")
        # Logic to call Anchor program: update_agent_status(score, assigned_tier, metadata_uri)
        
        results[handle] = {
            "score": score,
            "rank": i + 1,
            "percentile": rank_percentile,
            "tier_id": assigned_tier,
            "tier_name": TIERS[assigned_tier]["name"],
            "reward_share": TIERS[assigned_tier]["reward_share"],
            "metadata_uri": metadata_uri
        }
        
    return results

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
