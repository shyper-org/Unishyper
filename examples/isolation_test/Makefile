ARCH ?= aarch64
PROFILE ?= release
MACHINE ?= qemu

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
KERNEL_DIR := target/${ARCH}${MACHINE}/${PROFILE}
KERNEL := ${KERNEL_DIR}/isolation_test
RBOOT_DIR := ../../rboot
ESP := $(KERNEL_DIR)/esp
OVMF := ${RBOOT_DIR}/OVMF.fd

# Arch-specific tools
ifeq ($(ARCH), aarch64)
OBJCOPY := rust-objcopy
OBJDUMP := rust-objdump
else ifeq ($(ARCH), x86_64)
OBJCOPY := objcopy
OBJDUMP := objdump
endif

.PHONY: build clean

bootloader:
ifeq ($(ARCH), x86_64)
	cd ${RBOOT_DIR} && make build
endif

build: bootloader
	cargo build --target ${TARGET_CFG} -Z build-std=core,alloc ${CARGO_FLAGS} 
	${OBJCOPY} ${KERNEL} -O binary ${KERNEL}.bin
	${OBJDUMP} --demangle -d ${KERNEL} > ${KERNEL}.asm
ifeq ($(ARCH), x86_64)
	mkdir -p $(ESP)/EFI/Demo $(ESP)/EFI/Boot
	cp ${RBOOT_DIR}/target/x86_64-unknown-uefi/$(PROFILE)/rboot.efi $(ESP)/EFI/Boot/BootX64.efi
	cp ${RBOOT_DIR}/rboot.conf $(ESP)/EFI/Boot/rboot.conf
	cp $(KERNEL) $(ESP)/EFI/Demo/kernel.elf
endif

ifeq ($(ARCH), aarch64)
QEMU_CMD := qemu-system-aarch64 -M virt -cpu cortex-a53 -smp 4 -m 2048 \
			-device loader,file=${KERNEL},addr=0x80000000,force-raw=on -kernel ${KERNEL}.bin
else ifeq ($(ARCH), x86_64)
QEMU_CMD := qemu-system-x86_64 \
			-cpu qemu64,apic,fsgsbase,fxsr,rdrand,rdtscp,xsave,xsaveopt,pku \
			-smp 4 -m 4G \
			-drive if=pflash,format=raw,readonly,file=$(OVMF) \
			-drive format=raw,file=fat:rw:$(ESP) \
			-device isa-debug-exit,iobase=0xf4,iosize=0x04
endif

QEMU_DISK_OPTIONS := -drive file=disk.img,if=none,format=raw,id=x0 \
					 -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
					 -global virtio-mmio.force-legacy=false
QEMU_NETWORK_OPTIONS := -netdev tap,id=tap0,ifname=tap0,script=no,downscript=no \
						-device virtio-net-device,mac=48:b0:2d:0e:6e:9e,netdev=tap0 \
						-global virtio-mmio.force-legacy=false
QEMU_COMMON_OPTIONS := -serial stdio -display none -s

emu: build
	${QEMU_CMD} ${QEMU_COMMON_OPTIONS}

debug: build
	${QEMU_CMD} ${QEMU_COMMON_OPTIONS} -S

disk:
ifeq ($(wildcard disk.img),)
	rm -rf disk
	dd if=/dev/zero of=disk.img bs=4096 count=92160 2>/dev/null
	mkfs.fat -F 32 disk.img
endif

clean:
	-cargo clean