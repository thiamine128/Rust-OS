tools_dir   := ../../tools
INCLUDES    := -I../../include

.PRECIOUS: %.b %.b.c

%.b.rs: %.b
	$(tools_dir)/bintorust -f $< -o $@ -p test
%.b: entry.o %.o
	$(LD) -o $@ $(LDFLAGS) -T ../user.lds $^

%.o: %.c
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

%.o: %.S
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

.PHONY: all clean

all: quicksort.b.rs
	cat quicksort.b.rs >> ../../src/env/bare.rs

clean:
	rm -rf *.o *~ *.b *.b.rs