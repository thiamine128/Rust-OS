INCLUDES    := -I../include -Iinclude

tools_dir   := ../tools

include include.mk

.PRECIOUS: %.b %.b.rs

%.b.rs: %.b
	$(tools_dir)/bintorust -f $< -o $@ -p USER
	cat $@ >> ../src/env/bare.rs

%.b: %.o $(USERLIB)
	$(LD) -o $@ $(LDFLAGS) -T ./user.lds $^

%.o: %.c
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

%.o: %.S
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

%.o: lib.h

.PHONY: all clean

all: clean $(INITAPPS) $(USERAPPS) $(USERLIB)
	

clean:
	rm -rf *~ *.o *.b.c *.x *.b lib/*.o
