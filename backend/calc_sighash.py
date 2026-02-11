import hashlib

def get_sighash(namespace: str, name: str) -> bytes:
    preimage = f"{namespace}:{name}".encode()
    return hashlib.sha256(preimage).digest()[:8]

print(f"update_agent_status: {get_sighash('global', 'update_agent_status').hex()}")
print(f"calculate_tiers (not on chain but useful): {get_sighash('global', 'calculate_tiers').hex()}")
