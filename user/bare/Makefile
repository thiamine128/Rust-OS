tools_dir   := ../../tools
INCLUDES    := -I../../include

.PRECIOUS: %.b %.b.c


%.b.rs: %.b
	$(tools_dir)/bintorust -f $< -o $@ -p user_bare
%.b: %.o
	$(LD) -o $@ $(LDFLAGS) -T ../user.lds $^

%.o: %.c
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

%.o: %.S
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

.PHONY: all clean

all: loop.b.rs overflow.b.rs put_a.b.rs put_b.b.rs
	cat loop.b.rs >> ../../src/env/bare.rs
	cat overflow.b.rs >> ../../src/env/bare.rs
	cat put_a.b.rs >> ../../src/env/bare.rs
	cat put_b.b.rs >> ../../src/env/bare.rs

clean:
	rm -rf *.o *~ *.b *.b.rs