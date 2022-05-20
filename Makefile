ARCH ?= aarch64
MACHINE ?= shyper
PROFILE ?= release
USER_PROFILE ?= release
TRUSTED_PROFILE ?= release

# Panic Inject Function
export PI
# Page Fault Inject Function
export FI

# NOTE: this is to deal with `(signal: 11, SIGSEGV: invalid memory reference)`
# https://github.com/rust-lang/rust/issues/73677
RUSTFLAGS := -C llvm-args=-global-isel=false

# NOTE: generate frame pointer for every function
export RUSTFLAGS := ${RUSTFLAGS} -C force-frame-pointers=yes

CARGO_FLAGS := ${CARGO_FLAGS} --features ${MACHINE}

ifeq (${PROFILE}, release)
CARGO_FLAGS := ${CARGO_FLAGS} --release
endif

ifeq (${TRUSTED_PROFILE}, release)
CARGO_FLAGS := ${CARGO_FLAGS} --features user_release
endif

KERNEL := target/${ARCH}${MACHINE}/${PROFILE}/rust-shyper-os

.PHONY: all emu debug dependencies clean disk trusted_image user_image ramdisk.img

all: ${KERNEL} ${KERNEL}.bin ${KERNEL}.asm

${KERNEL}:
	cargo build --target src/target/${ARCH}${MACHINE}.json -Z build-std=core,alloc  ${CARGO_FLAGS}

${KERNEL}.bin: ${KERNEL}
	aarch64-elf-objcopy $< -O binary $@

${KERNEL}.asm: ${KERNEL}
	aarch64-elf-objdump --demangle -d $< > $@

build: ${KERNEL}.bin ${KERNEL}.asm
	echo "build success!"

clean:
	-cargo clean


QEMU_CMD := qemu-system-aarch64 -M virt -cpu cortex-a53 -device loader,file=${KERNEL},addr=0x80000000,force-raw=on
QEMU_DISK_OPTIONS := -drive file=disk.img,if=none,format=raw,id=x0 \
					 -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
					 -global virtio-mmio.force-legacy=false
QEMU_COMMON_OPTIONS := -serial stdio -display none -smp 4 -m 2048

emu: build
	${QEMU_CMD} ${QEMU_COMMON_OPTIONS} -kernel ${KERNEL}.bin -s


dependencies:
	rustup component add rust-src