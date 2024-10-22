# Utils
OBJCOPY := rust-objcopy
OBJDUMP := rust-objdump

# Rust flags, for unwind.
export RUSTFLAGS := ${RUSTFLAGS} -C force-frame-pointers=yes -C link-arg=-T$(LD_SCRIPT)

# Cargo flags.
ifeq (${PROFILE}, release)
CARGO_FLAGS = --release  --no-default-features
else
CARGO_FLAGS =  --no-default-features
endif

ifneq ($(TOOLCHAIN),)
TOOLCHAIN := +${TOOLCHAIN}
ifeq ($(ARCH), riscv64)
TARGET_DESC := ${ARCH}gc-unknown-shyper
else
TARGET_DESC := ${ARCH}-unknown-shyper
endif
TARGET_CFG := ${TARGET_DESC}
CARGO_FLAGS := ${CARGO_FLAGS} \
	-Z build-std=std,panic_unwind
else
TARGET_DESC := ${ARCH}
TARGET_CFG := $(CURDIR)/cfg/${TARGET_DESC}.json
CARGO_FLAGS := ${CARGO_FLAGS} \
	-Z build-std=core,alloc \
	-Z build-std-features=compiler-builtins-mem
endif

TARGET_DIR := $(CURDIR)/target

APP_DIR := $(APP)

OUT_DIR := ${TARGET_DIR}/${TARGET_DESC}/${PROFILE}
BUILD_ELF := ${OUT_DIR}/${APP_BIN}

OUT_APP := ${APP_DIR}/$(APP_BIN)_${TARGET_DESC}_${PROFILE}
OUT_ELF := ${OUT_APP}.elf
OUT_BIN := ${OUT_APP}.bin
OUT_ASM := ${OUT_APP}.asm

CARGO_ACTION ?= build

CARGO_ARGS := \
	--manifest-path ${APP_DIR}/Cargo.toml \
	--bin ${APP_BIN} \
	--target ${TARGET_CFG} \
	--target-dir ${TARGET_DIR} \
	${CARGO_FLAGS}

ifeq ($(filter $(ARCH),aarch64 x86_64 riscv64),)
  $(error "ARCH" must be one of "aarch64", "x86_64", "riscv64")
endif

ifeq ($(filter $(LOG),off error warn info debug trace),)
  $(error "LOG" must be one of "off", "error", "warn", "info", "debug", "trace")
endif

ifeq ($(filter $(BUS),mmio pci),)
  $(error "BUS" must be one of "mmio", "pci")
endif

FEATURES := ${MACHINE}, unishyper/log-level-${LOG}, unishyper/$(BUS)

# Currently we use [rboot](https://github.com/hky1999/rboot.git) as bootloader in x86_64.
ifeq ($(ARCH), x86_64)
RBOOT_DIR := $(CURDIR)/x86boot
OVMF := ${RBOOT_DIR}/OVMF.fd
endif

define rboot_pre
	mkdir -p $(RBOOT_DIR)/EFI/Demo
	cp $(OUT_ELF) $(RBOOT_DIR)/EFI/Demo/kernel.elf
endef

define cargo_build
	cargo ${TOOLCHAIN} ${CARGO_ACTION} $(CARGO_ARGS) --features "${FEATURES}"
	cp $(BUILD_ELF) $(OUT_ELF)
	${OBJCOPY} ${OUT_ELF} -O binary ${OUT_BIN}
	${OBJDUMP} --demangle -d ${OUT_ELF} > ${OUT_ASM}
endef
