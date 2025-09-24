from datetime import datetime, timedelta

def gerar_datas_desde_instalacao(data_instalacao: str, limite_dias: int = 30) -> list[str]:
    inicio = datetime.now() - timedelta(days=limite_dias)
    hoje = datetime.now()
    datas = []

    while inicio <= hoje:
        datas.append(inicio.strftime("%Y-%m-%d 00:00:00"))
        inicio += timedelta(days=1)

    return datas

def tratar_dados_energia(dados_eday: dict, dados_bateria: dict) -> dict:
    try:
        registros_energia = dados_eday.get("data", {}).get("column1", [])
        registros_bateria = dados_bateria.get("data", {}).get("column1", [])

        hoje = datetime.now()
        energia_diaria = 0
        energia_semanal = 0
        energia_mensal = 0
        energia_anual = 0

        for item in registros_energia:
            valor = float(item.get("column", 0))
            data_str = item.get("date")
            data = datetime.strptime(data_str, "%m/%d/%Y %H:%M:%S")

            if data.date() == hoje.date():
                energia_diaria += valor
            if (hoje - data).days <= 7:
                energia_semanal += valor
            if data.month == hoje.month and data.year == hoje.year:
                energia_mensal += valor
            if data.year == hoje.year:
                energia_anual += valor

        bateria_hoje = None
        for item in reversed(registros_bateria):
            data = datetime.strptime(item["date"], "%m/%d/%Y %H:%M:%S")
            valor = float(item["column"])
            if data.date() == hoje.date() and data <= hoje and valor > 0:
                bateria_hoje = valor   

        return {
            "energia_diaria_kwh": round(energia_diaria, 2),
            "energia_semanal_kwh": round(energia_semanal, 2),
            "energia_mensal_kwh": round(energia_mensal, 2),
            "energia_anual_kwh": round(energia_anual, 2),
            "bateria_percentual": min(bateria_hoje, 100) if bateria_hoje is not None else None
        }

    except Exception as e:
        return {"erro_tratamento": str(e)}
