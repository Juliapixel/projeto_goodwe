# Dockerfile para o serviço principal da API/backend
FROM python:3.13-slim-bookworm
RUN pip install uv
WORKDIR /app
COPY requirements.txt .
RUN uv pip install --system -r requirements.txt
COPY . .
ENTRYPOINT [ "hypercorn", "--bind=0.0.0.0", "app.py" ]
