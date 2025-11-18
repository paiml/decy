/* K&R C Chapter 7: File Path Manipulation
 * K&R §8.6: Path operations and manipulation
 * Tests file path parsing and manipulation
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <libgen.h>
#include <unistd.h>
#include <limits.h>

/* Extract directory name from path */
void extract_dirname(const char *path) {
    char *path_copy = strdup(path);
    char *dir = dirname(path_copy);

    printf("Path: %s\n", path);
    printf("  Directory: %s\n", dir);

    free(path_copy);
}

/* Extract filename from path */
void extract_basename(const char *path) {
    char *path_copy = strdup(path);
    char *base = basename(path_copy);

    printf("Path: %s\n", path);
    printf("  Basename: %s\n", base);

    free(path_copy);
}

/* Extract file extension */
const char *get_extension(const char *filename) {
    const char *dot = strrchr(filename, '.');
    if (dot && dot != filename && *(dot + 1) != '\0') {
        return dot + 1;
    }
    return "";
}

/* Remove extension from filename */
void remove_extension(char *filename) {
    char *dot = strrchr(filename, '.');
    if (dot && dot != filename) {
        *dot = '\0';
    }
}

/* Join path components */
void join_path(char *result, size_t size, const char *dir, const char *file) {
    snprintf(result, size, "%s/%s", dir, file);
}

/* Normalize path (remove . and ..) */
void normalize_path(char *path) {
    char *components[100];
    int count = 0;

    /* Split by / */
    char *token = strtok(path, "/");
    while (token != NULL) {
        if (strcmp(token, ".") == 0) {
            /* Skip current directory */
        } else if (strcmp(token, "..") == 0) {
            /* Go up one level */
            if (count > 0) {
                count--;
            }
        } else {
            /* Regular component */
            components[count++] = token;
        }
        token = strtok(NULL, "/");
    }

    /* Rebuild path */
    path[0] = '\0';
    for (int i = 0; i < count; i++) {
        if (i > 0) {
            strcat(path, "/");
        }
        strcat(path, components[i]);
    }
}

/* Check if path is absolute */
int is_absolute_path(const char *path) {
    return path[0] == '/';
}

/* Check if path is relative */
int is_relative_path(const char *path) {
    return path[0] != '/';
}

/* Get absolute path */
void get_absolute_path(const char *relative_path) {
    char absolute[PATH_MAX];

    if (realpath(relative_path, absolute) != NULL) {
        printf("Relative: %s\n", relative_path);
        printf("Absolute: %s\n", absolute);
    } else {
        perror("realpath");
    }
}

/* Split path into components */
void split_path(const char *path) {
    printf("Path: %s\n", path);
    printf("  Components:\n");

    char *path_copy = strdup(path);
    char *token = strtok(path_copy, "/");
    int index = 0;

    while (token != NULL) {
        printf("    [%d] %s\n", index++, token);
        token = strtok(NULL, "/");
    }

    free(path_copy);
}

/* Build path from components */
void build_path_from_components(void) {
    const char *components[] = {"home", "user", "documents", "file.txt"};
    int count = 4;
    char path[PATH_MAX] = "";

    printf("Building path from components:\n");
    for (int i = 0; i < count; i++) {
        printf("  [%d] %s\n", i, components[i]);
        if (i > 0) {
            strcat(path, "/");
        }
        strcat(path, components[i]);
    }

    printf("  Result: %s\n", path);
}

/* Change file extension */
void change_extension(char *filename, const char *new_ext) {
    char *dot = strrchr(filename, '.');
    if (dot) {
        *dot = '\0';
    }
    strcat(filename, ".");
    strcat(filename, new_ext);
}

/* Path comparison */
void compare_paths(const char *path1, const char *path2) {
    printf("Comparing paths:\n");
    printf("  Path 1: %s\n", path1);
    printf("  Path 2: %s\n", path2);

    if (strcmp(path1, path2) == 0) {
        printf("  Result: Identical\n");
    } else {
        printf("  Result: Different\n");
    }
}

