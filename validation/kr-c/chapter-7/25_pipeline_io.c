/* K&R C Chapter 7: Pipeline and Pipe I/O
 * K&R §8.7: Inter-process communication via pipes
 * Tests pipe creation and data flow between processes
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/wait.h>

/* Simple pipe example */
void simple_pipe_demo(void) {
    printf("=== Simple Pipe Demo ===\n");

    int pipefd[2];
    if (pipe(pipefd) == -1) {
        perror("pipe");
        return;
    }

    printf("Pipe created: read_fd=%d, write_fd=%d\n", pipefd[0], pipefd[1]);

    pid_t pid = fork();

    if (pid == -1) {
        perror("fork");
        return;
    }

    if (pid == 0) {
        /* Child process - writer */
        close(pipefd[0]);  /* Close unused read end */

        const char *message = "Hello from child process!";
        write(pipefd[1], message, strlen(message));
        printf("  Child: Wrote message\n");

        close(pipefd[1]);
        exit(0);
    } else {
        /* Parent process - reader */
        close(pipefd[1]);  /* Close unused write end */

        char buffer[100];
        ssize_t n = read(pipefd[0], buffer, sizeof(buffer) - 1);
        if (n > 0) {
            buffer[n] = '\0';
            printf("  Parent: Read '%s'\n", buffer);
        }

        close(pipefd[0]);
        wait(NULL);
    }
    printf("\n");
}

/* Bidirectional communication with two pipes */
void bidirectional_pipe(void) {
    printf("=== Bidirectional Pipe Demo ===\n");

    int pipe1[2], pipe2[2];

    if (pipe(pipe1) == -1 || pipe(pipe2) == -1) {
        perror("pipe");
        return;
    }

    pid_t pid = fork();

    if (pid == 0) {
        /* Child */
        close(pipe1[1]);  /* Close write end of pipe1 */
        close(pipe2[0]);  /* Close read end of pipe2 */

        /* Read from parent */
        char buffer[100];
        read(pipe1[0], buffer, sizeof(buffer));
        printf("  Child received: %s\n", buffer);

        /* Send response */
        const char *response = "Response from child";
        write(pipe2[1], response, strlen(response));

        close(pipe1[0]);
        close(pipe2[1]);
        exit(0);
    } else {
        /* Parent */
        close(pipe1[0]);  /* Close read end of pipe1 */
        close(pipe2[1]);  /* Close write end of pipe2 */

        /* Send to child */
        const char *message = "Message from parent";
        write(pipe1[1], message, strlen(message));
        printf("  Parent sent: %s\n", message);

        /* Read response */
        char buffer[100];
        read(pipe2[0], buffer, sizeof(buffer) - 1);
        buffer[strlen("Response from child")] = '\0';
        printf("  Parent received: %s\n", buffer);

        close(pipe1[1]);
        close(pipe2[0]);
        wait(NULL);
    }
    printf("\n");
}

/* Pipeline: chaining multiple processes */
void pipeline_demo(void) {
    printf("=== Pipeline Demo (3 processes) ===\n");

    int pipe1[2], pipe2[2];

    if (pipe(pipe1) == -1 || pipe(pipe2) == -1) {
        perror("pipe");
        return;
    }

    /* First process - generator */
    pid_t pid1 = fork();
    if (pid1 == 0) {
        close(pipe1[0]);
        dup2(pipe1[1], STDOUT_FILENO);
        close(pipe1[1]);

        for (int i = 1; i <= 5; i++) {
            printf("%d ", i);
        }
        exit(0);
    }

    /* Second process - transformer */
    pid_t pid2 = fork();
    if (pid2 == 0) {
        close(pipe1[1]);
        close(pipe2[0]);
        dup2(pipe1[0], STDIN_FILENO);
        dup2(pipe2[1], STDOUT_FILENO);
        close(pipe1[0]);
        close(pipe2[1]);

        int num;
        while (scanf("%d", &num) == 1) {
            printf("%d ", num * 2);
        }
        exit(0);
    }

    /* Parent - consumer */
    close(pipe1[0]);
    close(pipe1[1]);
    close(pipe2[1]);

    printf("Pipeline output: ");
    char buffer[100];
    ssize_t n = read(pipe2[0], buffer, sizeof(buffer) - 1);
    if (n > 0) {
        buffer[n] = '\0';
        printf("%s\n", buffer);
    }

    close(pipe2[0]);
    wait(NULL);
    wait(NULL);
    printf("\n");
}

