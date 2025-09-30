import asyncio
import traceback
import dotenv
from datetime import date, timedelta

from quart import Quart, jsonify, request

from tomada import get_tomada, set_tomada
from client import GoodweClient

app = Quart(__name__)

dotenv.load_dotenv()

@app.get('/api/assistente')
async def dados_assistente():
    try:
        client = await GoodweClient.create("eu")

        eday, emonth = await client.eday_emonth()
        bat = await client.cur_bat()

        dia, semana, mes = await client.report_economia_consumo(0, 6, 6)
        cons_dia, econ_dia = dia
        cons_semana, econ_semana = semana
        cons_mes, econ_mes = mes
        await client.close()

        return jsonify({
            "consumo_diario_kwh": cons_dia,
            "consumo_semanal_kwh": cons_semana,
            "consumo_mensal_kwh": cons_mes,
            "economia_diaria_kwh": econ_dia,
            "economia_semanal_kwh": econ_semana,
            "economia_mensal_kwh": econ_mes,
            "prod_diaria_kwh": eday,
            "prod_mensal_kwh": emonth,
            "bateria": bat,
        })
    except Exception as e:
        traceback.print_exc()
        return jsonify({"erro": str(e)}), 500

@app.get('/api/dados/bateria_agora')
async def bateria_agora():
    try:
        client = await GoodweClient.create("eu")
        bat = await client.cur_bat()
        await client.close()
        return jsonify({"data": bat})
    except Exception as e:
        traceback.print_exc()
        return jsonify({"erro": str(e)}), 500

@app.get('/api/dados/consumo_agora')
async def consumo_agora():
    try:
        client = await GoodweClient.create("eu")
        pv, bat, meter, load, charge = await client.plant_data(client.cur_time())
        await client.close()
        return jsonify({"data": load[-1][1]})
    except Exception as e:
        traceback.print_exc()
        return jsonify({"erro": str(e)}), 500

@app.get('/api/dados/producao_agora')
async def producao_agora():
    try:
        client = await GoodweClient.create("eu")
        gen = await client.cur_gen()
        await client.close()
        return jsonify({"data": gen})
    except Exception as e:
        traceback.print_exc()
        return jsonify({"erro": str(e)}), 500

@app.get('/api/graficos/econ_semana')
async def econ_semana():
    DIAS = [
        "segunda-feira",
        "terça-feira",
        "quarta-feira",
        "quinta-feira",
        "sexta-feira",
        "sábado",
        "domingo"
    ]
    try:
        client = await GoodweClient.create("eu")
        def days_gen():
                delta = timedelta(6)
                now = client.cur_time()
                today = date(now.year, now.month, now.day)
                cur = today - delta
                while cur <= today:
                    yield cur
                    cur += timedelta(days=1)
        tasks = []
        for day in days_gen():
            tasks.append(asyncio.create_task(client.day_econ(day)))

        days = days_gen()
        data = list(map(lambda d: (DIAS[next(days).weekday()], d), await asyncio.gather(*tasks)))
        await client.close()
        return jsonify(data)

    except Exception as e:
        traceback.print_exc()
        return jsonify({"erro": str(e)}), 500

@app.get('/api/dados')
async def dados():
    try:
        client = await GoodweClient.create("eu")
        pv, bat, meter, load, charge = await client.plant_data(client.cur_time())
        def to_ser(x):
            return list(map(lambda d: (d[0].isoformat(), d[1]), x))
        await client.close()
        return jsonify({
            "pv": to_ser(pv),
            "bat": to_ser(bat),
            "meter": to_ser(meter),
            "load": to_ser(load),
            "charge": to_ser(charge),
        })
    except Exception as e:
        traceback.print_exc()
        return jsonify({"erro": str(e)}), 500

status_tomada = {
    "economia": True,
    "ligada": None
}

@app.post("/api/tomada/set_economia")
async def set_economia():
    state = request.args.get("state", "").lower()
    if state not in ["on", "off"]:
        return jsonify({"erro": f"\"{state}\" não é um estado válido"}), 400
    setstate = state == "on"
    try:
        status_tomada["economia"] = setstate
        status_tomada["ligada"] = None
        return jsonify({}), 200
    except Exception as e:
        traceback.print_exc()
        return jsonify({"erro": str(e)}), 500

@app.get("/api/tomada/get_economia")
async def get_economia():
    return jsonify({"state": "on" if status_tomada["economia"] else "off"})

@app.post("/api/tomada/set")
async def tomada_set():
    state = request.args.get("state", "").lower()
    if state not in ["on", "off"]:
        return jsonify({"erro": f"\"{state}\" não é um estado válido"}), 400
    setstate = state == "on"
    try:
        d, status = await set_tomada(setstate)
        status_tomada["economia"] = False
        status_tomada["ligada"] = setstate
        return jsonify(d), status
    except Exception as e:
        traceback.print_exc()
        return jsonify({"erro": str(e)}), 500

@app.get("/api/tomada/get")
async def tomada_get():
    try:
        d, status = await get_tomada()
        return jsonify(d), status
    except Exception as e:
        traceback.print_exc()
        return jsonify({"erro": str(e)}), 500

if __name__ == '__main__':
    app.run(debug=True)