/* Relative path between two paths */
void compute_relative_path(const char *from, const char *to) {
    printf("From: %s\n", from);
    printf("To:   %s\n", to);

    /* Simple implementation - find common prefix */
    int i = 0;
    while (from[i] && to[i] && from[i] == to[i]) {
        i++;
    }

    /* Count remaining directories in 'from' */
    int up_count = 0;
    for (int j = i; from[j]; j++) {
        if (from[j] == '/') {
            up_count++;
        }
    }

    printf("  Relative: ");
    for (int j = 0; j < up_count; j++) {
        printf("../");
    }
    printf("%s\n", to + i + 1);
}

/* File path validation */
int validate_path(const char *path) {
    printf("Validating path: %s\n", path);

    /* Check for null or empty */
    if (path == NULL || path[0] == '\0') {
        printf("  Invalid: Empty path\n");
        return 0;
    }

    /* Check for invalid characters (platform-specific) */
    if (strstr(path, "//") != NULL) {
        printf("  Warning: Double slashes\n");
    }

    /* Check length */
    if (strlen(path) > PATH_MAX) {
        printf("  Invalid: Path too long\n");
        return 0;
    }

    printf("  Valid\n");
    return 1;
}

int main() {
    printf("=== File Path Manipulation ===\n\n");

    /* Extract directory and basename */
    printf("Directory and basename extraction:\n");
    extract_dirname("/home/user/documents/file.txt");
    extract_basename("/home/user/documents/file.txt");
    printf("\n");

    /* Extension operations */
    printf("Extension operations:\n");
    const char *test_files[] = {
        "document.txt",
        "program.c",
        "archive.tar.gz",
        "README",
        ".hidden"
    };

    for (int i = 0; i < 5; i++) {
        printf("  File: %s\n", test_files[i]);
        printf("    Extension: '%s'\n", get_extension(test_files[i]));
    }
    printf("\n");

    /* Change extension */
    printf("Change extension:\n");
    char filename[100] = "document.txt";
    printf("  Original: %s\n", filename);
    change_extension(filename, "pdf");
    printf("  Changed:  %s\n", filename);
    printf("\n");

    /* Join paths */
    printf("Join paths:\n");
    char joined[PATH_MAX];
    join_path(joined, sizeof(joined), "/home/user", "documents/file.txt");
    printf("  Result: %s\n", joined);
    printf("\n");

    /* Split path */
    printf("Split path:\n");
    split_path("/home/user/documents/file.txt");
    printf("\n");

    /* Build path from components */
    build_path_from_components();
    printf("\n");

    /* Absolute vs relative */
    printf("Path type detection:\n");
    printf("  /home/user: %s\n", is_absolute_path("/home/user") ? "Absolute" : "Relative");
    printf("  documents/file.txt: %s\n", is_absolute_path("documents/file.txt") ? "Absolute" : "Relative");
    printf("\n");

    /* Get absolute path */
    printf("Get absolute path:\n");
    get_absolute_path(".");
    printf("\n");

    /* Path normalization */
    printf("Path normalization:\n");
    char path1[] = "/home/user/../admin/./documents";
    printf("  Original: %s\n", path1);
    normalize_path(path1);
    printf("  Normalized: %s\n", path1);
    printf("\n");

    /* Relative path computation */
    printf("Relative path computation:\n");
    compute_relative_path("/home/user/projects", "/home/user/documents/file.txt");
    printf("\n");

    /* Path validation */
    printf("Path validation:\n");
    validate_path("/home/user/documents/file.txt");
    validate_path("");
    validate_path("/home//user/documents");
    printf("\n");

    printf("Path manipulation uses:\n");
    printf("  - Parse file system paths\n");
    printf("  - Extract components (dir, basename, extension)\n");
    printf("  - Build paths from components\n");
    printf("  - Normalize and validate paths\n");
    printf("  - Compute relative paths\n");

    return 0;
}
