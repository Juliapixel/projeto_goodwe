arch:="amd64"

_default:
    just --list

build_webserver:
    docker build -t goodwe_webserver -f webserver.Dockerfile --platform linux/{{arch}} .

# modulos não funcionam direito infelizmente
build_all: build_webserver
    just -f backend/justfile arch={{arch}} build
    just -f tomada/justfile arch={{arch}} build_docker

run_all: build_all
    docker compose up
