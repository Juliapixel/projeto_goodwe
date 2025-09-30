# Dockerfile para o serviço de desligamento automático da tomada
FROM python:3.13-slim-bookworm
WORKDIR /app
COPY . .
RUN pip install -r requirements.txt
ENTRYPOINT [ "python", "tomada.py" ]
