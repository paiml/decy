/* K&R C Chapter 7: Stream Redirection
 * K&R §7.5: Redirecting stdin/stdout/stderr
 * Tests stream redirection and duplication
 */

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <fcntl.h>

/* Redirect stdout to file */
int redirect_stdout(const char *filename) {
    int fd = open(filename, O_WRONLY | O_CREAT | O_TRUNC, 0644);
    if (fd == -1) {
        perror("open");
        return -1;
    }

    /* Save original stdout */
    int stdout_save = dup(STDOUT_FILENO);
    if (stdout_save == -1) {
        perror("dup");
        close(fd);
        return -1;
    }

    /* Redirect stdout to file */
    if (dup2(fd, STDOUT_FILENO) == -1) {
        perror("dup2");
        close(fd);
        close(stdout_save);
        return -1;
    }

    close(fd);
    return stdout_save;
}

/* Restore stdout */
void restore_stdout(int saved_fd) {
    fflush(stdout);
    dup2(saved_fd, STDOUT_FILENO);
    close(saved_fd);
}

/* Redirect stderr to file */
int redirect_stderr(const char *filename) {
    int fd = open(filename, O_WRONLY | O_CREAT | O_TRUNC, 0644);
    if (fd == -1) {
        perror("open");
        return -1;
    }

    int stderr_save = dup(STDERR_FILENO);
    if (stderr_save == -1) {
        perror("dup");
        close(fd);
        return -1;
    }

    if (dup2(fd, STDERR_FILENO) == -1) {
        perror("dup2");
        close(fd);
        close(stderr_save);
        return -1;
    }

    close(fd);
    return stderr_save;
}

/* Restore stderr */
void restore_stderr(int saved_fd) {
    fflush(stderr);
    dup2(saved_fd, STDERR_FILENO);
    close(saved_fd);
}

/* Redirect both stdout and stderr to same file */
void redirect_both_to_file(const char *filename) {
    freopen(filename, "w", stdout);
    freopen(filename, "w", stderr);
}

/* Redirect stdout to stderr */
void redirect_stdout_to_stderr(void) {
    dup2(STDERR_FILENO, STDOUT_FILENO);
}

/* Redirect stderr to stdout */
void redirect_stderr_to_stdout(void) {
    dup2(STDOUT_FILENO, STDERR_FILENO);
}

/* Test stdout redirection */
void test_stdout_redirection(void) {
    printf("=== Stdout Redirection Test ===\n");
    printf("This is printed to console (before redirection)\n\n");

    /* Redirect stdout to file */
    int saved_stdout = redirect_stdout("stdout_redirect.txt");
    if (saved_stdout == -1) {
        fprintf(stderr, "Failed to redirect stdout\n");
        return;
    }

    /* These go to file */
    printf("Line 1: This is redirected to file\n");
    printf("Line 2: More redirected output\n");
    printf("Line 3: Yet another line\n");

    /* Restore stdout */
    restore_stdout(saved_stdout);

    /* Back to console */
    printf("This is printed to console (after restoration)\n");

    /* Show file contents */
    printf("File contents:\n");
    FILE *fp = fopen("stdout_redirect.txt", "r");
    char buffer[100];
    while (fgets(buffer, sizeof(buffer), fp) != NULL) {
        printf("  %s", buffer);
    }
    fclose(fp);
    printf("\n");
}

/* Test stderr redirection */
void test_stderr_redirection(void) {
    printf("=== Stderr Redirection Test ===\n");
    fprintf(stderr, "This error goes to console (before redirection)\n\n");

    /* Redirect stderr to file */
    int saved_stderr = redirect_stderr("stderr_redirect.txt");
    if (saved_stderr == -1) {
        fprintf(stderr, "Failed to redirect stderr\n");
        return;
    }

    /* These go to file */
    fprintf(stderr, "Error 1: Redirected error message\n");
    fprintf(stderr, "Error 2: Another error\n");
    fprintf(stderr, "Error 3: Final error\n");

    /* Restore stderr */
    restore_stderr(saved_stderr);

    /* Back to console */
    fprintf(stderr, "This error goes to console (after restoration)\n");

    /* Show file contents */
    printf("Error file contents:\n");
    FILE *fp = fopen("stderr_redirect.txt", "r");
    char buffer[100];
    while (fgets(buffer, sizeof(buffer), fp) != NULL) {
        printf("  %s", buffer);
    }
    fclose(fp);
    printf("\n");
}

