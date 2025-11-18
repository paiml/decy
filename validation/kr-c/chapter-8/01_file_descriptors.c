/* K&R C Chapter 8.1: File Descriptors
 * Page 169-170
 * Basic file descriptor operations
 */

#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>

int main() {
    int fd;
    char buffer[100];
    ssize_t n;

    /* Standard file descriptors are predefined */
    printf("STDIN: %d\n", STDIN_FILENO);
    printf("STDOUT: %d\n", STDOUT_FILENO);
    printf("STDERR: %d\n", STDERR_FILENO);

    /* Open a file (creates file descriptor) */
    fd = open("test.txt", O_RDWR | O_CREAT, 0644);
    if (fd == -1) {
        perror("open");
        return 1;
    }

    printf("Opened file descriptor: %d\n", fd);

    /* Write to file */
    write(fd, "Hello, world!\n", 14);

    /* Close and reopen to read */
    close(fd);

    fd = open("test.txt", O_RDONLY);
    n = read(fd, buffer, sizeof(buffer) - 1);
    buffer[n] = '\0';

    printf("Read from file: %s", buffer);

    close(fd);

    return 0;
}
