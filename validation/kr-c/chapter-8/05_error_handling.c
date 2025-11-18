/* K&R C Chapter 8.5: Example - Implementation of fopen and getc
 * Page 176-177
 * Error handling with system calls
 */

#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>
#include <errno.h>
#include <string.h>

int main() {
    int fd;

    /* Try to open non-existent file */
    fd = open("nonexistent.txt", O_RDONLY);
    if (fd == -1) {
        printf("Error number: %d\n", errno);
        printf("Error message: %s\n", strerror(errno));
        perror("open");
    }

    /* Try to open with invalid flags */
    fd = open("/etc/passwd", O_RDONLY);
    if (fd != -1) {
        printf("Opened /etc/passwd successfully (fd=%d)\n", fd);

        /* Try to write to read-only file */
        if (write(fd, "test", 4) == -1) {
            printf("Cannot write to read-only file\n");
            perror("write");
        }

        close(fd);
    }

    /* Try to create file in non-existent directory */
    fd = creat("/nonexistent/dir/file.txt", 0644);
    if (fd == -1) {
        printf("Cannot create file in non-existent directory\n");
        perror("creat");
    }

    return 0;
}
