ARCH ?= aarch64
USER_PROFILE ?= release
MACHINE ?= shyper

ifeq (${USER_PROFILE}, release)
CARGO_FLAGS = --release
else
CARGO_FLAGS =
endif

KERNEL := target/${ARCH}${MACHINE}/${USER_PROFILE}/user

.PHONY: build clean

build:
	cargo build --target ${ARCH}${MACHINE}.json -Z build-std=core,alloc ${CARGO_FLAGS}
	aarch64-elf-objcopy ${KERNEL} -O binary ${KERNEL}.bin
	aarch64-elf-objdump --demangle -d ${KERNEL} > ${KERNEL}.asm

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

disk:
	rm -rf disk
	dd if=/dev/zero of=disk.img bs=4096 count=92160 2>/dev/null
	mkfs.fat -F 32 disk.img

clean:
	-cargo clean