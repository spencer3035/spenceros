PACKAGES:=common stage-0 stage-1 stage-2 stage-3 spenceros

all: build

run: build
	cargo run

build:
	cargo build

test: 
	for p in ${PACKAGES}; do cargo test --package $$p ;  done
