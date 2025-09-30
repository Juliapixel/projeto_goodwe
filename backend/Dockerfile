FROM python:3.13-slim-bookworm
WORKDIR /app
COPY . .
RUN pip install -r requirements.txt
ENTRYPOINT [ "hypercorn", "app.py" ]
