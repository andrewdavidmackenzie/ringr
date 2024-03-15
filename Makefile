PI_TARGET := pizero2w0.local

.PHONY: all
all: build

.PHONY: build
build:
	cargo build --target=arm-unknown-linux-musleabi

.PHONY: copy
copy: build
	scp target/arm-unknown-linux-musleabi/debug/ringr andrew@$(PI_TARGET):~/ringr

.PHONY: ssh
ssh:
	ssh andrew@$(PI_TARGET)