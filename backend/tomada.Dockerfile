# Dockerfile para o serviço de desligamento automático da tomada
FROM python:3.13-slim-bookworm
RUN pip install uv
WORKDIR /app
COPY requirements.txt .
RUN uv pip install --system -r requirements.txt
COPY . .
ENTRYPOINT [ "python", "tomada.py" ]
