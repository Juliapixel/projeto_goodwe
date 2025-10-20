import os
import json
from dotenv import load_dotenv

load_dotenv()

TUYA_CLIENT_ID = os.getenv("TUYA_CLIENT_ID")
TUYA_CLIENT_SECRET = os.getenv("TUYA_CLIENT_SECRET")
TUYA_REGION = os.getenv("TUYA_REGION", "us")
#TUYA_PROJECT_ID = os.getenv("TUYA_PROJECT_ID", "")  # opcional: id do projeto na Tuya IoT Console

REGION_BASES = {
    "eu": "https://openapi.tuyaeu.com",
    "us": "https://openapi.tuyaus.com",
    "cn": "https://openapi.tuyacn.com",
    "asia": "https://openapi.tuyacn.com",
}

base_url = REGION_BASES.get(TUYA_REGION, REGION_BASES["eu"])

try:
    from tuya_iot import TuyaOpenAPI
except Exception:
    print("SDK tuya-iot-py-sdk não encontrado. Rode: pip install tuya-iot-py-sdk")
    raise SystemExit(1)

api = TuyaOpenAPI(base_url, TUYA_CLIENT_ID, TUYA_CLIENT_SECRET)

try:
    api.connect()
    print("Conexão com Tuya OK — credenciais/region aceitas.")
except Exception as e:
    print("Falha ao conectar à Tuya:", e)
    raise SystemExit(1)

