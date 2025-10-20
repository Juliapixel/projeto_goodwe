import os
import asyncio
from typing import Any, Dict, List, Optional

# Tenta importar o SDK Tuya 
try:
    from tuya_iot import TuyaOpenAPI
except Exception:
    TuyaOpenAPI = None  # se não instalado, o código avisará ao criar o client

# Lê variáveis de ambiente (defina em .env)
TUYA_CLIENT_ID = os.getenv("TUYA_CLIENT_ID", "")
TUYA_CLIENT_SECRET = os.getenv("TUYA_CLIENT_SECRET", "")
TUYA_REGION = os.getenv("TUYA_REGION", "eu")  # ex: eu, us, asia

# Base URLs por região (ajuste se necessário)
REGION_BASES = {
    "eu": "https://openapi.tuyaeu.com",
    "us": "https://openapi.tuyaus.com",
    "asia": "https://openapi.tuyacn.com",
}

class TuyaClient:
    """
    Cliente simples para Tuya Cloud baseado no SDK.
    Métodos principais:
      - create(): inicializa/ conecta ao SDK
      - close(): limpa (se necessário)
      - get_device_status(device_id): retorna payload completo do device (inclui 'dps')
      - get_device_dps(device_id): retorna apenas o dps dict (útil para descobrir keys)
      - get_device_power(device_id): tenta extrair consumo instantâneo (W)
    """

    def __init__(self, base_url: Optional[str] = None):
        # escolhe base pela região se base_url não for passada
        self.base_url = base_url or REGION_BASES.get(TUYA_REGION, REGION_BASES["eu"])
        self._api: Optional[TuyaOpenAPI] = None

    @classmethod
    async def create(cls):
        """
        Cria a instância e conecta ao SDK.
        O SDK é síncrono, por isso usamos to_thread para não bloquear o loop async.
        """
        self = cls()
        if TuyaOpenAPI is None:
            raise RuntimeError("SDK tuya-iot-py-sdk não encontrado. Instale com: pip install tuya-iot-py-sdk")

        def init_and_connect():
            api = TuyaOpenAPI(self.base_url, TUYA_CLIENT_ID, TUYA_CLIENT_SECRET)
            api.connect()
            return api

        # roda a inicialização em thread para não travar o loop
        self._api = await asyncio.to_thread(init_and_connect)
        return self

    async def close(self):
        # SDK geralmente não precisa de close; mantenho método para simetria com GoodweClient
        self._api = None

    async def get_device_status(self, device_id: str) -> Dict[str, Any]:
        """
        Retorna o payload do endpoint /v1.0/devices/{device_id}/status
        Ex.: {'result': {'dps': {...}, ...}, ...}
        """
        if self._api is None:
            raise RuntimeError("TuyaClient não inicializado. Chame TuyaClient.create()")
        # a chamada do SDK pode ser feita via método get; executamos em thread
        def sync_get():
            return self._api.get(f"/v1.0/devices/{device_id}/status")
        res = await asyncio.to_thread(sync_get)
        # alguns SDKs retornam {'result': {...}}; normalizar retornando o 'result' quando existir
        if isinstance(res, dict) and "result" in res:
            return res["result"]
        return res

    async def get_device_dps(self, device_id: str) -> Dict[str, Any]:
        """Retorna apenas o dps dict (útil para depuração e para achar a chave do consumo)."""
        status = await self.get_device_status(device_id)
        dps = status.get("dps", {}) if isinstance(status, dict) else {}
        return dps

    async def get_device_power(self, device_id: str) -> float:
        """
        Tenta extrair leitura de potência (W) a partir do dps.
        Passo recomendado: primeiro chame get_device_dps para ver as keys reais do seu dispositivo.
        Ajuste possible_keys se necessário.
        """
        dps = await self.get_device_dps(device_id)
        # chaves comuns ou numéricas que alguns devices usam para consumo
        possible_keys = ["power", "current_power", "instant_power", "18", "17", "21", "22"]
        for key in possible_keys:
            if key in dps:
                try:
                    return float(dps[key])
                except Exception:
                    # se não conseguir converter, continua tentando outras keys
                    pass
        # fallback: se Tuya fornecer endpoint de energy, pode ser necessário usar outro endpoint.
        raise RuntimeError(f"Não encontrei leitura de power no dps. Keys encontradas: {list(dps.keys())}")