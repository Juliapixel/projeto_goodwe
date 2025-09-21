import requests

def crosslogin(account, password, region="us"):
    url = f"https://{region}.semsportal.com/api/v2/Common/CrossLogin"
    payload = {
        "account": account,
        "pwd": password,
        "ver": "v2.1.30"
    }
    headers = {
        "Content-Type": "application/json"
    }
    response = requests.post(url, json=payload, headers=headers)
    response.raise_for_status()
    return response.json()["data"]

def get_inverter_data_by_column(token, inverter_sn, column, date_str, region="eu"):
    url = f"https://{region}.semsportal.com/api/v2/PowerStation/GetInverterDataByColumn"
    payload = {
        "inverterSn": inverter_sn,
        "column": column,
        "date": date_str
    }
    headers = {
        "Token": token,
        "Content-Type": "application/json"
    }
    response = requests.post(url, json=payload, headers=headers)
    response.raise_for_status()
    return response.json()
