import os
from dotenv import load_dotenv
from tuya_iot import TuyaOpenAPI

load_dotenv()

TUYA_CLIENT_ID = os.getenv("TUYA_CLIENT_ID")
TUYA_CLIENT_SECRET = os.getenv("TUYA_CLIENT_SECRET")
TUYA_REGION = os.getenv("TUYA_REGION", "us")
TUYA_DEVICE_ID = os.getenv("TUYA_DEVICE_ID")

api = TuyaOpenAPI(f"https://openapi.tuya{TUYA_REGION}.com", TUYA_CLIENT_ID, TUYA_CLIENT_SECRET)
api.connect()

print("Token info:", api.token_info)

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

