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

## Rotas

### GET `/api/assistente`

#### Retorna

```json
{
    "consumo_diario_kwh": float,
    "consumo_semanal_kwh": float,
    "economia_diaria_kwh": float,
    "economia_semanal_kwh": float,
    "prod_diaria_kwh": float,
    "prod_mensal_kwh": float,
    "bateria": float,
}
```

### GET `/api/dados/bateria_agora`

#### Retorna

```json
{
    "data": 50 // em %
}
```

### GET `/api/dados/consumo_agora`

#### Retorna

```json
{
    "data": 123.0 // em W
}
```

### GET `/api/dados/producao_agora`

#### Retorna

```json
{
    "data": 123 // em W
}
```

### GET `/api/graficos/econ_semana`

#### Retorna

```json
// array de 7 tuplas, começando no dia mais antigo até o dia atual
[
    [
        "terça-feira", // dia da semana em português
        123 // em kWh
    ],
    [
        "quarta-feira", // dia da semana em português
        456 // em kWh
    ],
]
```

### GET `/api/dados`

Endpoint para gráficos do dia atual com vários dados diferentes

#### Retorna

```json
// mapa de lista de tuplas com valores
{
    // geração fotovoltaica
    "pv": [
        [
            "2025-10-01T00:10:00+02:00", // Data da medição em ISO8601
            1234, // valor
        ],
        ...
        [
            "2025-10-01T02:15:00+02:00"
            5678
        ]
    ],
    // tensão da bateria em V
    "bat": [...],
    // medidor do relógio da concessionária em W
    "meter": [...],
    // consumo em W
    "load": [...],
    // carga da bateria em %
    "charge": [...],
}
```

### POST `/api/tomada/set`

Liga ou desliga a tomada

#### Parâmetros

`state=on`,
`state=off`

(auto-explicativo)

#### Retorna

```json
{
    "success": bool, // se completou com sucesso
    "present": bool // se a tomada está presente
}
```

### GET `/api/tomada/get`

#### Retorna

```json
// ambos os campos são null se a tomada está desconectada
{
    "state": "on" | "off" | null, // estado atual
    "lastseen": "2025-10-01T02:15:00Z" | null // ultima mensagem recebida pela tomada em ISO8601
}
```

### POST `/api/tomada/set_economia`

Liga ou desliga a economia de energia

#### Parâmetros

`state=on`,
`state=off`

(auto-explicativo)

#### Retorna

Status 200

```json
{}
```

### GET `/api/tomada/get_economia`

#### Retorna

```json
{
    "state": "on" | "off" // estado atual
}
```

### GET `/api/`

#### Retorna

```json

```
