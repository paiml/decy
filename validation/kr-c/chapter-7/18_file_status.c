/* K&R C Chapter 7: File Status and Metadata
 * K&R §7.5, §8.6: File operations and status
 * Tests file metadata, status, and properties
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <time.h>
#include <unistd.h>

/* Get file size */
long get_file_size(const char *filename) {
    struct stat st;
    if (stat(filename, &st) == 0) {
        return st.st_size;
    }
    return -1;
}

/* Check if file exists */
int file_exists(const char *filename) {
    return access(filename, F_OK) == 0;
}

/* Check file permissions */
void check_permissions(const char *filename) {
    printf("File: %s\n", filename);

    if (access(filename, F_OK) == 0) {
        printf("  Exists: YES\n");
    } else {
        printf("  Exists: NO\n");
        return;
    }

    printf("  Readable:   %s\n", access(filename, R_OK) == 0 ? "YES" : "NO");
    printf("  Writable:   %s\n", access(filename, W_OK) == 0 ? "YES" : "NO");
    printf("  Executable: %s\n", access(filename, X_OK) == 0 ? "YES" : "NO");
}

/* Get detailed file information */
void file_info(const char *filename) {
    struct stat st;

    if (stat(filename, &st) != 0) {
        perror("stat");
        return;
    }

    printf("File information for: %s\n", filename);
    printf("  Size:        %ld bytes\n", (long)st.st_size);
    printf("  Blocks:      %ld\n", (long)st.st_blocks);
    printf("  Block size:  %ld bytes\n", (long)st.st_blksize);
    printf("  Inode:       %lu\n", (unsigned long)st.st_ino);
    printf("  Links:       %lu\n", (unsigned long)st.st_nlink);
    printf("  Mode:        %o (octal)\n", st.st_mode);

    /* File type */
    printf("  Type:        ");
    if (S_ISREG(st.st_mode))
        printf("Regular file\n");
    else if (S_ISDIR(st.st_mode))
        printf("Directory\n");
    else if (S_ISCHR(st.st_mode))
        printf("Character device\n");
    else if (S_ISBLK(st.st_mode))
        printf("Block device\n");
    else if (S_ISFIFO(st.st_mode))
        printf("FIFO/pipe\n");
    else if (S_ISLNK(st.st_mode))
        printf("Symbolic link\n");
    else
        printf("Unknown\n");

    /* Timestamps */
    char time_buffer[100];

    strftime(time_buffer, sizeof(time_buffer), "%Y-%m-%d %H:%M:%S",
             localtime(&st.st_mtime));
    printf("  Modified:    %s\n", time_buffer);

    strftime(time_buffer, sizeof(time_buffer), "%Y-%m-%d %H:%M:%S",
             localtime(&st.st_atime));
    printf("  Accessed:    %s\n", time_buffer);

    strftime(time_buffer, sizeof(time_buffer), "%Y-%m-%d %H:%M:%S",
             localtime(&st.st_ctime));
    printf("  Changed:     %s\n", time_buffer);
}

/* Compare file timestamps */
int file_is_newer(const char *file1, const char *file2) {
    struct stat st1, st2;

    if (stat(file1, &st1) != 0 || stat(file2, &st2) != 0) {
        return -1;
    }

    return st1.st_mtime > st2.st_mtime;
}

/* Get file extension */
const char *get_file_extension(const char *filename) {
    const char *dot = strrchr(filename, '.');
    if (dot && dot != filename) {
        return dot + 1;
    }
    return "";
}

/* Classify file by extension */
void classify_file(const char *filename) {
    const char *ext = get_file_extension(filename);

    printf("File: %s\n", filename);
    printf("  Extension: %s\n", *ext ? ext : "(none)");
    printf("  Type: ");

    if (strcmp(ext, "c") == 0 || strcmp(ext, "h") == 0) {
        printf("C source/header\n");
    } else if (strcmp(ext, "txt") == 0) {
        printf("Text file\n");
    } else if (strcmp(ext, "jpg") == 0 || strcmp(ext, "png") == 0) {
        printf("Image file\n");
    } else if (strcmp(ext, "pdf") == 0) {
        printf("PDF document\n");
    } else if (strcmp(ext, "zip") == 0 || strcmp(ext, "tar") == 0) {
        printf("Archive\n");
    } else {
        printf("Unknown\n");
    }
}

/* File status flags */
void check_file_stream_status(FILE *fp) {
    printf("Stream status:\n");

    if (feof(fp)) {
        printf("  EOF indicator set\n");
    } else {
        printf("  EOF indicator not set\n");
    }

    if (ferror(fp)) {
        printf("  Error indicator set\n");
    } else {
        printf("  Error indicator not set\n");
    }

    /* Get file descriptor */
    int fd = fileno(fp);
    printf("  File descriptor: %d\n", fd);

    /* Check if seekable */
    if (fseek(fp, 0, SEEK_CUR) == 0) {
        printf("  Seekable: YES\n");
    } else {
        printf("  Seekable: NO\n");
    }
}

/* Directory test */
void test_directory_vs_file(const char *path) {
    struct stat st;

    printf("Testing: %s\n", path);

    if (stat(path, &st) != 0) {
        printf("  Error: Cannot stat\n");
        return;
    }

    if (S_ISDIR(st.st_mode)) {
        printf("  Type: Directory\n");
        printf("  Can list contents: YES\n");
    } else if (S_ISREG(st.st_mode)) {
        printf("  Type: Regular file\n");
        printf("  Size: %ld bytes\n", (long)st.st_size);
    } else {
        printf("  Type: Other\n");
    }
}

int main() {
    const char *test_file = "status_test.txt";

    printf("=== File Status and Metadata ===\n\n");

    /* Create test file */
    FILE *fp = fopen(test_file, "w");
    if (fp == NULL) {
        perror("fopen");
        return 1;
    }
    fprintf(fp, "This is a test file for status checking.\n");
    fprintf(fp, "It contains multiple lines.\n");
    fprintf(fp, "And some data to check.\n");
    fclose(fp);

    /* Check existence */
    printf("File existence:\n");
    printf("  %s: %s\n", test_file, file_exists(test_file) ? "EXISTS" : "NOT FOUND");
    printf("  nonexistent.txt: %s\n\n", file_exists("nonexistent.txt") ? "EXISTS" : "NOT FOUND");

    /* Check permissions */
    printf("Permissions:\n");
    check_permissions(test_file);
    printf("\n");

    /* Get file size */
    printf("File size:\n");
    long size = get_file_size(test_file);
    printf("  %s: %ld bytes\n\n", test_file, size);

    /* Detailed info */
    printf("Detailed file information:\n");
    file_info(test_file);
    printf("\n");

    /* Stream status */
    printf("Stream status checks:\n");
    fp = fopen(test_file, "r");
    check_file_stream_status(fp);
    fgetc(fp);  /* Read one character */
    printf("\nAfter reading one character:\n");
    check_file_stream_status(fp);
    fclose(fp);
    printf("\n");

    /* File classification */
    printf("File classification:\n");
    classify_file("program.c");
    classify_file("document.pdf");
    classify_file("photo.jpg");
    classify_file("README");
    printf("\n");

    /* Directory vs file */
    printf("Directory vs file:\n");
    test_directory_vs_file(".");
    test_directory_vs_file(test_file);
    printf("\n");

    /* Cleanup */
    remove(test_file);

    printf("File status operations:\n");
    printf("  - stat() provides detailed file metadata\n");
    printf("  - access() checks permissions\n");
    printf("  - feof()/ferror() check stream status\n");
    printf("  - fileno() gets file descriptor\n");

    return 0;
}
