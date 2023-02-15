ARCH ?= aarch64
MACHINE ?= shyper
PROFILE ?= release

export RUSTFLAGS := ${RUSTFLAGS} -C force-frame-pointers=yes

# Cargo flags.
ifeq (${PROFILE}, release)
CARGO_FLAGS = --release
else
CARGO_FLAGS = 
endif

# Target directory.
KERNEL := target/${ARCH}-unknown-${MACHINE}/${PROFILE}/hello_world

# Arch-specific tools
OBJCOPY := aarch64-elf-objcopy
OBJDUMP := aarch64-elf-objdump

.PHONY: build clean emu

build:
	cargo +stage1 build -Zbuild-std=std,panic_unwind --target aarch64-unknown-shyper ${CARGO_FLAGS} 
	${OBJCOPY} ${KERNEL} -O binary ${KERNEL}.bin
	${OBJDUMP} --demangle -d ${KERNEL} > ${KERNEL}.asm

QEMU_COMMON_OPTIONS := -serial stdio -display none -smp 4 -m 2048

emu: build
	sudo qemu-system-aarch64 -M virt -cpu cortex-a53 \
		-device loader,file=${KERNEL},addr=0x80000000,force-raw=on \
		${QEMU_COMMON_OPTIONS} \
		-kernel ${KERNEL}.bin -s

debug: build
	sudo qemu-system-aarch64 -M virt -cpu cortex-a53 \
		-device loader,file=${KERNEL},addr=0x80000000,force-raw=on \
		${QEMU_COMMON_OPTIONS} \
		-kernel ${KERNEL}.bin -s -S

clean:
	-cargo clean