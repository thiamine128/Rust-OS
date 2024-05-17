include include.mk

export CC CFLAGS LD LDFLAGS
target_dir := os_target
mos_elf := $(target_dir)/mos
user_disk := $(target_dir)/fs.img
empty_disk := $(target_dir)/empty.img

QEMU_FLAGS += -cpu 24Kc -m 64 -nographic -M malta \
	$(shell [ -f '$(user_disk)' ] && echo '-drive id=ide0,file=$(user_disk),if=ide,format=raw') \
	$(shell [ -f '$(empty_disk)' ] && echo '-drive id=ide1,file=$(empty_disk),if=ide,format=raw') \
	-no-reboot


.PHONEY: all run ASM kern users fs-image

all: kern

kern: ASM users
	cargo build --release
	cp target/mipsel-unknown-none/release/mos_rust $(mos_elf)

ASM:
	$(CC) $(CFLAGS) -E src/init/start.S -o src/init/start.gen.S -I./include4asm
	$(CC) $(CFLAGS) -E src/memory/tlb_asm.S -o src/memory/tlb_asm.gen.S -I./include4asm
	$(CC) $(CFLAGS) -E src/exception/entry.S -o src/exception/entry.gen.S -I./include4asm
	$(CC) $(CFLAGS) -E src/exception/genex.S -o src/exception/genex.gen.S -I./include4asm
	$(CC) $(CFLAGS) -E src/env/env_asm.S -o src/env/env_asm.gen.S -I./include4asm

clean:
	rm -rf target
run:
	$(QEMU) $(QEMU_FLAGS) -kernel $(mos_elf)

dbg:
	$(QEMU) $(QEMU_FLAGS) -s -S -kernel $(mos_elf)

gdb:
	gdb-multiarch \
	-q os_target/mos \
    -ex 'target remote localhost:1234'

doc:
	cargo doc --bin mos_rust

users:
	echo '' > src/env/bare.rs
	$(MAKE) -C lib
	$(MAKE) -C fs
	$(MAKE) -C user

fs-files := user/test/fs_strong_check/rootfs/*

fs-image:
	$(MAKE) -C fs image fs-files="$(addprefix ../, $(fs-files))"