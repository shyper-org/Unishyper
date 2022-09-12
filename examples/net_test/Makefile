ARCH ?= aarch64
MACHINE ?= qemu
GCC ?= gcc

.PHONY: all build clean

build:
	gcc socket_client.c -o client
	gcc socket_server.c -o server

clean:
	rm client
	rm server
