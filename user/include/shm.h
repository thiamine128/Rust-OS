#ifndef SHM_H
#define SHM_H

enum {
	SHM_RMID = 1,
};

int shmget(int, int);
int shmat(int, void*, u_int);
int shmdt(int, void*);
int shmctl(int, u_int);
#endif