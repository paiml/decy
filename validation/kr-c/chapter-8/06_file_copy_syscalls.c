/* K&R C Chapter 8.2: File Copy Using System Calls
 * Page 171-172
 * Efficient file copying with system calls
 */

#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>

#define BUFSIZE 8192

int main(int argc, char *argv[]) {
    int fd_in, fd_out;
    char buf[BUFSIZE];
    ssize_t n;

    if (argc != 3) {
        fprintf(stderr, "Usage: %s <source> <destination>\n", argv[0]);
        return 1;
    }

    /* Open source file */
    fd_in = open(argv[1], O_RDONLY);
    if (fd_in == -1) {
        perror("Cannot open source file");
        return 2;
    }

    /* Create destination file */
    fd_out = creat(argv[2], 0644);
    if (fd_out == -1) {
        perror("Cannot create destination file");
        close(fd_in);
        return 3;
    }

    /* Copy data */
    while ((n = read(fd_in, buf, BUFSIZE)) > 0) {
        if (write(fd_out, buf, n) != n) {
            perror("Write error");
            close(fd_in);
            close(fd_out);
            return 4;
        }
    }

    if (n == -1) {
        perror("Read error");
    }

    close(fd_in);
    close(fd_out);

    printf("File copied successfully\n");

    return 0;
}
