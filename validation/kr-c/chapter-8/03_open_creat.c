/* K&R C Chapter 8.3: Open, Creat, Close, Unlink
 * Page 172-173
 * File creation and manipulation
 */

#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>
#include <string.h>

int main() {
    int fd;
    char *message = "Hello from creat!\n";

    /* Create new file (or truncate existing) */
    fd = creat("newfile.txt", 0644);
    if (fd == -1) {
        perror("creat");
        return 1;
    }

    /* Write to file */
    write(fd, message, strlen(message));
    close(fd);

    printf("Created and wrote to newfile.txt\n");

    /* Open existing file for append */
    fd = open("newfile.txt", O_WRONLY | O_APPEND);
    if (fd != -1) {
        write(fd, "Appended line\n", 14);
        close(fd);
        printf("Appended to newfile.txt\n");
    }

    /* Remove file */
    if (unlink("newfile.txt") == 0)
        printf("Deleted newfile.txt\n");

    return 0;
}
