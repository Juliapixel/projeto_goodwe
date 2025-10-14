# Firmware

## Configuração

É necessário adicionar algumas variáveis de ambiente em um .env nesta pasta.

```bash
SSID=WIFI123 # SSID do Wi-Fi primário
PASSWORD=SENHA123 # Senha do Wi-Fi primário

SSID2=WIFI2 # Opcional: SSID do Wi-Fi secundário
PASSWORD2=SENHA2 # Opcional: Senha do Wi-Fi secundário

SSID3=WIFI3 # Opcional: SSID do Wi-Fi terciário
PASSWORD3=SENHA3 # Opcional: Senha do Wi-Fi terciário

BROKER_IP=123.123.123.123 # Opcional: IPv4 do broker (usado somente em caso de falha de DNS)
BROKER_PORT=8080 # Porta do broker
BROKER_HOST=broker.example.com # Opcional: Hostname do broker (padrão: goodwe.juliapixel.com)
```
