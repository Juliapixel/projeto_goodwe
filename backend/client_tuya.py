import os
from dotenv import load_dotenv
from tuya_connector import TuyaOpenAPI

load_dotenv()

TUYA_CLIENT_ID = "eqcskt5np78gxnyrcrmu"
TUYA_CLIENT_SECRET = "b261a73b1d7941288b73f0ef3ab22422"
TUYA_REGION = "us"
TUYA_DEVICE_ID = "eb9efd0da2bd405863ks0k"

api = TuyaOpenAPI(f"https://openapi.tuya{TUYA_REGION}.com", TUYA_CLIENT_ID, TUYA_CLIENT_SECRET)
api.connect()

print("Token info:", api.token_info)
print("Token info:", {
    "access_token": api.token_info.access_token,
    "expire_time": api.token_info.expire_time,
    "refresh_token": api.token_info.refresh_token
})

def get_status(device_id: str) -> bool:
    """
    Verifica se a tomada está ligada ou desligada.

    Args:
        device_id: ID do dispositivo Tuya

    Returns:
        bool: True se ligada, False se desligada
    """
    try:
        endpoint = f"/v1.0/devices/{device_id}/status"
        result = api.get(endpoint)

        if not result.get("success"):
            print(f"Erro ao obter status: {result.get('msg', 'Unknown')}")
            return False

        for item in result["result"]:
            if item["code"] in ["switch_1", "switch"]:
                return bool(item["value"])

        print("Campo de status não encontrado.")
        return False

    except Exception as e:
        print(f"Falha ao obter status: {e}")
        return False

def get_device_current_power(device_id: str, force_refresh: bool = True) -> float:
    try:
        # Adiciona timestamp para evitar cache
        endpoint = f"/v1.0/devices/{device_id}/status"

        if force_refresh:
            import time
            endpoint += f"?_t={int(time.time() * 1000)}"

        result = api.get(endpoint)

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
        print(f"Falha ao obter potência: {e}")
        return 0.0

def control_switch(device_id: str, turn_on: bool) -> bool:
    """
    Controla a tomada.

    Args:
        device_id: ID do dispositivo
        turn_on: True para ligar, False para desligar

    Returns:
        bool: True se sucesso
    """
    try:
        result = api.post(
            f"/v1.0/devices/{device_id}/commands",
            {"commands": [{"code": "switch_1", "value": turn_on}]}
        )
        return result.get("success", False)
    except:
        return False


if __name__ == "__main__":
    if not TUYA_DEVICE_ID:
        print("Defina TUYA_DEVICE_ID no .env para testar.")
    else:
        status = get_status(TUYA_DEVICE_ID)
        if status:
            print("LIGADA")
        else:
            print("DESLIGADA")

        power = get_device_current_power(TUYA_DEVICE_ID)
        print(f"Potência atual do dispositivo {TUYA_DEVICE_ID}: {power} W")

