#include <lib.h>
#include <shm.h>
#include <sem.h>

int main() {
    int* addr = 0x12000;
    sem_open(1, 1);
    int r = shmget(1, 1024);
    if (r < 0) {
        user_panic("shmget error: %d\n", r);
    }
    int id = r;
    r = shmat(id, addr, PTE_V | PTE_D);
    if (r < 0) {
        user_panic("shmat error: %d\n", r);
    }
    int *a = addr;
    int *b = addr + 1;
    for (int i = 0; i < 10; ++i) {
        sem_wait(1);
        *a += 1;
        syscall_yield();
        *b += 1;
        if (*a != *b) {
            user_panic("not sync %d %d\n", *a, *b);
        }
        sem_post(1);
        //debugf("env[%x]: b in shm: %d\n", env->env_id, *b);
        
    }
    shmctl(id, SHM_RMID);
    shmdt(id, addr);
    return 0;
}