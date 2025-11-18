/* K&R C Chapter 7: Directory Operations
 * K&R §8.6: Directory operations (POSIX extensions)
 * Tests directory listing, traversal, and filtering
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <dirent.h>
#include <sys/stat.h>
#include <sys/types.h>

/* List directory contents */
void list_directory(const char *path) {
    DIR *dir = opendir(path);
    if (dir == NULL) {
        perror("opendir");
        return;
    }

    printf("Contents of directory: %s\n", path);

    struct dirent *entry;
    int count = 0;

    while ((entry = readdir(dir)) != NULL) {
        count++;
        printf("  [%d] %s", count, entry->d_name);

        /* Show file type if available */
        if (entry->d_type == DT_DIR)
            printf(" (directory)");
        else if (entry->d_type == DT_REG)
            printf(" (file)");
        else if (entry->d_type == DT_LNK)
            printf(" (symlink)");

        printf("\n");
    }

    printf("Total entries: %d\n", count);
    closedir(dir);
}

/* Count files and directories */
void count_entries(const char *path) {
    DIR *dir = opendir(path);
    if (dir == NULL) {
        perror("opendir");
        return;
    }

    int files = 0, dirs = 0, links = 0, other = 0;
    struct dirent *entry;

    while ((entry = readdir(dir)) != NULL) {
        /* Skip . and .. */
        if (strcmp(entry->d_name, ".") == 0 || strcmp(entry->d_name, "..") == 0)
            continue;

        if (entry->d_type == DT_DIR)
            dirs++;
        else if (entry->d_type == DT_REG)
            files++;
        else if (entry->d_type == DT_LNK)
            links++;
        else
            other++;
    }

    printf("Directory statistics for: %s\n", path);
    printf("  Files:       %d\n", files);
    printf("  Directories: %d\n", dirs);
    printf("  Symlinks:    %d\n", links);
    printf("  Other:       %d\n", other);
    printf("  Total:       %d\n", files + dirs + links + other);

    closedir(dir);
}

/* Filter files by extension */
void list_files_by_extension(const char *path, const char *ext) {
    DIR *dir = opendir(path);
    if (dir == NULL) {
        perror("opendir");
        return;
    }

    printf("Files with extension '.%s' in %s:\n", ext, path);

    struct dirent *entry;
    int count = 0;

    while ((entry = readdir(dir)) != NULL) {
        /* Check if regular file */
        if (entry->d_type != DT_REG && entry->d_type != DT_UNKNOWN)
            continue;

        /* Check extension */
        const char *dot = strrchr(entry->d_name, '.');
        if (dot && strcmp(dot + 1, ext) == 0) {
            count++;
            printf("  [%d] %s\n", count, entry->d_name);
        }
    }

    printf("Found %d files\n", count);
    closedir(dir);
}

/* Get directory size (recursive) */
long get_directory_size(const char *path) {
    DIR *dir = opendir(path);
    if (dir == NULL) {
        return 0;
    }

    long total_size = 0;
    struct dirent *entry;
    char full_path[1024];

    while ((entry = readdir(dir)) != NULL) {
        /* Skip . and .. */
        if (strcmp(entry->d_name, ".") == 0 || strcmp(entry->d_name, "..") == 0)
            continue;

        /* Build full path */
        snprintf(full_path, sizeof(full_path), "%s/%s", path, entry->d_name);

        struct stat st;
        if (stat(full_path, &st) == 0) {
            if (S_ISREG(st.st_mode)) {
                total_size += st.st_size;
            } else if (S_ISDIR(st.st_mode)) {
                /* Recursive call for subdirectories */
                total_size += get_directory_size(full_path);
            }
        }
    }

    closedir(dir);
    return total_size;
}

/* Find files matching pattern */
void find_files(const char *path, const char *pattern) {
    DIR *dir = opendir(path);
    if (dir == NULL) {
        perror("opendir");
        return;
    }

    printf("Files matching '%s' in %s:\n", pattern, path);

    struct dirent *entry;
    int count = 0;

    while ((entry = readdir(dir)) != NULL) {
        /* Simple substring match */
        if (strstr(entry->d_name, pattern) != NULL) {
            count++;
            printf("  [%d] %s\n", count, entry->d_name);
        }
    }

    printf("Found %d matches\n", count);
    closedir(dir);
}

