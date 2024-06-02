INITAPPS             := tltest.x fktest.x pingpong.x

USERLIB              := entry.o \
			syscall_wrap.o \
			debugf.o \
			libos.o \
			fork.o \
			syscall_lib.o \
			ipc.o \
			shm.o \
			sem.o

INITAPPS     += devtst.x fstest.x
USERLIB      += fd.o \
		pageref.o \
		file.o \
		fsipc.o \
		console.o \
		fprintf.o

USERLIB := $(addprefix lib/, $(USERLIB)) $(wildcard ../lib/*.o)