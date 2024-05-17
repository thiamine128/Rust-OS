INCLUDES    := -I../include -Iinclude

tools_dir   := ../tools

include include.mk

%.b.rs: %.b
	$(tools_dir)/bintorust -f $< -o $@ -p test
%.b: entry.o %.o
	$(LD) -o $@ $(LDFLAGS) -T ../user.lds $^

%.o: %.c
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

%.o: %.S
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

%.o: lib.h

.PHONY: all clean

all: clean $(USERLIB)
	

clean:
	rm -rf *~ *.o *.b.c *.x *.b lib/*.o
