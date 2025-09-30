import glob # automatiza o processo de pegar todos os arquivos xls na pasta do note
import pandas as pd
from sklearn.ensemble import RandomForestClassifier
from sklearn.model_selection import train_test_split
import os

# ===============================
# 1) Carregar e Preparar Dados
# ===============================

'''
procura todos os arquivos de dados xls na pasta e cria uma lista com os nomes desses arquivos
'''
caminho_pasta = os.path.join(os.path.dirname(__file__), "dados_Goodwe")
arquivos = glob.glob(os.path.join(caminho_pasta, "*.xls")) + glob.glob(os.path.join(caminho_pasta, "*.xlsx"))


'''
para cada arquivo:
    le os dados
    padroniza os nomes das colunas
    converte coluna de tempo pra formato data-hora
    renomeia a coluna de consumo pra load
    limpa dados ruins faltando
    guarda os dados limpos numa lista
    mostra quantos registros foram lidos de cada arquivo
'''
lista_dfs = []

for arq in arquivos:
    try:
        if arq.endswith(".xlsx"):
            df_temp = pd.read_excel(arq, header=2, engine="openpyxl")
        else:
            df_temp = pd.read_excel(arq, header=2, engine="xlrd")

        df_temp.columns = df_temp.columns.str.strip().str.lower()

        if "time" in df_temp.columns:
            df_temp["time"] = pd.to_datetime(df_temp["time"], errors="coerce", dayfirst=True)

        for col in df_temp.columns:
            if "load" in col:
                df_temp.rename(columns={col: "load"}, inplace=True)

        df_temp = df_temp.dropna(subset=["time", "load"])
        df_temp["load"] = pd.to_numeric(df_temp["load"], errors="coerce")

        lista_dfs.append(df_temp)
        # print(f"✅ {os.path.basename(arq)}: {len(df_temp)} registros")

    except Exception as e:
        print(f"⚠️ Erro em {arq}: {e}")


'''
junta todos os dados dos arquivos em um unico df
organiza por tempo
'''
df = pd.concat(lista_dfs, ignore_index=True)
df = df.sort_values('time').reset_index(drop=True)

# print(f"\n📊 Dataset consolidado: {len(df)} registros de {df['time'].dt.date.nunique()} dias")


'''
cria coluna com a hora de cada registro
calcula o consumo médio na madrugada (standby)
calcula o limite de standby como o valor que esta no topo dos 90% menores consumos da madrugada
'''
df["hora"] = df["time"].dt.hour
standby = df[df["hora"].between(0, 5)]["load"].mean()
standby_limite = df[df["hora"].between(0,5)]["load"].quantile(0.75)


'''
marca para desligar standby (valor 1) se:
    for madrugada entre 0h e 5h
    e o consu,o for menor ou igual a 400W
caso contrario, marca 0 (manter ligado)
'''
df["deve_desligar"] = ((df["load"] <= 400)).astype(int)

'''
separa dados para treinar IA
X = dados que serao usados para prever (hora e consumo)
y = a decisao que queremos que o modelo aprenda (desligar ou nao)
'''
X = df[["hora", "load"]]
y = df["deve_desligar"]


'''
treino e teste
separa parte dos dados pra treino e parte pra teste
treina modelo pra aprender a decidir quando desligar
mostra acuracia (percentual de acertos no teste
'''
X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.2)
modelo = RandomForestClassifier()
modelo.fit(X_train, y_train)

# Teste de acurácia
# print("Acurácia:", modelo.score(X_test, y_test))


'''
simula uma decisao com valores fixos
hora = 2h e consumo = 105W
'''
# Simulação de decisão
# Exemplo: hora=2, load=105
previsao = modelo.predict([[2, 105]])
# print("Ação:", "Desligar standby!" if previsao[0] == 1 else "Manter ligado!")


'''
aplica o modelo pra todos do dataset
o modelo faz previsao pra todos os registros
add o resultado "Desligar Standby!" ou "Manter ligado!" em uma nova coluna
Mostra os primeiros 2000 resultados
'''
df["acao_predita"] = modelo.predict(df[["hora", "load"]])
df["acao_predita"] = df["acao_predita"].map({1: "Desligar standby!", 0: "Manter ligado!"})


'''
Calculos de Economia
Consumo Real vs Otimizado
'''
# Define intervalo entre medições (ajuste conforme seus dados)
intervalo_medicao = 5  # minutos por medição
horas_por_medicao = intervalo_medicao / 60

# Cria coluna com consumo otimizado (quando IA decide desligar, economiza 70% do consumo)
df["consumo_otimizado"] = df["load"].copy()
mask_desligar = df["acao_predita"] == "Desligar standby!"
df.loc[mask_desligar, "consumo_otimizado"] = df.loc[mask_desligar, "load"] * 0.3

# VARIÁVEIS PARA O SEU PRINT:
energia_real_kwh = (df["load"] * horas_por_medicao / 1000).sum()
energia_ai_kwh = (df["consumo_otimizado"] * horas_por_medicao / 1000).sum()
economia_kwh = energia_real_kwh - energia_ai_kwh
preco_kwh = 0.65  # R$ por kWh - ajuste conforme sua região
economia_reais = economia_kwh * preco_kwh
dia_escolhido = f"{df['time'].dt.date.min()} a {df['time'].dt.date.max()}"

# print("\n📊 RESULTADOS", dia_escolhido)
# print("Consumo real:", round(energia_real_kwh, 2), "kWh")
# print("Consumo otimizado IA:", round(energia_ai_kwh, 2), "kWh")
# print("Economia:", round(economia_kwh, 2), "kWh → R$", round(economia_reais, 2))


# Filtra por dia 16 e horas entre 16 e 18
df_filtrado = df[
    (df["time"].dt.date == pd.to_datetime("2025-09-16").date()) &
    (df["hora"].between(10, 12))
]

# print(df_filtrado[["time", "load", "hora", "acao_predita"]])

'''
testa desligamento com base nos valores de limites
150W --> mais seguro (desliga só nos consumos realmente baixos); pode deixar de desligar nos momentos que PODIAM ser em standby
200W --> ainda seguro, um pouco mais "agressivo"
250W - 300W --> mais abrangente; pode acabar desligando em momentos que não são standby verdadeiro
qual seria melhor?
'''
for limite in [150, 200, 250, 300]:
    df["deve_desligar"] = ((df["hora"].between(0, 5)) & (df["load"] <= limite)).astype(int)
    # print(f"Com limite {limite}W, quantidade de desligamentos:", df["deve_desligar"].sum())

'''
funções para backend
1. tomada liga ou desliga
2. consumo real
3. consumo otimizado
4. calculo economia de energia
'''

def deve_desligar(hora, load):
    return modelo.predict(pd.DataFrame({"hora": [hora], "load": [load]}))[0] == 1


def calcular_economia(load_atual: float, deve_desligar: bool) -> float:
    """
    Calcula a economia de energia em W

    Args:
        load_atual (float): Consumo atual em Watts
        deve_desligar (bool): True se deve desligar, False se mantém ligado

    Returns:
        consumo_otimizado_w (float)
    """

    if deve_desligar:
        consumo_otimizado_w = (load_atual * 0.3)
    else:
        consumo_otimizado_w = load_atual

    # Economia
    economia_w = load_atual - consumo_otimizado_w

    return economia_w
