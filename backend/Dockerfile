FROM python:3.13-slim-bookworm
WORKDIR /app
COPY . .
RUN python -m venv .venv
RUN source .venv/Scripts/activate && pip install -r requirements.txt
ENTRYPOINT [ "hypercorn", "app.py" ]
