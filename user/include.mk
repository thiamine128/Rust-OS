INITAPPS             := tltest.b.rs fktest.b.rs pingpong.b.rs

USERLIB              := entry.o \
			syscall_wrap.o \
			debugf.o \
			libos.o \
			fork.o \
			syscall_lib.o \
			ipc.o \
			shm.o \
			sem.o
			

INITAPPS     += devtst.b.rs fstest.b.rs
USERLIB      += fd.o \
		pageref.o \
		file.o \
		fsipc.o \
		console.o \
		fprintf.o

INITAPPS     += icode.b.rs \
			testpipe.b.rs \
			testpiperace.b.rs \
			testptelibrary.b.rs

USERLIB      += wait.o spawn.o pipe.o
USERAPPS     := num.b  \
		echo.b \
		halt.b \
		ls.b \
		sh.b  \
		cat.b \
		testpipe.b \
		testpiperace.b \
		testptelibrary.b \
		testarg.b \
		testbss.b \
		testfdsharing.b \
		pingpong.b \
		init.b

USERLIB := $(addprefix lib/, $(USERLIB)) $(wildcard ../lib/*.o)