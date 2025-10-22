import os
import json
from dotenv import load_dotenv

load_dotenv()

TUYA_CLIENT_ID = os.getenv("TUYA_CLIENT_ID")
TUYA_CLIENT_SECRET = os.getenv("TUYA_CLIENT_SECRET")
TUYA_REGION = os.getenv("TUYA_REGION", "us")
TUYA_DEVICE_ID = os.getenv("TUYA_DEVICE_ID")

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

def get_device_current_power(device_id: str) -> float:
    try:
        result = api.get(f"/v1.0/devices/{device_id}/status")
        if not result.get("success"):
            raise ValueError(f"Erro na API Tuya: {result}")
        for item in result["result"]:
            if item["code"] in ["cur_power", "current_power", "power"]:
                val = float(item["value"])
                if val > 10000:
                    val /= 10
                return val
        raise KeyError("Campo de potência não encontrado.")
    except Exception as e:
        print(f"Falha ao obter potência do dispositivo {device_id}: {e}")
        return 0.0

if __name__ == "__main__":
    if not TUYA_DEVICE_ID:
        print("Defina TUYA_DEVICE_ID no .env para testar.")
    else:
        power = get_device_current_power(TUYA_DEVICE_ID)
        print(f"Potência atual do dispositivo {TUYA_DEVICE_ID}: {power} W")



