set positional-arguments

build:
	cargo build

@simple-recv protocol port: build
	cargo run --bin simple-$1 -- receiver $2

@simple-send protocol port: build
	cargo run --bin simple-$1 -- sender $2

@file-recv protocol port file: build
	cargo run --bin file-$1 -- receiver $2 $3

@file-send protocol port file: build
	cargo run --bin file-$1 -- sender $2 $3
