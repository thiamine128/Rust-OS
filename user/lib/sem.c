#include <env.h>
#include <lib.h>
#include <sem.h>

void sem_open(int id, int v) {
    syscall_semopen(id, v);
}

void sem_wait(int id) {
    while (syscall_semwait(id) != 0) {
        
    }
}

void sem_post(int id) {
    syscall_sempost(id);
}

void sem_kill(int id) {
    syscall_semkill(id);
}