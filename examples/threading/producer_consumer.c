// Producer-consumer pattern with pthreads
// Tests: threading, mutexes, condition variables

#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#define BUFFER_SIZE 10

typedef struct {
    int buffer[BUFFER_SIZE];
    int count;
    int in;
    int out;
    pthread_mutex_t mutex;
    pthread_cond_t not_empty;
    pthread_cond_t not_full;
} Queue;

void queue_init(Queue* q) {
    q->count = 0;
    q->in = 0;
    q->out = 0;
    pthread_mutex_init(&q->mutex, NULL);
    pthread_cond_init(&q->not_empty, NULL);
    pthread_cond_init(&q->not_full, NULL);
}

void queue_put(Queue* q, int item) {
    pthread_mutex_lock(&q->mutex);

    while (q->count == BUFFER_SIZE) {
        pthread_cond_wait(&q->not_full, &q->mutex);
    }

    q->buffer[q->in] = item;
    q->in = (q->in + 1) % BUFFER_SIZE;
    q->count++;

    pthread_cond_signal(&q->not_empty);
    pthread_mutex_unlock(&q->mutex);
}

int queue_get(Queue* q) {
    pthread_mutex_lock(&q->mutex);

    while (q->count == 0) {
        pthread_cond_wait(&q->not_empty, &q->mutex);
    }

    int item = q->buffer[q->out];
    q->out = (q->out + 1) % BUFFER_SIZE;
    q->count--;

    pthread_cond_signal(&q->not_full);
    pthread_mutex_unlock(&q->mutex);

    return item;
}

void* producer(void* arg) {
    Queue* q = (Queue*)arg;

    for (int i = 0; i < 20; i++) {
        queue_put(q, i);
        printf("Produced: %d\n", i);
        usleep(100000);
    }

    return NULL;
}

void* consumer(void* arg) {
    Queue* q = (Queue*)arg;

    for (int i = 0; i < 20; i++) {
        int item = queue_get(q);
        printf("Consumed: %d\n", item);
        usleep(150000);
    }

    return NULL;
}

int main(void) {
    Queue q;
    queue_init(&q);

    pthread_t producer_thread, consumer_thread;

    pthread_create(&producer_thread, NULL, producer, &q);
    pthread_create(&consumer_thread, NULL, consumer, &q);

    pthread_join(producer_thread, NULL);
    pthread_join(consumer_thread, NULL);

    pthread_mutex_destroy(&q.mutex);
    pthread_cond_destroy(&q.not_empty);
    pthread_cond_destroy(&q.not_full);

    return 0;
}
