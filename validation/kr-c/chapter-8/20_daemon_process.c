/* K&R C Chapter 8: Daemon Process Creation
 * K&R §8.7: Creating background daemon processes
 * Tests daemon initialization
 */

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>

void daemonize(void) {
    /* Fork and exit parent */
    pid_t pid = fork();
    if (pid < 0) {
        exit(EXIT_FAILURE);
    }
    if (pid > 0) {
        exit(EXIT_SUCCESS);  /* Parent exits */
    }

    /* Create new session */
    if (setsid() < 0) {
        exit(EXIT_FAILURE);
    }

    /* Fork again to prevent reacquiring terminal */
    pid = fork();
    if (pid < 0) {
        exit(EXIT_FAILURE);
    }
    if (pid > 0) {
        exit(EXIT_SUCCESS);
    }

    /* Change working directory */
    chdir("/");

    /* Close all file descriptors */
    for (int i = 0; i < sysconf(_SC_OPEN_MAX); i++) {
        close(i);
    }

    /* Redirect stdio to /dev/null */
    int fd = open("/dev/null", O_RDWR);
    dup2(fd, STDIN_FILENO);
    dup2(fd, STDOUT_FILENO);
    dup2(fd, STDERR_FILENO);
    if (fd > 2) {
        close(fd);
    }

    /* Reset file mode mask */
    umask(0);
}

void write_log(const char *message) {
    FILE *log = fopen("/tmp/daemon_test.log", "a");
    if (log) {
        fprintf(log, "%s\n", message);
        fclose(log);
    }
}

int main(int argc, char *argv[]) {
    if (argc > 1 && strcmp(argv[1], "--daemon") == 0) {
        printf("Starting daemon process...\n");

        daemonize();

        /* Daemon code */
        write_log("Daemon started");

        for (int i = 0; i < 5; i++) {
            sleep(1);
            char msg[50];
            snprintf(msg, sizeof(msg), "Daemon iteration %d", i);
            write_log(msg);
        }

        write_log("Daemon stopped");
        return 0;
    } else {
        printf("=== Daemon Process Creation ===\n\n");
        printf("Daemon characteristics:\n");
        printf("  1. Run in background\n");
        printf("  2. Detached from terminal\n");
        printf("  3. Parent is init (PID 1)\n");
        printf("  4. No controlling terminal\n");
        printf("  5. Changed working directory to /\n");
        printf("  6. Closed inherited file descriptors\n");
        printf("  7. Reset file mode mask\n");
        printf("\nDaemonization steps:\n");
        printf("  1. fork() and exit parent\n");
        printf("  2. setsid() to create new session\n");
        printf("  3. fork() again\n");
        printf("  4. chdir(\"/\")\n");
        printf("  5. Close all file descriptors\n");
        printf("  6. Redirect stdio to /dev/null\n");
        printf("  7. umask(0)\n");
        printf("\nTo test daemon: %s --daemon\n", argv[0]);
        printf("Check log: cat /tmp/daemon_test.log\n");

        return 0;
    }
}
