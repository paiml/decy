/* K&R C Chapter 7: File Locking
 * K&R §8.7: File locking for concurrent access (POSIX)
 * Tests advisory file locking mechanisms
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>

/* Acquire exclusive lock */
int lock_file(int fd, int wait) {
    struct flock lock;

    lock.l_type = F_WRLCK;    /* Exclusive lock */
    lock.l_whence = SEEK_SET;  /* Start from beginning */
    lock.l_start = 0;          /* Offset */
    lock.l_len = 0;            /* Lock entire file */

    int cmd = wait ? F_SETLKW : F_SETLK;

    if (fcntl(fd, cmd, &lock) == -1) {
        if (errno == EACCES || errno == EAGAIN) {
            printf("  File is locked by another process\n");
        } else {
            perror("fcntl");
        }
        return -1;
    }

    printf("  Exclusive lock acquired\n");
    return 0;
}

/* Acquire shared lock */
int lock_file_shared(int fd) {
    struct flock lock;

    lock.l_type = F_RDLCK;     /* Shared lock */
    lock.l_whence = SEEK_SET;
    lock.l_start = 0;
    lock.l_len = 0;

    if (fcntl(fd, F_SETLK, &lock) == -1) {
        perror("fcntl");
        return -1;
    }

    printf("  Shared lock acquired\n");
    return 0;
}

/* Unlock file */
int unlock_file(int fd) {
    struct flock lock;

    lock.l_type = F_UNLCK;     /* Unlock */
    lock.l_whence = SEEK_SET;
    lock.l_start = 0;
    lock.l_len = 0;

    if (fcntl(fd, F_SETLK, &lock) == -1) {
        perror("fcntl");
        return -1;
    }

    printf("  Lock released\n");
    return 0;
}

/* Check if file is locked */
int check_lock(int fd) {
    struct flock lock;

    lock.l_type = F_WRLCK;
    lock.l_whence = SEEK_SET;
    lock.l_start = 0;
    lock.l_len = 0;

    if (fcntl(fd, F_GETLK, &lock) == -1) {
        perror("fcntl");
        return -1;
    }

    if (lock.l_type == F_UNLCK) {
        printf("  File is not locked\n");
        return 0;
    }

    printf("  File is locked by PID %d\n", lock.l_pid);
    printf("  Lock type: %s\n", lock.l_type == F_WRLCK ? "Exclusive" : "Shared");
    return 1;
}

/* Lock specific region */
int lock_region(int fd, off_t start, off_t length) {
    struct flock lock;

    lock.l_type = F_WRLCK;
    lock.l_whence = SEEK_SET;
    lock.l_start = start;
    lock.l_len = length;

    if (fcntl(fd, F_SETLK, &lock) == -1) {
        printf("  Failed to lock region [%ld, %ld)\n", start, start + length);
        return -1;
    }

    printf("  Locked region [%ld, %ld)\n", start, start + length);
    return 0;
}

/* Safe read with locking */
int safe_read(const char *filename, char *buffer, size_t size) {
    int fd = open(filename, O_RDONLY);
    if (fd == -1) {
        perror("open");
        return -1;
    }

    printf("Safe read from: %s\n", filename);

    /* Acquire shared lock for reading */
    if (lock_file_shared(fd) == -1) {
        close(fd);
        return -1;
    }

    /* Read data */
    ssize_t bytes_read = read(fd, buffer, size - 1);
    if (bytes_read == -1) {
        perror("read");
        unlock_file(fd);
        close(fd);
        return -1;
    }
    buffer[bytes_read] = '\0';

    printf("  Read %zd bytes\n", bytes_read);

    /* Release lock */
    unlock_file(fd);
    close(fd);
    return bytes_read;
}

/* Safe write with locking */
int safe_write(const char *filename, const char *data) {
    int fd = open(filename, O_WRONLY | O_CREAT | O_TRUNC, 0644);
    if (fd == -1) {
        perror("open");
        return -1;
    }

    printf("Safe write to: %s\n", filename);

    /* Acquire exclusive lock for writing */
    if (lock_file(fd, 1) == -1) {  /* Wait for lock */
        close(fd);
        return -1;
    }

    /* Write data */
    ssize_t bytes_written = write(fd, data, strlen(data));
    if (bytes_written == -1) {
        perror("write");
        unlock_file(fd);
        close(fd);
        return -1;
    }

    printf("  Wrote %zd bytes\n", bytes_written);

    /* Release lock */
    unlock_file(fd);
    close(fd);
    return bytes_written;
}

/* Append with locking */
int safe_append(const char *filename, const char *data) {
    int fd = open(filename, O_WRONLY | O_CREAT | O_APPEND, 0644);
    if (fd == -1) {
        perror("open");
        return -1;
    }

    printf("Safe append to: %s\n", filename);

    /* Lock file */
    if (lock_file(fd, 1) == -1) {
        close(fd);
        return -1;
    }

    /* Append data */
    ssize_t bytes_written = write(fd, data, strlen(data));
    if (bytes_written == -1) {
        perror("write");
    } else {
        printf("  Appended %zd bytes\n", bytes_written);
    }

    /* Unlock and close */
    unlock_file(fd);
    close(fd);
    return bytes_written;
}

/* Demonstrate record locking */
void demo_record_locking(const char *filename) {
    printf("Record-level locking demo:\n");

    int fd = open(filename, O_RDWR | O_CREAT, 0644);
    if (fd == -1) {
        perror("open");
        return;
    }

    /* Write some records */
    const char *records[] = {
        "Record 1\n",
        "Record 2\n",
        "Record 3\n",
        "Record 4\n"
    };

    for (int i = 0; i < 4; i++) {
        write(fd, records[i], strlen(records[i]));
    }

    /* Lock specific records */
    printf("  Locking record 2 (bytes 10-19):\n");
    lock_region(fd, 10, 10);

    printf("  Locking record 4 (bytes 30-39):\n");
    lock_region(fd, 30, 10);

    printf("  Records 2 and 4 are locked\n");
    printf("  Other processes can still access records 1 and 3\n");

    /* Cleanup */
    unlock_file(fd);
    close(fd);
}

int main() {
    const char *test_file = "lock_test.txt";

    printf("=== File Locking ===\n\n");

    /* Create test file */
    FILE *fp = fopen(test_file, "w");
    fprintf(fp, "Initial content\n");
    fclose(fp);

    /* Safe write */
    printf("1. Safe Write:\n");
    safe_write(test_file, "Locked write operation\n");
    printf("\n");

    /* Safe read */
    printf("2. Safe Read:\n");
    char buffer[100];
    safe_read(test_file, buffer, sizeof(buffer));
    printf("  Content: %s", buffer);
    printf("\n");

    /* Safe append */
    printf("3. Safe Append:\n");
    safe_append(test_file, "Appended line 1\n");
    safe_append(test_file, "Appended line 2\n");
    printf("\n");

    /* Check lock status */
    printf("4. Check Lock Status:\n");
    int fd = open(test_file, O_RDONLY);
    check_lock(fd);
    close(fd);
    printf("\n");

    /* Record locking */
    printf("5. Record Locking:\n");
    demo_record_locking("record_lock_test.txt");
    printf("\n");

    /* Cleanup */
    remove(test_file);
    remove("record_lock_test.txt");

    printf("File locking benefits:\n");
    printf("  - Prevents concurrent write conflicts\n");
    printf("  - Supports shared read locks\n");
    printf("  - Can lock specific file regions\n");
    printf("  - Advisory (cooperative) locking\n");
    printf("\nNote: Locks are advisory - processes must cooperate\n");

    return 0;
}
