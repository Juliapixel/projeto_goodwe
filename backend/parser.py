import pandas as pd
import glob
import os

def carregar_dados() -> pd.DataFrame:
    caminho = os.path.join("modelo_IA", "dados_Goodwe", "*.xls")
    arquivos = glob.glob(caminho)
    dfs = []

    for arq in arquivos:
        try:
            df = pd.read_excel(arq, skiprows=2, engine="xlrd")
            df.columns = ["Time", "PV(W)", "SOC(%)", "Battery(W)", "Grid(W)", "Load(W)"]
            df["Time"] = pd.to_datetime(df["Time"], format="%d.%m.%Y %H:%M:%S")
            dfs.append(df)
        except Exception as e:
            print(f"Erro ao ler {arq}: {e}")

    if not dfs:
        raise FileNotFoundError("Nenhum arquivo de dados encontrado.")

    return pd.concat(dfs, ignore_index=True).sort_values("Time").reset_index(drop=True)