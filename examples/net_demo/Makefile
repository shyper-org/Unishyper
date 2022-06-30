ARCH ?= aarch64
MACHINE ?= shyper
PROFILE ?= release

ifeq (${PROFILE}, release)
CARGO_FLAGS = --release
else
CARGO_FLAGS =
endif

KERNEL := target/${ARCH}${MACHINE}/${PROFILE}/net_demo

.PHONY: all clean

all:
	cargo build --target ${ARCH}${MACHINE}.json -Z build-std=core,alloc ${CARGO_FLAGS}
	aarch64-elf-objcopy ${KERNEL} -O binary ${KERNEL}.bin
	aarch64-elf-objdump --demangle -d ${KERNEL} > ${KERNEL}.asm

clean:
	-cargo clean