/* K&R C Chapter 8.2: Low Level I/O - Read and Write
 * Page 170-171
 * Using read() and write() system calls
 */

#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>

#define BUFSIZE 4096

int main(int argc, char *argv[]) {
    int fd;
    char buf[BUFSIZE];
    ssize_t n;

    if (argc < 2) {
        fprintf(stderr, "Usage: %s <filename>\n", argv[0]);
        return 1;
    }

    /* Open file for reading */
    fd = open(argv[1], O_RDONLY);
    if (fd == -1) {
        perror("Cannot open file");
        return 2;
    }

    /* Copy file to stdout */
    while ((n = read(fd, buf, BUFSIZE)) > 0)
        write(STDOUT_FILENO, buf, n);

    close(fd);

    return 0;
}
