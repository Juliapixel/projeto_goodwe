from flask import Flask, jsonify
from datetime import datetime
from client import crosslogin, get_inverter_data_by_column

app = Flask(__name__)

@app.route('/dados')
def dados_goodwe():
    try:
        account = "demo@goodwe.com"
        password = "GoodweSems123!@#"
        inverter_sn = "5010KETU229W6177"
        date_str = datetime.now().strftime("%Y-%m-%d 00:00:00")

        token = crosslogin(account, password)
        dados = get_inverter_data_by_column(token, inverter_sn, "Pac", date_str)
        return jsonify(dados)
    except Exception as e:
        return jsonify({"erro": str(e)}), 500

if __name__ == '__main__':
    app.run(debug=True)
