/* K&R C Chapter 8.6: Example - Listing Directories
 * Page 179-184
 * File information with stat
 */

#include <stdio.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <unistd.h>
#include <time.h>

void print_file_info(const char *filename) {
    struct stat st;

    if (stat(filename, &st) == -1) {
        perror("stat");
        return;
    }

    printf("File: %s\n", filename);
    printf("Size: %ld bytes\n", (long)st.st_size);
    printf("Inode: %ld\n", (long)st.st_ino);
    printf("Mode: %o\n", st.st_mode & 0777);
    printf("Links: %ld\n", (long)st.st_nlink);

    printf("Type: ");
    if (S_ISREG(st.st_mode))
        printf("Regular file\n");
    else if (S_ISDIR(st.st_mode))
        printf("Directory\n");
    else if (S_ISLNK(st.st_mode))
        printf("Symbolic link\n");
    else
        printf("Other\n");

    printf("Last modified: %s", ctime(&st.st_mtime));
    printf("\n");
}

int main(int argc, char *argv[]) {
    if (argc < 2) {
        fprintf(stderr, "Usage: %s <file1> [file2 ...]\n", argv[0]);
        return 1;
    }

    for (int i = 1; i < argc; i++) {
        print_file_info(argv[i]);
    }

    return 0;
}