/* Test freopen */
void test_freopen(void) {
    printf("=== freopen Test ===\n");
    printf("Before freopen\n");

    /* Redirect stdout using freopen */
    FILE *fp = freopen("freopen_test.txt", "w", stdout);
    if (fp == NULL) {
        fprintf(stderr, "freopen failed\n");
        return;
    }

    printf("This goes to file via freopen\n");
    printf("Another line via freopen\n");

    /* Restore stdout to terminal */
    freopen("/dev/tty", "w", stdout);

    printf("Back to terminal\n");

    /* Show file contents */
    printf("File contents:\n");
    fp = fopen("freopen_test.txt", "r");
    char buffer[100];
    while (fgets(buffer, sizeof(buffer), fp) != NULL) {
        printf("  %s", buffer);
    }
    fclose(fp);
    printf("\n");
}

/* Test stream duplication */
void test_stream_duplication(void) {
    printf("=== Stream Duplication Test ===\n");

    /* Create file */
    int fd = open("dup_test.txt", O_WRONLY | O_CREAT | O_TRUNC, 0644);

    /* Duplicate fd multiple times */
    int fd2 = dup(fd);
    int fd3 = dup(fd);

    printf("Original fd: %d\n", fd);
    printf("Duplicated fd2: %d\n", fd2);
    printf("Duplicated fd3: %d\n", fd3);

    /* Write through different fds */
    write(fd, "Line from fd\n", 13);
    write(fd2, "Line from fd2\n", 14);
    write(fd3, "Line from fd3\n", 14);

    close(fd);
    close(fd2);
    close(fd3);

    /* Show file contents */
    printf("File contents:\n");
    FILE *fp = fopen("dup_test.txt", "r");
    char buffer[100];
    while (fgets(buffer, sizeof(buffer), fp) != NULL) {
        printf("  %s", buffer);
    }
    fclose(fp);
    printf("\n");
}

/* Test pipe with redirection */
void test_pipe_redirection(void) {
    printf("=== Pipe Redirection Test ===\n");

    int pipefd[2];
    if (pipe(pipefd) == -1) {
        perror("pipe");
        return;
    }

    pid_t pid = fork();

    if (pid == 0) {
        /* Child: redirect stdout to pipe write end */
        close(pipefd[0]);
        dup2(pipefd[1], STDOUT_FILENO);
        close(pipefd[1]);

        /* This goes to pipe */
        printf("Message from child process\n");
        exit(0);
    } else {
        /* Parent: read from pipe */
        close(pipefd[1]);

        char buffer[100];
        ssize_t n = read(pipefd[0], buffer, sizeof(buffer) - 1);
        if (n > 0) {
            buffer[n] = '\0';
            printf("Parent received: %s", buffer);
        }

        close(pipefd[0]);
        wait(NULL);
    }
    printf("\n");
}

int main() {
    printf("=== Stream Redirection ===\n\n");

    test_stdout_redirection();
    test_stderr_redirection();
    test_freopen();
    test_stream_duplication();
    test_pipe_redirection();

    /* Cleanup */
    remove("stdout_redirect.txt");
    remove("stderr_redirect.txt");
    remove("freopen_test.txt");
    remove("dup_test.txt");

    printf("Stream redirection techniques:\n");
    printf("  - dup/dup2: Low-level file descriptor duplication\n");
    printf("  - freopen: High-level stream redirection\n");
    printf("  - Pipes: Inter-process communication\n");
    printf("  - Useful for logging, testing, and IPC\n");

    return 0;
}
