ARCH ?= aarch64
MACHINE ?= qemu
PROFILE ?= release
KVM ?= true

# Cargo flags.
ifeq (${PROFILE}, release)
CARGO_FLAGS = --release
else
CARGO_FLAGS = 
endif

# Target directory.
KERNEL_DIR := ../../target/${ARCH}-unknown-shyper/${PROFILE}
KERNEL := ${KERNEL_DIR}/httpd

RBOOT_DIR := ../../rboot
ESP := $(KERNEL_DIR)/esp
OVMF := ${RBOOT_DIR}/OVMF.fd

# Arch-specific tools
OBJCOPY := rust-objcopy
OBJDUMP := rust-objdump

.PHONY: build clean emu

bootloader:
ifeq ($(ARCH), x86_64)
	cd ${RBOOT_DIR} && make build
endif

build: bootloader
	cargo +shyper build -Zbuild-std=std,panic_unwind --target ${ARCH}-unknown-shyper ${CARGO_FLAGS} 
	${OBJCOPY} ${KERNEL} -O binary ${KERNEL}.bin
	${OBJDUMP} --demangle -d ${KERNEL} > ${KERNEL}.asm
ifeq ($(ARCH), x86_64)
	mkdir -p $(ESP)/EFI/Demo $(ESP)/EFI/Boot
	cp ${RBOOT_DIR}/target/x86_64-unknown-uefi/$(PROFILE)/rboot.efi $(ESP)/EFI/Boot/BootX64.efi
	cp ${RBOOT_DIR}/rboot.conf $(ESP)/EFI/Boot/rboot.conf
	cp ${KERNEL} $(ESP)/EFI/Demo/kernel.elf
endif

QEMU_NETWORK_OPTIONS := -netdev tap,id=tap0,ifname=tap0,script=no,downscript=no \
						-device virtio-net-device,mac=48:b0:2d:0e:6e:9e,netdev=tap0 \
						-global virtio-mmio.force-legacy=false
QEMU_COMMON_OPTIONS := -serial stdio -display none -smp 4 -m 2048

QEMU_CMD := qemu-system-$(ARCH)

## Set machine type.

ifeq ($(ARCH), aarch64)
QEMU_CMD := ${QEMU_CMD} -M virt -cpu cortex-a53 -smp 4 -m 2048
endif

ifeq ($(ARCH), riscv64)
QEMU_CMD := ${QEMU_CMD} -M virt -bios default -smp 4 -m 2048 
endif

ifeq ($(ARCH), x86_64)
ifeq ($(KVM), true)
QEMU_CMD := ${QEMU_CMD} -cpu host -enable-kvm -smp 1 -m 4G
else
QEMU_CMD := ${QEMU_CMD} -cpu qemu64,apic,fsgsbase,fxsr,rdrand,rdtscp,xsave,xsaveopt,pku -smp 1 -m 4G
endif
endif

## Set loader device.
ifeq ($(ARCH), aarch64)
QEMU_CMD := ${QEMU_CMD} -device loader,file=${KERNEL},addr=0x80000000,force-raw=on -kernel ${KERNEL}.bin
endif
ifeq ($(ARCH), riscv64)
QEMU_CMD := ${QEMU_CMD} -device loader,file=${KERNEL},addr=0xc0000000,force-raw=on -kernel ${KERNEL}.bin
endif
ifeq ($(ARCH), x86_64)
QEMU_CMD := ${QEMU_CMD} \
			-drive if=pflash,format=raw,readonly=on,file=$(OVMF) \
			-drive format=raw,file=fat:rw:$(ESP) \
			-device isa-debug-exit,iobase=0xf4,iosize=0x04
endif

ifneq ($(ARCH), x86_64)
QEMU_NETWORK_OPTIONS := -netdev tap,id=tap0,ifname=tap0,script=no,downscript=no \
						-device virtio-net-device,mac=48:b0:2d:0e:6e:9e,netdev=tap0 \
						-global virtio-mmio.force-legacy=false
endif
ifeq ($(ARCH), x86_64)
QEMU_NETWORK_OPTIONS := -netdev tap,id=tap0,ifname=tap0,script=no,downscript=no,vhost=on \
						-device virtio-net-pci,netdev=tap0,disable-legacy=on,mac=48:b0:2d:0e:6e:9e
endif

QEMU_COMMON_OPTIONS := -serial stdio -display none -s

emu: build
	sudo ${QEMU_CMD} \
		${QEMU_COMMON_OPTIONS} \
		${QEMU_NETWORK_OPTIONS}

debug: build
	sudo ${QEMU_CMD} \
		${QEMU_COMMON_OPTIONS} \
		${QEMU_NETWORK_OPTIONS} -S

# Set up tap device as qemu netdev.
tap_setup:
	sudo ip tuntap add tap0 mode tap
	sudo ip addr add 10.0.0.1/24 broadcast 10.0.0.255 dev tap0
	sudo ip link set dev tap0 up
	sudo bash -c 'echo 1 > /proc/sys/net/ipv4/conf/tap0/proxy_arp'

clean:
	-cargo clean