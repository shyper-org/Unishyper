ARCH ?= aarch64
USER_PROFILE ?= release
MACHINE ?= shyper

ifeq (${USER_PROFILE}, release)
CARGO_FLAGS = --release
else
CARGO_FLAGS =
endif

KERNEL := target/${ARCH}${MACHINE}/${USER_PROFILE}/user

.PHONY: all clean

all:
	cargo build --target ${ARCH}${MACHINE}.json -Z build-std=core,alloc ${CARGO_FLAGS}
	aarch64-elf-objcopy ${KERNEL} -O binary ${KERNEL}.bin
	aarch64-elf-objdump --demangle -d ${KERNEL} > ${KERNEL}.asm

clean:
	-cargo clean