# Dockerfile para o serviço principal da API/backend
FROM python:3.13-slim-bookworm
WORKDIR /app
COPY requirements.txt .
RUN pip install -r requirements.txt
COPY . .
ENTRYPOINT [ "hypercorn", "--bind=0.0.0.0", "app.py" ]
