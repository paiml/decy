/* K&R C Chapter 8.6: Directory Listing
 * Page 179-184
 * Simple directory listing with opendir/readdir
 */

#include <stdio.h>
#include <dirent.h>
#include <sys/types.h>

int main(int argc, char *argv[]) {
    DIR *dp;
    struct dirent *entry;
    const char *dirname;

    dirname = (argc > 1) ? argv[1] : ".";

    dp = opendir(dirname);
    if (dp == NULL) {
        perror("Cannot open directory");
        return 1;
    }

    printf("Contents of %s:\n", dirname);

    while ((entry = readdir(dp)) != NULL) {
        printf("  %s (inode: %ld)\n", entry->d_name, (long)entry->d_ino);
    }

    closedir(dp);

    return 0;
}