/* List directory sorted by name */
void list_directory_sorted(const char *path) {
    DIR *dir = opendir(path);
    if (dir == NULL) {
        perror("opendir");
        return;
    }

    /* Count entries first */
    int count = 0;
    struct dirent *entry;
    while ((entry = readdir(dir)) != NULL) {
        count++;
    }

    /* Allocate array */
    char **names = malloc(count * sizeof(char *));
    if (names == NULL) {
        closedir(dir);
        return;
    }

    /* Read names */
    rewinddir(dir);
    int i = 0;
    while ((entry = readdir(dir)) != NULL) {
        names[i] = strdup(entry->d_name);
        i++;
    }

    /* Simple bubble sort */
    for (int j = 0; j < count - 1; j++) {
        for (int k = 0; k < count - j - 1; k++) {
            if (strcmp(names[k], names[k + 1]) > 0) {
                char *temp = names[k];
                names[k] = names[k + 1];
                names[k + 1] = temp;
            }
        }
    }

    /* Print sorted */
    printf("Directory %s (sorted):\n", path);
    for (int j = 0; j < count; j++) {
        printf("  [%d] %s\n", j + 1, names[j]);
        free(names[j]);
    }

    free(names);
    closedir(dir);
}

/* Check if directory is empty */
int is_directory_empty(const char *path) {
    DIR *dir = opendir(path);
    if (dir == NULL) {
        return -1;
    }

    struct dirent *entry;
    int count = 0;

    while ((entry = readdir(dir)) != NULL) {
        /* Skip . and .. */
        if (strcmp(entry->d_name, ".") != 0 && strcmp(entry->d_name, "..") != 0) {
            count++;
            break;  /* Found at least one entry */
        }
    }

    closedir(dir);
    return count == 0;
}

/* Recursive directory listing */
void list_directory_recursive(const char *path, int depth) {
    DIR *dir = opendir(path);
    if (dir == NULL) {
        return;
    }

    struct dirent *entry;
    char full_path[1024];

    while ((entry = readdir(dir)) != NULL) {
        /* Skip . and .. */
        if (strcmp(entry->d_name, ".") == 0 || strcmp(entry->d_name, "..") == 0)
            continue;

        /* Print with indentation */
        for (int i = 0; i < depth; i++)
            printf("  ");
        printf("%s", entry->d_name);

        if (entry->d_type == DT_DIR) {
            printf("/\n");

            /* Recurse into subdirectory */
            snprintf(full_path, sizeof(full_path), "%s/%s", path, entry->d_name);
            list_directory_recursive(full_path, depth + 1);
        } else {
            printf("\n");
        }
    }

    closedir(dir);
}

int main() {
    printf("=== Directory Operations ===\n\n");

    /* Create test directory structure */
    system("mkdir -p test_dir/subdir1 test_dir/subdir2");
    system("touch test_dir/file1.txt test_dir/file2.c test_dir/file3.txt");
    system("touch test_dir/subdir1/nested.txt");

    /* List directory */
    printf("Basic directory listing:\n");
    list_directory("test_dir");
    printf("\n");

    /* Count entries */
    printf("Directory statistics:\n");
    count_entries("test_dir");
    printf("\n");

    /* Filter by extension */
    printf("Filter by extension:\n");
    list_files_by_extension("test_dir", "txt");
    printf("\n");

    /* Find files */
    printf("Find files:\n");
    find_files("test_dir", "file");
    printf("\n");

    /* Sorted listing */
    printf("Sorted listing:\n");
    list_directory_sorted("test_dir");
    printf("\n");

    /* Check if empty */
    printf("Empty check:\n");
    printf("  test_dir: %s\n", is_directory_empty("test_dir") ? "EMPTY" : "NOT EMPTY");
    system("mkdir -p test_dir/empty_dir");
    printf("  test_dir/empty_dir: %s\n", is_directory_empty("test_dir/empty_dir") ? "EMPTY" : "NOT EMPTY");
    printf("\n");

    /* Recursive listing */
    printf("Recursive listing:\n");
    list_directory_recursive("test_dir", 0);
    printf("\n");

    /* Directory size */
    printf("Directory size:\n");
    long size = get_directory_size("test_dir");
    printf("  Total size: %ld bytes\n", size);

    /* Cleanup */
    system("rm -rf test_dir");

    printf("\nDirectory operations:\n");
    printf("  - opendir/readdir/closedir for traversal\n");
    printf("  - struct dirent provides entry info\n");
    printf("  - Use stat() for detailed file info\n");
    printf("  - Recursive traversal requires careful path handling\n");

    return 0;
}
