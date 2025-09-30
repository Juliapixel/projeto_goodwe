# Backend

Backend em python para nosso serviço

## Rodar

### Instale as dependências

Windows:

```ps1
python -m venv .venv
.\.venv\Scripts\activate.ps1
pip install -r requirements.txt
```

Linux/MacOS:

```bash
python -m venv .venv
source .venv/Scripts/activate
pip install -r requirements.txt
```

### Worker da tomada

```bash
export BROKER_HOST=https://example.com
python tomada.py
```

### Serviço do backend

```bash
export BROKER_HOST=https://example.com
hypercorn app.py
```
