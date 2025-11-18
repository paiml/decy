/* K&R C Chapter 8.4: Random Access - lseek
 * Page 174-175
 * File positioning with lseek
 */

#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>
#include <string.h>

int main() {
    int fd;
    char buffer[20];
    off_t pos;

    /* Create file with some content */
    fd = creat("seektest.txt", 0644);
    write(fd, "0123456789ABCDEFGHIJ", 20);
    close(fd);

    /* Open for reading */
    fd = open("seektest.txt", O_RDONLY);

    /* Read from beginning */
    read(fd, buffer, 5);
    buffer[5] = '\0';
    printf("First 5 bytes: %s\n", buffer);

    /* Seek to position 10 */
    pos = lseek(fd, 10, SEEK_SET);
    printf("Seeked to position: %ld\n", (long)pos);

    read(fd, buffer, 5);
    buffer[5] = '\0';
    printf("Next 5 bytes: %s\n", buffer);

    /* Seek to end */
    pos = lseek(fd, 0, SEEK_END);
    printf("File size: %ld bytes\n", (long)pos);

    /* Seek backward */
    lseek(fd, -5, SEEK_END);
    read(fd, buffer, 5);
    buffer[5] = '\0';
    printf("Last 5 bytes: %s\n", buffer);

    close(fd);
    unlink("seektest.txt");

    return 0;
}
