PI_TARGET := pizero2w0.local

.PHONY: all
all: clippy build

.PHONY: clippy
clippy:
	CROSS_CONTAINER_OPTS="--platform linux/amd64" cross clippy --release --target=aarch64-unknown-linux-gnu

.PHONY: build
build:
	CROSS_CONTAINER_OPTS="--platform linux/amd64" cross build --release --target=aarch64-unknown-linux-gnu

.PHONY: copy
copy: build
	scp target/aarch64-unknown-linux-gnu/release/ringr andrew@$(PI_TARGET):~/ringr

.PHONY: ssh
ssh:
	ssh andrew@$(PI_TARGET)