ARCH ?= aarch64
PROFILE ?= release
MACHINE ?= qemu

# Panic Inject Function
export PI

export RUSTFLAGS := ${RUSTFLAGS} -C force-frame-pointers=yes

# Cargo flags.
ifeq (${PROFILE}, release)
CARGO_FLAGS = --release  --no-default-features
else
CARGO_FLAGS =  --no-default-features
endif

CARGO_FLAGS := ${CARGO_FLAGS} --features "${MACHINE}"

# Target directory.
TARGET_CFG := ../../cfg/${ARCH}${MACHINE}.json
KERNEL := target/${ARCH}${MACHINE}/${PROFILE}/fault_recover

# Arch-specific tools
OBJCOPY := aarch64-elf-objcopy
OBJDUMP := aarch64-elf-objdump

.PHONY: build clean

build:
	cargo build --target ${TARGET_CFG} -Z build-std=core,alloc -Z build-std-features=compiler-builtins-mem ${CARGO_FLAGS} 
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

emu: build
	${QEMU_CMD} ${QEMU_COMMON_OPTIONS} \
		-kernel ${KERNEL}.bin -s

debug: build
	${QEMU_CMD} ${QEMU_COMMON_OPTIONS} \
		-kernel ${KERNEL}.bin -s -S

clean:
	cargo clean