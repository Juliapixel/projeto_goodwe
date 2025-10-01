arch:="amd64"
ssh:=""

_default:
    just --list

build_webserver:
    docker build -t goodwe_webserver -f webserver.Dockerfile --platform linux/{{arch}} .

# modulos não funcionam direito infelizmente
build_all: build_webserver
    just -f backend/justfile arch={{arch}} build
    just -f tomada/justfile arch={{arch}} build_docker

deploy_all: build_all
    docker save --platform=linux/{{ arch }} goodwe_broker:latest | zstd -T8 -5 | pv -W | ssh {{ ssh }} 'docker load'
    docker save --platform=linux/{{ arch }} goodwe_webserver:latest | zstd -T8 -5 | pv -W | ssh {{ ssh }} 'docker load'
    docker save --platform=linux/{{ arch }} goodwe_backend:latest | zstd -T8 -5 | pv -W | ssh {{ ssh }} 'docker load'
    docker save --platform=linux/{{ arch }} goodwe_tomada:latest | zstd -T8 -5 | pv -W | ssh {{ ssh }} 'docker load'

run_all: build_all
    docker compose up
