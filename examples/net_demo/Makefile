ARCH ?= aarch64
MACHINE ?= qemu
PROFILE ?= release

# Cargo flags.
ifeq (${PROFILE}, release)
CARGO_FLAGS = --release  --no-default-features
else
CARGO_FLAGS =  --no-default-features
endif

ifeq (${MACHINE}, shyper)
CARGO_FLAGS := ${CARGO_FLAGS} --features "shyper"
else
CARGO_FLAGS := ${CARGO_FLAGS} --features "qemu"
endif

# Target directory.
TARGET_CFG := ../../cfg/${ARCH}${MACHINE}.json
KERNEL_DIR := target/${ARCH}${MACHINE}/${PROFILE}/

# Arch-specific tools
OBJCOPY := rust-objcopy
OBJDUMP := rust-objdump

.PHONY: server client clean

server-bw:
	cargo build --target ${TARGET_CFG} --bin $@ \
		-Z build-std=core,alloc ${CARGO_FLAGS}
	${OBJCOPY} ${KERNEL_DIR}$@ -O binary ${KERNEL_DIR}$@.bin
	${OBJDUMP} --demangle -d ${KERNEL_DIR}$@ > ${KERNEL_DIR}$@.asm

client-bw:
	cargo build --target ${TARGET_CFG} --bin $@ \
		-Z build-std=core,alloc ${CARGO_FLAGS}
	${OBJCOPY} ${KERNEL_DIR}$@ -O binary ${KERNEL_DIR}$@.bin
	${OBJDUMP} --demangle -d ${KERNEL_DIR}$@ > ${KERNEL_DIR}$@.asm

server-latency:
	cargo build --target ${TARGET_CFG} --bin $@ \
		-Z build-std=core,alloc ${CARGO_FLAGS}
	${OBJCOPY} ${KERNEL_DIR}$@ -O binary ${KERNEL_DIR}$@.bin
	${OBJDUMP} --demangle -d ${KERNEL_DIR}$@ > ${KERNEL_DIR}$@.asm

client-latency:
	cargo build --target ${TARGET_CFG} --bin $@ \
		-Z build-std=core,alloc ${CARGO_FLAGS}
	${OBJCOPY} ${KERNEL_DIR}$@ -O binary ${KERNEL_DIR}$@.bin
	${OBJDUMP} --demangle -d ${KERNEL_DIR}$@ > ${KERNEL_DIR}$@.asm

QEMU_NETWORK_OPTIONS := -netdev tap,id=tap0,ifname=tap0,script=no,downscript=no \
						-device virtio-net-device,mac=48:b0:2d:0e:6e:9e,netdev=tap0 \
						-global virtio-mmio.force-legacy=false
QEMU_COMMON_OPTIONS := -serial stdio -display none -smp 4 -m 2048

server_bw_emu: server-bw
	sudo qemu-system-aarch64 -M virt -cpu cortex-a53 \
		-device loader,file=${KERNEL_DIR}$<,addr=0x80000000,force-raw=on \
		${QEMU_COMMON_OPTIONS} \
		${QEMU_NETWORK_OPTIONS} \
		-kernel ${KERNEL_DIR}$<.bin -s

client_bw_emu: client-bw
	sudo qemu-system-aarch64 -M virt -cpu cortex-a53 \
		-device loader,file=${KERNEL_DIR}$<,addr=0x80000000,force-raw=on \
		${QEMU_COMMON_OPTIONS} \
		${QEMU_NETWORK_OPTIONS} \
		-kernel ${KERNEL_DIR}$<.bin -s

server_latency_emu: server-latency
	sudo qemu-system-aarch64 -M virt -cpu cortex-a53 \
		-device loader,file=${KERNEL_DIR}$<,addr=0x80000000,force-raw=on \
		${QEMU_COMMON_OPTIONS} \
		${QEMU_NETWORK_OPTIONS} \
		-kernel ${KERNEL_DIR}$<.bin -s

client_latency_emu: client-latency
	sudo qemu-system-aarch64 -M virt -cpu cortex-a53 \
		-device loader,file=${KERNEL_DIR}$<,addr=0x80000000,force-raw=on \
		${QEMU_COMMON_OPTIONS} \
		${QEMU_NETWORK_OPTIONS} \
		-kernel ${KERNEL_DIR}$<.bin -s

server_bw_debug: server-bw
	sudo qemu-system-aarch64 -M virt -cpu cortex-a53 \
		-device loader,file=${KERNEL_DIR}$<,addr=0x80000000,force-raw=on \
		${QEMU_COMMON_OPTIONS} \
		${QEMU_NETWORK_OPTIONS} \
		-kernel ${KERNEL_DIR}$<.bin -s -S

client_bw_debug: client-bw
	sudo qemu-system-aarch64 -M virt -cpu cortex-a53 \
		-device loader,file=${KERNEL_DIR}$<,addr=0x80000000,force-raw=on \
		${QEMU_COMMON_OPTIONS} \
		${QEMU_NETWORK_OPTIONS} \
		-kernel ${KERNEL_DIR}$<.bin -s -S

clean:
	-cargo clean