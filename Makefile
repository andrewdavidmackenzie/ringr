PI_TARGET := pizero2w0.local

.PHONY: all
all: build

.PHONY: build
build:
	cross build --target=aarch64-unknown-linux-gnu

.PHONY: copy
copy: build
	scp target/aarch64-unknown-linux-gnu/debug/ringr andrew@$(PI_TARGET):~/ringr

.PHONY: ssh
ssh:
	ssh andrew@$(PI_TARGET)