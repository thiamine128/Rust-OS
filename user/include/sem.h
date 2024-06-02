#ifndef SEM_H
#define SEM_H

void sem_open(int id, int v);

void sem_wait(int id);

void sem_post(int id);

void sem_kill(int id);

#endif