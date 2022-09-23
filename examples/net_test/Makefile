ARCH ?= aarch64
MACHINE ?= qemu
GCC ?= gcc

.PHONY: all build clean

build:
	gcc socket_client.c -o client
	gcc socket_server.c -o server

clean:
	@if [ -e client ]; then rm client; fi
	@if [ -e server ]; then rm server; fi
