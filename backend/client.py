import asyncio
from datetime import date, datetime, time, timedelta, timezone
import json
import base64
from typing import Literal, Tuple
import aiohttp

class GoodweClient():
    # sim tudo hardcoded fodase
    ACCOUNT = "demo@goodwe.com"
    PASSWORD = "GoodweSems123!@#"
    INVERTER_SN = "5010KETU229W6177"
    POWER_STATION_ID = "6ef62eb2-7959-4c49-ad0a-0ce75565023a"
    TIMEZONE = timezone(timedelta(hours=2))

    client: aiohttp.ClientSession
    token: str

    @classmethod
    async def create(cls, region: Literal["eu", "us"]):
        self = cls()
        self.client = aiohttp.ClientSession(f"https://{region}.semsportal.com")
        self.token = await self.__crosslogin()
        return self

    async def close(self):
        await self.client.close()

    async def refresh_token(self):
        self.token = await self.__crosslogin()

    def default_headers(self):
        return {
            "Token": self.token,
            "Content-Type": "application/json",
            "Accept": "*/*"
        }

    async def inverter_data_by_column(self, date: date, column: Literal["Eday", "Pac", "Cbattery1"]) -> list[Tuple[datetime, float]]:
        response = await self.client.post(
            "/api/PowerStationMonitor/GetInverterDataByColumn",
            json={
                "date": f"{date.strftime("%m/%d/%Y")} 00:00:00",
                "column": column,
                "id": self.INVERTER_SN
            },
            headers=self.default_headers()
        )
        response.raise_for_status()
        data = (await response.json())["data"]["column1"]
        datapoints = list(map(lambda d: (datetime.strptime(d["date"], "%m/%d/%Y %H:%M:%S").astimezone(self.TIMEZONE), d["column"]), data))
        return datapoints

    async def consumo_periodo(self, days: int) -> float:
        """retorna o consumo em kWh dos ultimos n dias"""
        def days_gen():
            delta = timedelta(max(days - 1, 0))
            now = self.cur_time()
            today = date(now.year, now.month, now.day)
            cur = today - delta
            while cur <= today:
                yield cur
                cur += timedelta(days=1)

        tasks = []
        for day in days_gen():
            tasks.append(asyncio.create_task(self.day_cons(day)))

        results = await asyncio.gather(*tasks)
        return sum(results)

    def cur_time(self) -> datetime:
        now = datetime.now(self.TIMEZONE)
        return now

    async def cur_bat(self) -> int:
        """retorna porcentagem de bateria restante"""
        return (await self.__latest_points())[0]

    async def cur_gen(self) -> float:
        """retorna geração fotovoltáica atual"""
        return (await self.__latest_points())[1]

    async def cur_load(self) -> float:
        """retorna consumo atual em W"""
        return (await self.plant_data(self.cur_time()))[3][-1][1]

    async def eday_emonth(self):
        bat, gen, eday, emonth = await self.__latest_points()
        return eday, emonth

    async def __latest_points(self):
        """
        retorna:
        1. carga da bateria em %
        2. geração em W
        3. geração do dia em kWh
        3. geração do mes em kWh
        """
        resp = await self.client.post(
            "/api/v3/PowerStation/GetInverterAllPoint",
            json={
                "powerStationId": self.POWER_STATION_ID
            },
            headers=self.default_headers()
        )
        data: dict = (await resp.json())["data"]["inverterPoints"][0]
        bat = int(data["soc"][:-1])
        gen = float(data["out_pac"])
        eday = float(data["eday"])
        emonth = float(data["emonth"])
        return (bat, gen, eday, emonth)

    async def plant_data(self, date: date):
        """
        retorna:
        1. geração pv
        2. potência na bateria (negativo carga, positivo descarga)
        3. potência da concessionária
        4. potência da casa
        5. carga da bateria
        """
        resp = await self.client.post(
            "/api/v2/Charts/GetPlantPowerChart",
            json={
                "date": f"{date.strftime("%m/%d/%Y")} 00:00:00",
                "full_script": False,
                "id": self.POWER_STATION_ID
            },
            headers=self.default_headers()
        )

        data: list[dict] = (await resp.json())["data"]["lines"]
        lines = dict(map(lambda d: (d["key"], d["xy"]), data))

        def xy_parse(x: str, y: float):
            parts = list(map(int, x.split(":")))
            t = time(parts[0], parts[1])
            return (
                datetime.combine(date, t, self.TIMEZONE),
                y
            )

        pv = list(map(lambda d: (xy_parse(d["x"], d["y"])), lines["PCurve_Power_PV"]))
        bat = list(map(lambda d: (xy_parse(d["x"], d["y"])), lines["PCurve_Power_Battery"]))
        meter = list(map(lambda d: (xy_parse(d["x"], d["y"])), lines["PCurve_Power_Meter"]))
        load = list(map(lambda d: (xy_parse(d["x"], d["y"])), lines["PCurve_Power_Load"]))
        charge = list(map(lambda d: (xy_parse(d["x"], d["y"])), lines["PCurve_Power_SOC"]))
        return pv, bat, meter, load, charge

    async def day_cons(self, date: date) -> float:
        """consumo do dia em kWh"""
        pv, bat, meter, load, charge = await self.plant_data(date)
        return sum(map(lambda c: (c[1] * (5 / 60)) / 1000, load))

    async def __crosslogin(self):
        resp = await self.client.post(
            "/api/v2/Common/Crosslogin",
            headers={
                "Token": _initial_token(),
                "Content-Type": "application/json",
                "Accept": "*/*"
            },
            json = {
                "account": self.ACCOUNT,
                "pwd": self.PASSWORD,
                "agreement_agreement": 0,
                "is_local": False
            }
        )
        resp.raise_for_status()
        d = await resp.json()
        if "data" not in d or str(d.get("code")) not in ("0", "1", "200"):
            raise RuntimeError(f"Login falhou: {d}")
        data_to_string = json.dumps(d["data"])
        token = base64.b64encode(data_to_string.encode("utf-8")).decode("utf-8")
        return token

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
