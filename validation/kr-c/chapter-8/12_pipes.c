/* K&R C Chapter 8: Inter-Process Communication with Pipes
 * K&R §8.6: pipe() system call
 * Tests pipe-based IPC between processes
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/wait.h>

void demo_simple_pipe(void) {
    printf("=== Simple Pipe Demo ===\n");

    int pipefd[2];
    pipe(pipefd);

    pid_t pid = fork();

    if (pid == 0) {
        /* Child: Reader */
        close(pipefd[1]);  /* Close write end */

        char buffer[100];
        ssize_t n = read(pipefd[0], buffer, sizeof(buffer) - 1);
        buffer[n] = '\0';

        printf("  Child received: %s\n", buffer);

        close(pipefd[0]);
        exit(0);
    } else {
        /* Parent: Writer */
        close(pipefd[0]);  /* Close read end */

        const char *msg = "Hello from parent!";
        write(pipefd[1], msg, strlen(msg));
        printf("  Parent sent: %s\n", msg);

        close(pipefd[1]);
        wait(NULL);
    }

    printf("\n");
}

void demo_pipe_exec(void) {
    printf("=== Pipe with exec() Demo ===\n");

    int pipefd[2];
    pipe(pipefd);

    pid_t pid = fork();

    if (pid == 0) {
        /* Child: Execute 'wc -l' */
        close(pipefd[1]);
        dup2(pipefd[0], STDIN_FILENO);
        close(pipefd[0]);

        execlp("wc", "wc", "-l", NULL);
        perror("exec failed");
        exit(1);
    } else {
        /* Parent: Write lines */
        close(pipefd[0]);

        const char *lines = "Line 1\nLine 2\nLine 3\n";
        write(pipefd[1], lines, strlen(lines));

        close(pipefd[1]);
        wait(NULL);
    }

    printf("\n");
}

int main() {
    printf("=== Pipes (IPC) ===\n\n");

    demo_simple_pipe();
    demo_pipe_exec();

    printf("Pipes:\n");
    printf("  - Unidirectional byte stream\n");
    printf("  - Connects processes\n");
    printf("  - Used for command pipelines\n");

    return 0;
}
