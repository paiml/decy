/* K&R C Chapter 8: Recursive Directory Traversal
 * K&R §8.6: opendir, readdir, stat
 * Tests recursive file tree walking
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <dirent.h>
#include <sys/stat.h>

void print_indent(int depth) {
    for (int i = 0; i < depth; i++) {
        printf("  ");
    }
}

void traverse_directory(const char *path, int depth) {
    DIR *dir = opendir(path);
    if (dir == NULL) {
        return;
    }

    struct dirent *entry;
    while ((entry = readdir(dir)) != NULL) {
        if (strcmp(entry->d_name, ".") == 0 || strcmp(entry->d_name, "..") == 0) {
            continue;
        }

        char full_path[1024];
        snprintf(full_path, sizeof(full_path), "%s/%s", path, entry->d_name);

        struct stat st;
        if (stat(full_path, &st) == 0) {
            print_indent(depth);

            if (S_ISDIR(st.st_mode)) {
                printf("[DIR]  %s/\n", entry->d_name);
                if (depth < 3) {  /* Limit recursion depth */
                    traverse_directory(full_path, depth + 1);
                }
            } else {
                printf("[FILE] %s (%ld bytes)\n", entry->d_name, st.st_size);
            }
        }
    }

    closedir(dir);
}

int main() {
    printf("=== Recursive Directory Traversal ===\n\n");

    const char *path = "/tmp";
    printf("Traversing: %s\n", path);
    traverse_directory(path, 0);

    printf("\nDirectory traversal:\n");
    printf("  - opendir(): Open directory\n");
    printf("  - readdir(): Read entries\n");
    printf("  - stat(): Get file info\n");
    printf("  - Recursive for subdirectories\n");

    return 0;
}
