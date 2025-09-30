from flask import Flask, redirect, request, jsonify
import jwt
import datetime

app = Flask(__name__)

SECRET_KEY = "oieuamoobts"

@app.route("/auth/token", methods=["POST"])
def generate_token():
    data = request.get_json()
    client_id = data.get("client_id")
    client_secret = data.get("client_secret")
    grant_type = data.get("grant_type")


    if grant_type != "client_credentials" or client_id != "tralalerotralala" or client_secret != "123456":
        return jsonify({"error": "invalid_client"}), 401

    # token JWT
    payload = {
        "sub": client_id,
        "scope": "read write",
        "exp": datetime.datetime.now(datetime.timezone.utc) + datetime.timedelta(days=365)
    }
    token = jwt.encode(payload, SECRET_KEY, algorithm="HS256")

    return jsonify({
        "access_token": token,
        "token_type": "Bearer",
        "expires_in": 22896000,
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
            request.user = decoded
        except jwt.ExpiredSignatureError:
            return jsonify({"error": "token_expired"}), 401
        except jwt.InvalidTokenError:
            return jsonify({"error": "invalid_token"}), 401

        return f(*args, **kwargs)
    wrapper.__name__ = f.__name__
    return wrapper

@app.route("/dados", methods=["GET"])
@token_required
def dados_protegidos():
    return jsonify({"msg": "Você acessou o endpoint protegido", "user": request.user})

if __name__ == "__main__":
    app.run(debug=True)

@app.route("/auth/authorize", methods=["GET"])
def authorize():

    redirect_uri = request.args.get("redirect_uri")
    client_id = request.args.get("client_id")
    state = request.args.get("state")
    code = "sevendaysaweek"
    if not redirect_uri:
        return "Erro: redirect_uri não foi fornecido", 400

    if client_id != "traladetraladala":
        return "Client ID inválido", 401

    return redirect(f"{redirect_uri}?code={code}&state={state}")
