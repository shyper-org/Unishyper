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
KERNEL := target/${ARCH}-unknown-${MACHINE}/${PROFILE}/httpd

# Arch-specific tools
OBJCOPY := aarch64-elf-objcopy
OBJDUMP := aarch64-elf-objdump

.PHONY: build clean emu

build:
	cargo +stage1 build -Zbuild-std=std,panic_unwind --target aarch64-unknown-shyper ${CARGO_FLAGS} 
	${OBJCOPY} ${KERNEL} -O binary ${KERNEL}.bin
	${OBJDUMP} --demangle -d ${KERNEL} > ${KERNEL}.asm

QEMU_DISK_OPTIONS := -drive file=disk.img,if=none,format=raw,id=x0 \
					 -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
					 -global virtio-mmio.force-legacy=false
QEMU_NETWORK_OPTIONS := -netdev tap,id=tap0,ifname=tap0,script=no,downscript=no \
						-device virtio-net-device,mac=48:b0:2d:0e:6e:9e,netdev=tap0 \
						-global virtio-mmio.force-legacy=false
QEMU_COMMON_OPTIONS := -serial stdio -display none -smp 4 -m 2048

emu: build disk
	sudo qemu-system-aarch64 -M virt -cpu cortex-a53 \
		-device loader,file=${KERNEL},addr=0x80000000,force-raw=on \
		${QEMU_COMMON_OPTIONS} \
		${QEMU_DISK_OPTIONS} \
		${QEMU_NETWORK_OPTIONS} \
		-kernel ${KERNEL}.bin -s

# Set up tap device as qemu netdev.
tap_setup:
	sudo ip tuntap add tap0 mode tap
	sudo ip addr add 10.0.0.1/24 broadcast 10.0.0.255 dev tap0
	sudo ip link set dev tap0 up
	sudo bash -c 'echo 1 > /proc/sys/net/ipv4/conf/tap0/proxy_arp'

# Prepare disk.img for qemu.
disk:
ifeq ($(wildcard disk.img),)
	rm -rf disk
	dd if=/dev/zero of=disk.img bs=4096 count=92160 2>/dev/null
	mkfs.fat -F 32 disk.img
endif

clean:
	-cargo clean