arch:="amd64"

_default:
    just --list

build_all:
    just -f backend/justfile arch={{arch}} build
    just -f tomada/justfile arch={{arch}} build_docker

run_all: build_all
    docker compose up
