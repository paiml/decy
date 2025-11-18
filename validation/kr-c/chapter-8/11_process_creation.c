/* K&R C Chapter 8: Process Creation and Management
 * K&R §8.6: fork, exec, wait
 * Tests process creation and execution
 */

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/wait.h>

void demo_fork(void) {
    printf("=== fork() Demo ===\n");

    pid_t pid = fork();

    if (pid < 0) {
        perror("fork failed");
        return;
    } else if (pid == 0) {
        /* Child process */
        printf("  Child: PID=%d, Parent PID=%d\n", getpid(), getppid());
        exit(0);
    } else {
        /* Parent process */
        printf("  Parent: PID=%d, Child PID=%d\n", getpid(), pid);
        wait(NULL);
    }

    printf("\n");
}

void demo_exec(void) {
    printf("=== exec() Demo ===\n");

    pid_t pid = fork();

    if (pid == 0) {
        /* Child: Replace with ls command */
        printf("  Child: Executing 'ls -l'\n");
        execlp("ls", "ls", "-l", "/tmp", NULL);
        perror("exec failed");
        exit(1);
    } else {
        wait(NULL);
    }

    printf("\n");
}

void demo_wait_status(void) {
    printf("=== wait() Status Demo ===\n");

    pid_t pid = fork();

    if (pid == 0) {
        printf("  Child: Exiting with status 42\n");
        exit(42);
    } else {
        int status;
        wait(&status);

        if (WIFEXITED(status)) {
            printf("  Parent: Child exited with status %d\n", WEXITSTATUS(status));
        }
    }

    printf("\n");
}

int main() {
    printf("=== Process Creation ===\n\n");

    demo_fork();
    demo_exec();
    demo_wait_status();

    printf("Process management:\n");
    printf("  - fork(): Create child process\n");
    printf("  - exec(): Replace process image\n");
    printf("  - wait(): Wait for child termination\n");

    return 0;
}
