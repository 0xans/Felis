import os
import json
import hashlib
from cryptography.hazmat.primitives.ciphers.aead import AESGCM

secret = b"password"
derived_key = hashlib.sha256(secret).digest()

data = {
    "session_id": "dummy-session",
    "hostname": "dummy",
    "username": "yeet",
    "os": "Windows",
    "pid": 1337,
    "process": "dummy.exe",
    "arch": "x64",
    "integrity": "High",
    "timestamp": 1700000000,
    "metadata": {}
}
plaintext = json.dumps(data).encode('utf-8')

aesgcm = AESGCM(derived_key)
nonce = os.urandom(12)
ciphertext = aesgcm.encrypt(nonce, plaintext, None)

payload = nonce + ciphertext

with open("payload.bin", "wb") as f:
    f.write(payload)

print("Encrypted payload saved to payload.bin!")