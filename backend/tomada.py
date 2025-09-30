import asyncio
import os
from aiohttp import ClientSession
from dotenv import load_dotenv

from modelo_IA.IA_treinada import deve_desligar
from client import GoodweClient

TOMADA_ID = "338c1c8a-c3a2-4715-be92-8911248bbb8c"

async def set_tomada(on: bool):
    client = ClientSession(os.getenv("BROKER_HOST"))
    state = "on" if on else "off"
    resp = await client.post("/api/setstate", params={"state": state, "id": TOMADA_ID})
    d = await resp.json()
    await client.close()
    return d, resp.status

async def get_tomada():
    client = ClientSession(os.getenv("BROKER_HOST"))
    resp = await client.get("/api/query", params={"id": TOMADA_ID})
    d = await resp.json()
    await client.close()
    return d, resp.status

async def main():
    client = await GoodweClient.create("eu")
    http = ClientSession(os.getenv("BACKEND_HOST"))
    while True:
        await asyncio.sleep(10)

        try:
            resp = await http.get("/tomada/get_economia")
            econ = (await resp.json())["state"] == "on"
            if not econ:
                print("Economia desligada, continuando...")
                continue
        except Exception as e:
            print(f"Erro ao ler estado de economia: {e}")
            continue

        bat = await client.cur_bat()
        print(f"nível da bateria: {bat}%")
        bateria_baixa = bat <= 15

        load = await client.cur_load()
        print(f"Consumo atual: {load}W")
        ia_deve_desligar = deve_desligar(client.cur_time().hour, load)
        print(f"IA: {ia_deve_desligar}, bateria baixa: {bateria_baixa}")
        deve_ligar = not bateria_baixa and not ia_deve_desligar

        d, status = await set_tomada(deve_ligar)
        if d["success"] and 200 <= status <= 299:
            print(f"Tomada {"ligada" if deve_ligar else "desligada"} com sucesso")
        else:
            print(f"Tomada não foi {"ligada" if deve_ligar else "desligada"}")

if __name__ == "__main__":
    load_dotenv()
    asyncio.run(main())
