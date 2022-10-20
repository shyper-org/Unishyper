ARCH ?= aarch64
PROFILE ?= release
MACHINE ?= qemu

# Cargo flags.
ifeq (${PROFILE}, release)
CARGO_FLAGS = --release  --no-default-features
else
CARGO_FLAGS =  --no-default-features
endif

CARGO_FLAGS := ${CARGO_FLAGS} --features "${MACHINE}"

# Target directory.
TARGET_CFG := ../../cfg/${ARCH}${MACHINE}.json
KERNEL := target/${ARCH}${MACHINE}/${PROFILE}/user

# Arch-specific tools
OBJCOPY := aarch64-elf-objcopy
OBJDUMP := aarch64-elf-objdump

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

# Use tftp to load unikernel binary file to target location on Tegra X2 ROM.
tx2: build
	mkimage -n unishyper -A arm64 -O linux -C none -T kernel -a 0x80080000 -e 0x80080000 -d ${KERNEL}.bin ${KERNEL}.ubi
	cp ${KERNEL} /tftp/
	cp ${KERNEL}.ubi /tftp/
	echo "tftp 0xc0000000 192.168.106.140:user; tftp 0x8a000000 192.168.106.140:user.ubi; bootm start 0x8a000000 - 0x80000000; bootm loados; bootm go"

tx2_153: build
	mkimage -n unishyper -A arm64 -O linux -C none -T kernel -a 0x80080000 -e 0x80080000 -d ${KERNEL}.bin ${KERNEL}.ubi
	scp ${KERNEL} root@192.168.106.153:/tftp
	scp ${KERNEL}.ubi root@192.168.106.153:/tftp
	echo "tftp 0xc0000000 192.168.106.153:user; tftp 0x8a000000 192.168.106.153:user.ubi; bootm start 0x8a000000 - 0x80000000; bootm loados; bootm go"