/* Producer-consumer with pipe */
void producer_consumer(void) {
    printf("=== Producer-Consumer with Pipe ===\n");

    int pipefd[2];
    pipe(pipefd);

    pid_t pid = fork();

    if (pid == 0) {
        /* Consumer */
        close(pipefd[1]);

        FILE *fp = fdopen(pipefd[0], "r");
        char buffer[100];

        printf("  Consumer: Reading items...\n");
        while (fgets(buffer, sizeof(buffer), fp) != NULL) {
            printf("    Consumed: %s", buffer);
        }

        fclose(fp);
        exit(0);
    } else {
        /* Producer */
        close(pipefd[0]);

        FILE *fp = fdopen(pipefd[1], "w");

        printf("  Producer: Generating items...\n");
        for (int i = 1; i <= 5; i++) {
            fprintf(fp, "Item %d\n", i);
            fflush(fp);
            usleep(100000);  /* 0.1 second delay */
        }

        fclose(fp);
        wait(NULL);
    }
    printf("\n");
}

/* Named pipe (FIFO) simulation */
void named_pipe_demo(void) {
    printf("=== Named Pipe (FIFO) Demo ===\n");

    const char *fifo_path = "/tmp/test_fifo";

    /* Create FIFO */
    unlink(fifo_path);  /* Remove if exists */
    if (mkfifo(fifo_path, 0666) == -1) {
        perror("mkfifo");
        return;
    }

    printf("Created FIFO: %s\n", fifo_path);

    pid_t pid = fork();

    if (pid == 0) {
        /* Child - reader */
        printf("  Child: Opening FIFO for reading...\n");
        FILE *fp = fopen(fifo_path, "r");
        if (fp == NULL) {
            perror("fopen");
            exit(1);
        }

        char buffer[100];
        fgets(buffer, sizeof(buffer), fp);
        printf("  Child: Read '%s'\n", buffer);

        fclose(fp);
        exit(0);
    } else {
        /* Parent - writer */
        sleep(1);  /* Give child time to open */

        printf("  Parent: Opening FIFO for writing...\n");
        FILE *fp = fopen(fifo_path, "w");
        if (fp == NULL) {
            perror("fopen");
            return;
        }

        fprintf(fp, "Message through FIFO\n");
        printf("  Parent: Wrote message\n");

        fclose(fp);
        wait(NULL);
    }

    unlink(fifo_path);
    printf("\n");
}

/* Pipe capacity test */
void pipe_capacity_test(void) {
    printf("=== Pipe Capacity Test ===\n");

    int pipefd[2];
    pipe(pipefd);

    pid_t pid = fork();

    if (pid == 0) {
        /* Child - slow reader */
        close(pipefd[1]);
        sleep(2);  /* Delay before reading */

        char buffer[1000];
        int total = 0;
        ssize_t n;

        while ((n = read(pipefd[0], buffer, sizeof(buffer))) > 0) {
            total += n;
        }

        printf("  Child: Read %d bytes total\n", total);
        close(pipefd[0]);
        exit(0);
    } else {
        /* Parent - fast writer */
        close(pipefd[0]);

        char data[1000];
        memset(data, 'A', sizeof(data));

        printf("  Parent: Writing data...\n");
        int written = 0;
        for (int i = 0; i < 100; i++) {
            ssize_t n = write(pipefd[1], data, sizeof(data));
            if (n > 0) {
                written += n;
            }
        }

        printf("  Parent: Wrote %d bytes\n", written);
        close(pipefd[1]);
        wait(NULL);
    }
    printf("\n");
}

int main() {
    printf("=== Pipeline and Pipe I/O ===\n\n");

    simple_pipe_demo();
    bidirectional_pipe();
    pipeline_demo();
    producer_consumer();
    named_pipe_demo();
    pipe_capacity_test();

    printf("Pipe I/O benefits:\n");
    printf("  - Efficient inter-process communication\n");
    printf("  - No intermediate files needed\n");
    printf("  - Supports streaming data\n");
    printf("  - Unidirectional by default\n");
    printf("  - Kernel-buffered (typically 64KB)\n");

    return 0;
}
