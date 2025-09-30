from datetime import datetime
from typing import Tuple

def tratar_dados_energia(dados_eday: list[Tuple[datetime, float]], dados_bateria: list[Tuple[datetime, float]]) -> dict:
    try:
        hoje = datetime.now()
        energia_diaria = 0
        energia_semanal = 0
        energia_mensal = 0
        energia_anual = 0

        for data, valor in dados_eday:
            if data.date() == hoje.date():
                energia_diaria += valor
            if (hoje - data).days <= 7:
                energia_semanal += valor
            if data.month == hoje.month and data.year == hoje.year:
                energia_mensal += valor
            if data.year == hoje.year:
                energia_anual += valor

        print(f"ultima bat: {dados_bateria[-1]}")
        bateria_hoje = dados_bateria[-1][1]

        return {
            "energia_diaria_kwh": round(energia_diaria, 2),
            "energia_semanal_kwh": round(energia_semanal, 2),
            "energia_mensal_kwh": round(energia_mensal, 2),
            "energia_anual_kwh": round(energia_anual, 2),
            "bateria_percentual": min(bateria_hoje, 100) if bateria_hoje is not None else None
        }

    except Exception as e:
        return {"erro_tratamento": str(e)}
