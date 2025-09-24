from flask import Flask, jsonify
from datetime import datetime
from client import crosslogin, get_inverter_data_by_column
from parser import tratar_dados_energia, gerar_datas_desde_instalacao

app = Flask(__name__)

@app.route('/assistente')
def dados_assistente():
    try:
        account = "demo@goodwe.com"
        password = "GoodweSems123!@#"
        inverter_sn = "5010KETU229W6177"
        data_instalacao = "2024-12-06" 

        token = crosslogin(account, password)
        datas = gerar_datas_desde_instalacao(data_instalacao, limite_dias=30)

        registros_eday = []
        registros_bateria = []

        for data_str in datas:  
            eday = get_inverter_data_by_column(token, inverter_sn, "Eday", data_str)
            bateria = get_inverter_data_by_column(token, inverter_sn, "Cbattery1", data_str)

            registros_eday.extend(eday.get("data", {}).get("column1", []))
            registros_bateria.extend(bateria.get("data", {}).get("column1", []))

        dados_tratados = tratar_dados_energia(
            {"data": {"column1": registros_eday}},
            {"data": {"column1": registros_bateria}}
        )

        return jsonify(dados_tratados)
    except Exception as e:
        return jsonify({"erro": str(e)}), 500

if __name__ == '__main__':
    app.run(debug=True)
