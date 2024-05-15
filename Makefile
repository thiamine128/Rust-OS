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


.PHONEY: all run ASM

all: ASM
	cargo build --release
	cp target/mipsel-unknown-none/release/mos_rust $(mos_elf)

ASM:
	gcc -E src/init/start.S -o src/init/start.gen.S -I./include
	gcc -E src/memory/tlb_asm.S -o src/memory/tlb_asm.gen.S -I./include
	gcc -E src/exception/entry.S -o src/exception/entry.gen.S -I./include
	gcc -E src/exception/genex.S -o src/exception/genex.gen.S -I./include
	gcc -E src/env/env_asm.S -o src/env/env_asm.gen.S -I./include

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
	$(MAKE) -C user/bare