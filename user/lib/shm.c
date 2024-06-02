#include <env.h>
#include <lib.h>
#include <shm.h>

int shmget(int key, int size) {
    return syscall_shmget(key, size);
}

int shmat(int id, void* addr, u_int perm) {
    return syscall_shmat(id, addr, perm);
}

int shmdt(int id, void* addr) {
    return syscall_shmdt(id, addr);
}

int shmctl(int id, u_int ctl) {
    return syscall_shmctl(id, ctl);
}