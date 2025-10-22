import asyncio
from datetime import datetime, date, timedelta
from typing import Iterable, Tuple

from parser import carregar_dados
from modelo_IA.IA_treinada import calcular_economia, deve_desligar

class GoodweClient:
    TIMEZONE = None

    def __init__(self, df):
        self.df = df

    @classmethod
    async def create(cls, region: str):  
        df = carregar_dados()
        return cls(df)

    async def close(self):
        pass 

    def cur_time(self) -> datetime:
        return self.df["Time"].max()

    async def cur_bat(self) -> int:
        return int(self.df["SOC(%)"].iloc[-1])

    async def cur_gen(self) -> float:
        return float(self.df["PV(W)"].iloc[-1])

    async def cur_load(self) -> float:
        return float(self.df["Load(W)"].iloc[-1])

    async def eday_emonth(self) -> Tuple[float, float]:
        hoje = self.cur_time().date()
        df_hoje = self.df[self.df["Time"].dt.date == hoje]
        eday = (df_hoje["PV(W)"] * (5 / 60) / 1000).sum()

        df_mes = self.df[self.df["Time"].dt.month == hoje.month]
        emonth = (df_mes["PV(W)"] * (5 / 60) / 1000).sum()

        return round(eday, 2), round(emonth, 2)

    async def plant_data(self, dia: date):
        df = self.df[self.df["Time"].dt.date == dia]
        if df.empty:
            return [], [], [], [], []

        return (
            list(zip(df["Time"], df["PV(W)"])),
            list(zip(df["Time"], df["Battery(W)"])),
            list(zip(df["Time"], df["Grid(W)"])),
            list(zip(df["Time"], df["Load(W)"])),
            list(zip(df["Time"], df["SOC(%)"]))
        )

    async def day_cons(self, dia: date) -> float:
        _, _, _, load, _ = await self.plant_data(dia)
        return self.__cons_from_day_load(load)

    async def day_econ(self, dia: date) -> float:
        _, _, _, load, _ = await self.plant_data(dia)
        return self.__econ_from_day_load(load)

    async def consumo_periodo(self, days: int) -> float:
        return await self.__periodo_total(days, self.day_cons)

    async def economia_periodo(self, days: int) -> float:
        return await self.__periodo_total(days, self.day_econ)

    async def report_economia_consumo(self, *dias_atras: int) -> list[tuple[float, float]]:
        hoje = self.cur_time().date()
        resultados = []

        for cutoff in dias_atras:
            cons_total = 0.0
            econ_total = 0.0
            for i in range(cutoff + 1):
                dia = hoje - timedelta(days=i)
                cons_total += await self.day_cons(dia)
                econ_total += await self.day_econ(dia)
            resultados.append((round(cons_total, 2), round(econ_total, 2)))

        return resultados

    @staticmethod
    def __cons_from_day_load(data: Iterable[tuple[datetime, float]]) -> float:
        return sum((v * (5 / 60)) / 1000 for _, v in data)

    @staticmethod
    def __econ_from_day_load(data: Iterable[tuple[datetime, float]]) -> float:
        return sum(
            calcular_economia(v, deve_desligar(t.hour, v)) * (5 / 60) / 1000
            for t, v in data
        )

    async def __periodo_total(self, days: int, func) -> float:
        hoje = self.cur_time().date()
        tasks = [asyncio.create_task(func(hoje - timedelta(days=i))) for i in range(days + 1)]
        return sum(await asyncio.gather(*tasks))