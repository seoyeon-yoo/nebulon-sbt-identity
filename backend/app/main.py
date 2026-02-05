import os
import httpx
import re
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from typing import Optional
from bs4 import BeautifulSoup
from dotenv import load_dotenv

# Note: In a real environment, we'd use anchorpy to talk to Solana.
# For now, we simulate the on-chain interaction.

load_dotenv()

app = FastAPI(title="Nebulon SBT Identity Backend")

# Simulation of on-chain state or connection
PROGRAM_ID = "AVPj6DchcE2yZQPidaYqt2MoyNx3TyH1BpRyB9E1TW7h"

class VerifyPostRequest(BaseModel):
    handle: str
    post_url: str
    account_type: str  # "moltbook" or "moltx"

@app.get("/")
async def root():
    return {"message": "Nebulon Verification Backend is running"}

async def update_onchain_linking(handle: str, platform: str, social_handle: str):
    """
    Calls the Anchor 'link_social_account' instruction.
    Adds 5 points and records the social handle on-chain.
    """
    print(f"ON-CHAIN ACTION: Linking {platform} account {social_handle} to agent {handle}")
    # anchorpy logic to invoke link_social_account(platform, social_handle)
    return True

@app.post("/verify-linking")
async def verify_linking(request: VerifyPostRequest):
    if request.account_type not in ["moltbook", "moltx"]:
        raise HTTPException(status_code=400, detail="Invalid account type. Use 'moltbook' or 'moltx'.")

    # 1. Fetch the post content
    async with httpx.AsyncClient() as client:
        try:
            response = await client.get(request.post_url, follow_redirects=True)
            if response.status_code != 200:
                raise HTTPException(status_code=404, detail="Could not access the post URL.")
            
            html_content = response.text
            soup = BeautifulSoup(html_content, 'html.parser')
            page_text = soup.get_text()
            
        except Exception as e:
            raise HTTPException(status_code=500, detail=f"Error fetching URL: {str(e)}")

    # 2. Check for the verification pattern
    # Pattern: NEBULON-LINK-[HANDLE]
    verification_pattern = f"NEBULON-LINK-{request.handle}"
    
    if verification_pattern in page_text:
        # 3. Update On-chain Record & Grant 5 points
        success = await update_onchain_linking(request.handle, request.account_type, request.handle)
        if success:
            return {
                "status": "success",
                "message": f"Successfully verified and linked {request.account_type}! On-chain handle recorded and 5 points granted.",
                "verified_handle": request.handle,
                "platform": request.account_type
            }
        else:
            raise HTTPException(status_code=500, detail="Failed to update on-chain record.")
    else:
        raise HTTPException(
            status_code=401, 
            detail=f"Verification failed. Could not find '{verification_pattern}' in the provided post."
        )

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
