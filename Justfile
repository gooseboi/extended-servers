set positional-arguments

build:
	cargo build

@simple-recv protocol port: build
	cargo run --bin simple-$1 -- receiver $2

@simple-send protocol port: build
	cargo run --bin simple-$1 -- sender $2
