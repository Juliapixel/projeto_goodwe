import json
import base64
import requests

BASE = {
    "us": "https://us.semsportal.com",
    "eu": "https://eu.semsportal.com",
}

def _initial_token() -> str:
    original = {
        "uid": "",
        "timestamp": 0,
        "token": "",
        "client": "web",
        "version": "",
        "language": "en"
    }
    b = json.dumps(original).encode("utf-8")
    return base64.b64encode(b).decode("utf-8")

def crosslogin(account: str, password: str, region: str = "us") -> str:
    url = f"{BASE[region]}/api/v2/Common/CrossLogin"
    headers = {
        "Token": _initial_token(),
        "Content-Type": "application/json",
        "Accept": "*/*"
    }
    payload = {
        "account": account,
        "pwd": password,
        "agreement_agreement": 0,
        "is_local": False
    }
    response = requests.post(url, json=payload, headers=headers, timeout=20)
    response.raise_for_status()
    js = response.json()
    if "data" not in js or str(js.get("code")) not in ("0", "1", "200"):
        raise RuntimeError(f"Login falhou: {js}")
    data_to_string = json.dumps(js["data"])
    token = base64.b64encode(data_to_string.encode("utf-8")).decode("utf-8")
    return token

def get_inverter_data_by_column(token: str, inverter_sn: str, column: str, date_str: str, region: str = "eu"):
    url = f"{BASE[region]}/api/PowerStationMonitor/GetInverterDataByColumn"
    headers = {
        "Token": token,
        "Content-Type": "application/json",
        "Accept": "*/*"
    }
    payload = {
        "date": date_str,
        "column": column,
        "id": inverter_sn
    }
    response = requests.post(url, json=payload, headers=headers, timeout=20)
    response.raise_for_status()
    return response.json()


def get_power_station_list(token: str, region: str = "eu"):
    url = f"{BASE[region]}/api/v2/PowerStation/GetPowerStationList"
    headers = {
        "Token": token,
        "Content-Type": "application/json",
        "Accept": "*/*"
    }
    response = requests.post(url, json={}, headers=headers, timeout=20)
    response.raise_for_status()
    return response.json()
