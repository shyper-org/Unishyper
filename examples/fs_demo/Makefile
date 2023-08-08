ARCH ?= aarch64
PROFILE ?= release
MACHINE ?= qemu

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
KERNEL := ../../target/${ARCH}${MACHINE}/${PROFILE}/fs_demo

# Arch-specific tools
OBJCOPY := rust-objcopy
OBJDUMP := rust-objdump

.PHONY: build clean

build:
	cargo build --target ${TARGET_CFG} -Z build-std=core,alloc ${CARGO_FLAGS}
	${OBJCOPY} ${KERNEL} -O binary ${KERNEL}.bin
	${OBJDUMP} --demangle -d ${KERNEL} > ${KERNEL}.asm

QEMU_CMD := qemu-system-aarch64 -M virt -cpu cortex-a53 \
			-device loader,file=${KERNEL},addr=0x80000000,force-raw=on
QEMU_DISK_OPTIONS := -drive file=disk.img,if=none,format=raw,id=x0 \
					 -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
					 -global virtio-mmio.force-legacy=false
QEMU_NETWORK_OPTIONS := -netdev tap,id=tap0,ifname=tap0,script=no,downscript=no \
						-device virtio-net-device,mac=48:b0:2d:0e:6e:9e,netdev=tap0 \
						-global virtio-mmio.force-legacy=false
QEMU_COMMON_OPTIONS := -serial stdio -display none -smp 4 -m 2048

emu: build disk
	${QEMU_CMD} ${QEMU_COMMON_OPTIONS} \
		${QEMU_DISK_OPTIONS} \
		-kernel ${KERNEL}.bin -s

debug: build disk
	${QEMU_CMD} ${QEMU_COMMON_OPTIONS} \
		${QEMU_DISK_OPTIONS} \
		-kernel ${KERNEL}.bin -s -S

disk:
ifeq ($(wildcard disk.img),)
	rm -rf disk
	dd if=/dev/zero of=disk.img bs=4096 count=92160 2>/dev/null
	mkfs.fat -F 32 disk.img
endif

clean:
	-cargo clean