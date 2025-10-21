import asyncio
import time
import hmac
import hashlib
import base64
from datetime import datetime, timezone, timedelta
import aiohttp


class TuyaClient():
    ACCESS_ID = "7gr7nrajngaer7tnp9ds"
    ACCESS_SECRET = "ea87d02231034d22950a09e7c0df1cf9"
    REGION = "us"
    DEVICE_ID = "eb9efd0da2bd405863ks0k"
    API_ENDPOINT = f"https://openapi.tuya{REGION}.com"
    TIMEZONE = timezone(timedelta(hours=-3))

    client: aiohttp.ClientSession
    token: str

    @classmethod
    async def create(cls):
        self = cls()
        self.client = aiohttp.ClientSession(base_url=self.API_ENDPOINT)
        try:
            self.token = await self.__get_token()
        except Exception:
            await self.client.close()
            raise
        return self

    async def close(self):
        await self.client.close()

    def _calculate_sign(self, body: object = None, use_token: bool = False) -> tuple[str, str]:
        """
        Construção da string para assinatura:
        - token ainda não obtido (token endpoint): client_id + t [+ body]
        - com token: client_id + access_token + t [+ body]
        Body (quando presente) deve ser JSON sem espaços significativos.
        """
        t = str(int(time.time() * 1000))
        if use_token and getattr(self, "token", None):
            sign_str = f"{self.ACCESS_ID}{self.token}{t}"
        else:
            sign_str = f"{self.ACCESS_ID}{t}"

        if body is not None:
            # normaliza JSON para assinatura
            if not isinstance(body, str):
                body_str = json.dumps(body, separators=(",", ":"), ensure_ascii=False)
            else:
                body_str = body
            sign_str += body_str

        mac = hmac.new(self.ACCESS_SECRET.encode("utf-8"), sign_str.encode("utf-8"), hashlib.sha256).digest()
        sign = base64.b64encode(mac).decode("utf-8")
        return t, sign

    async def __get_token(self) -> str:
        t, sign = self._calculate_sign(body=None, use_token=False)
        headers = {
            "client_id": self.ACCESS_ID,
            "sign": sign,
            "sign_method": "HMAC-SHA256",
            "t": t
        }

        resp = await self.client.get("/v1.0/token", params={"grant_type": "1"}, headers=headers)
        resp.raise_for_status()
        data = await resp.json()

        if not isinstance(data, dict) or "result" not in data:
            raise RuntimeError(f"Token request failed, response: {data}")
        result = data["result"]
        if "access_token" not in result:
            raise RuntimeError(f"Token response missing access_token: {data}")
        return result["access_token"]

    async def get_current_power(self) -> float:
        """Retorna o consumo atual de energia em watts (procura por 'cur_power')."""
        t, sign = self._calculate_sign(body=None, use_token=True)
        headers = {
            "client_id": self.ACCESS_ID,
            "access_token": self.token,
            "sign": sign,
            "sign_method": "HMAC-SHA256",
            "t": t
        }

        resp = await self.client.get(f"/v1.0/devices/{self.DEVICE_ID}/status", headers=headers)
        resp.raise_for_status()
        data = await resp.json()

        for status in data.get("result", []):
            code = status.get("code") or status.get("dpCode")
            if code in ("cur_power", "power", "current_power"):
                try:
                    return float(status.get("value", 0))
                except (TypeError, ValueError):
                    return 0.0

        # debug: imprime resposta para identificar o dpCode correto
        print("cur_power não encontrado. resposta completa (parcial):")
        print(json.dumps(data, indent=2, ensure_ascii=False))
        return 0.0


async def main():
    client = await TuyaClient.create()
    try:
        power = await client.get_current_power()
        print(f"Consumo atual: {power} W")
    finally:
        await client.close()


if __name__ == "__main__":
    asyncio.run(main())