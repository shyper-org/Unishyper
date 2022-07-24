ARCH ?= aarch64
MACHINE ?= shyper
PROFILE ?= release

ifeq (${PROFILE}, release)
CARGO_FLAGS = --release
else
CARGO_FLAGS =
endif

KERNEL_DIR := target/${ARCH}${MACHINE}/${PROFILE}
SERVER_KERNEL := ${KERNEL_DIR}/server
CLIENT_KERNEL := ${KERNEL_DIR}/client

.PHONY: server client clean

server:
	cargo build --target ${ARCH}${MACHINE}.json \
		--bin server \
		-Z build-std=core,alloc ${CARGO_FLAGS}
	aarch64-elf-objcopy ${SERVER_KERNEL} -O binary ${SERVER_KERNEL}.bin
	aarch64-elf-objdump --demangle -d ${SERVER_KERNEL} > ${SERVER_KERNEL}.asm

client:
	cargo build --target ${ARCH}${MACHINE}.json \
		--bin client \
		-Z build-std=core,alloc ${CARGO_FLAGS}
	aarch64-elf-objcopy ${CLIENT_KERNEL} -O binary ${CLIENT_KERNEL}.bin
	aarch64-elf-objdump --demangle -d ${CLIENT_KERNEL} > ${CLIENT_KERNEL}.asm

QEMU_NETWORK_OPTIONS := -netdev tap,id=tap0,ifname=tap0,script=no,downscript=no \
						-device virtio-net-device,mac=48:b0:2d:0e:6e:9e,netdev=tap0 \
						-global virtio-mmio.force-legacy=false
QEMU_COMMON_OPTIONS := -serial stdio -display none -smp 4 -m 2048

server_emu: server
	sudo qemu-system-aarch64 -M virt -cpu cortex-a53 \
		-device loader,file=${SERVER_KERNEL},addr=0x80000000,force-raw=on \
		${QEMU_COMMON_OPTIONS} \
		${QEMU_NETWORK_OPTIONS} \
		-kernel ${SERVER_KERNEL}.bin -s

client_emu: client
	sudo qemu-system-aarch64 -M virt -cpu cortex-a53 \
		-device loader,file=${CLIENT_KERNEL},addr=0x80000000,force-raw=on \
		${QEMU_COMMON_OPTIONS} \
		${QEMU_NETWORK_OPTIONS} \
		-kernel ${CLIENT_KERNEL}.bin -s

server_debug: server
	sudo qemu-system-aarch64 -M virt -cpu cortex-a53 \
		-device loader,file=${SERVER_KERNEL},addr=0x80000000,force-raw=on \
		${QEMU_COMMON_OPTIONS} \
		${QEMU_NETWORK_OPTIONS} \
		-kernel ${SERVER_KERNEL}.bin -s -S

client_debug: client
	sudo qemu-system-aarch64 -M virt -cpu cortex-a53 \
		-device loader,file=${CLIENT_KERNEL},addr=0x80000000,force-raw=on \
		${QEMU_COMMON_OPTIONS} \
		${QEMU_NETWORK_OPTIONS} \
		-kernel ${CLIENT_KERNEL}.bin -s -S

clean:
	-cargo clean