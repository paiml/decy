/* K&R C Chapter 8: File Descriptor Duplication
 * Related to page 176
 * dup and dup2 for file descriptor manipulation
 */

#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>
#include <string.h>

int main() {
    int fd, fd_dup;
    char *msg1 = "Original fd\n";
    char *msg2 = "Duplicated fd\n";

    /* Create a file */
    fd = creat("dup_test.txt", 0644);
    if (fd == -1) {
        perror("creat");
        return 1;
    }

    /* Write with original fd */
    write(fd, msg1, strlen(msg1));

    /* Duplicate file descriptor */
    fd_dup = dup(fd);
    printf("Original fd: %d, Duplicated fd: %d\n", fd, fd_dup);

    /* Write with duplicated fd */
    write(fd_dup, msg2, strlen(msg2));

    /* Both point to same file - close one */
    close(fd);

    /* Can still write with duplicate */
    write(fd_dup, "Still works!\n", 13);

    close(fd_dup);

    /* Demonstrate dup2 - redirect stdout to file */
    fd = creat("redirect_test.txt", 0644);

    /* Save original stdout */
    int stdout_copy = dup(STDOUT_FILENO);

    /* Redirect stdout to file */
    dup2(fd, STDOUT_FILENO);

    printf("This goes to redirect_test.txt\n");
    printf("Not visible on screen!\n");

    /* Restore stdout */
    dup2(stdout_copy, STDOUT_FILENO);

    printf("Back to normal stdout\n");

    close(fd);
    close(stdout_copy);

    /* Cleanup */
    unlink("dup_test.txt");
    unlink("redirect_test.txt");

    return 0;
}
