import asyncio
import datetime
import traceback
import dotenv
from datetime import date, timedelta

import jwt
from quart import Quart, jsonify, redirect, request

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

        dia, semana, mes = await client.report_economia_consumo(0, 6)
        cons_dia, econ_dia = dia
        cons_semana, econ_semana = semana
        await client.close()

        return jsonify({
            "consumo_diario_kwh": cons_dia,
            "consumo_semanal_kwh": cons_semana,
            "economia_diaria_kwh": econ_dia,
            "economia_semanal_kwh": econ_semana,
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

SECRET_KEY = "oieuamoobts"

@app.get("/api/auth/token")
async def generate_token():
    data = await request.get_json()
    client_id = data.get("client_id")
    client_secret = data.get("client_secret")
    grant_type = data.get("grant_type")


    if grant_type != "client_credentials" or client_id != "tralalerotralala" or client_secret != "123456":
        return jsonify({"error": "invalid_client"}), 401

    # token JWT
    payload = {
        "sub": client_id,
        "scope": "read write",
        "exp": int((datetime.datetime.now(datetime.timezone.utc) + datetime.timedelta(days=365)).timestamp())
    }
    token = jwt.encode(payload, SECRET_KEY.encode("utf-8"))

    return jsonify({
        "access_token": token,
        "token_type": "Bearer",
        "expires_in": 60*60*24*365,
    })

# Decorador para proteger endpoints
def token_required(f):
    def wrapper(*args, **kwargs):
        auth_header = request.headers.get("Authorization")
        if not auth_header or not auth_header.startswith("Bearer "):
            return jsonify({"error": "missing_token"}), 401

        token = auth_header.split(" ")[1]
        try:
            decoded = jwt.decode(token, SECRET_KEY, algorithms=["HS256"])
            # request = decoded
        except jwt.ExpiredSignatureError:
            return jsonify({"error": "token_expired"}), 401
        except jwt.InvalidTokenError:
            return jsonify({"error": "invalid_token"}), 401

        return f(*args, **kwargs)
    wrapper.__name__ = f.__name__
    return wrapper

@app.get("/api/auth/authorize")
def authorize():
    redirect_uri = request.args.get("redirect_uri")
    client_id = request.args.get("client_id")
    state = request.args.get("state")
    code = "sevendaysaweek"
    if not redirect_uri:
        return "Erro: redirect_uri não foi fornecido", 400

    if client_id != "tralalerotralala":
        return "Client ID inválido", 401

    return redirect(f"{redirect_uri}?code={code}&state={state}")

if __name__ == '__main__':
    app.run(debug=True)
