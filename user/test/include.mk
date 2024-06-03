root_dir    :=  ../../..
tools_dir   :=  $(root_dir)/tools
user_dir    :=  $(root_dir)/user
fs_dir      :=  $(root_dir)/fs
lib_dir     :=  $(root_dir)/lib
INCLUDES    +=  -I$(root_dir)/include -I$(user_dir)/include

%.b.rs: %.b
	$(tools_dir)/bintorust -f $< -o $@ -p TEST

%.b: SHELL := /bin/bash
%.b: %.o $(libs)
	shopt -s nullglob && $(LD) -o $@ $(LDFLAGS) -T $(root_dir)/user/user.lds $^ \
	$(user_dir)/lib/*.o $(fs_dir)/*.o $(lib_dir)/*.o

%.o: %.c
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

%.o: %.S
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

.PHONY: all clean embed

all: $(targets) embed

clean::
	rm -rf *.o *~ *.x *.b *.b.c

$(user_dir):
	$(MAKE) --directory=$(user_dir)