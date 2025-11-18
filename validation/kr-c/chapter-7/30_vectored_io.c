/* K&R C Chapter 7: Vectored I/O (Scatter-Gather)
 * K&R Appendix B: Advanced I/O patterns
 * Tests readv/writev and advanced I/O techniques
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/uio.h>

/* Basic writev demo */
void writev_demo(void) {
    printf("=== writev Demo ===\n");

    const char *filename = "writev_test.txt";
    int fd = open(filename, O_WRONLY | O_CREAT | O_TRUNC, 0644);

    /* Prepare multiple buffers */
    char buf1[] = "First part, ";
    char buf2[] = "second part, ";
    char buf3[] = "third part.\n";

    struct iovec iov[3];
    iov[0].iov_base = buf1;
    iov[0].iov_len = strlen(buf1);
    iov[1].iov_base = buf2;
    iov[1].iov_len = strlen(buf2);
    iov[2].iov_base = buf3;
    iov[2].iov_len = strlen(buf3);

    /* Write all buffers in one system call */
    ssize_t nwritten = writev(fd, iov, 3);
    printf("Wrote %zd bytes in one writev call\n", nwritten);

    close(fd);

    /* Verify */
    FILE *fp = fopen(filename, "r");
    char buffer[100];
    fgets(buffer, sizeof(buffer), fp);
    printf("File contents: %s", buffer);
    fclose(fp);

    remove(filename);
    printf("\n");
}

/* Basic readv demo */
void readv_demo(void) {
    printf("=== readv Demo ===\n");

    const char *filename = "readv_test.txt";

    /* Create test file */
    FILE *fp = fopen(filename, "w");
    fprintf(fp, "123456789ABCDEFGHIJKLMNOP");
    fclose(fp);

    /* Open for reading */
    int fd = open(filename, O_RDONLY);

    /* Prepare multiple buffers */
    char buf1[10], buf2[10], buf3[10];

    struct iovec iov[3];
    iov[0].iov_base = buf1;
    iov[0].iov_len = 9;
    iov[1].iov_base = buf2;
    iov[1].iov_len = 9;
    iov[2].iov_base = buf3;
    iov[2].iov_len = 9;

    /* Read into all buffers in one system call */
    ssize_t nread = readv(fd, iov, 3);
    printf("Read %zd bytes in one readv call\n", nread);

    buf1[9] = '\0';
    buf2[9] = '\0';
    buf3[9] = '\0';

    printf("Buffer 1: %s\n", buf1);
    printf("Buffer 2: %s\n", buf2);
    printf("Buffer 3: %s\n", buf3);

    close(fd);
    remove(filename);
    printf("\n");
}

/* Scatter input across multiple buffers */
void scatter_input(void) {
    printf("=== Scatter Input ===\n");

    const char *filename = "scatter_test.txt";

    /* Create structured file */
    FILE *fp = fopen(filename, "w");
    int header = 0x12345678;
    char data[] = "Important data goes here";
    int footer = 0x87654321;

    fwrite(&header, sizeof(header), 1, fp);
    fwrite(data, 1, sizeof(data), fp);
    fwrite(&footer, sizeof(footer), 1, fp);
    fclose(fp);

    /* Read into separate structures */
    int fd = open(filename, O_RDONLY);

    int read_header, read_footer;
    char read_data[50];

    struct iovec iov[3];
    iov[0].iov_base = &read_header;
    iov[0].iov_len = sizeof(read_header);
    iov[1].iov_base = read_data;
    iov[1].iov_len = sizeof(data);
    iov[2].iov_base = &read_footer;
    iov[2].iov_len = sizeof(read_footer);

    ssize_t n = readv(fd, iov, 3);
    printf("Read %zd bytes\n", n);
    printf("  Header: 0x%08X\n", read_header);
    printf("  Data:   %s\n", read_data);
    printf("  Footer: 0x%08X\n", read_footer);

    close(fd);
    remove(filename);
    printf("\n");
}

/* Gather output from multiple buffers */
void gather_output(void) {
    printf("=== Gather Output ===\n");

    const char *filename = "gather_test.txt";
    int fd = open(filename, O_WRONLY | O_CREAT | O_TRUNC, 0644);

    /* Multiple data sources */
    int record_id = 42;
    char name[] = "Alice";
    float score = 95.5;
    char status = 'A';

    /* Gather into single write */
    struct iovec iov[4];
    iov[0].iov_base = &record_id;
    iov[0].iov_len = sizeof(record_id);
    iov[1].iov_base = name;
    iov[1].iov_len = sizeof(name);
    iov[2].iov_base = &score;
    iov[2].iov_len = sizeof(score);
    iov[3].iov_base = &status;
    iov[3].iov_len = sizeof(status);

    ssize_t n = writev(fd, iov, 4);
    printf("Wrote %zd bytes from 4 separate buffers\n", n);

    close(fd);

    /* Verify by reading back */
    fp = fopen(filename, "r");
    struct stat st;
    stat(filename, &st);
    printf("File size: %ld bytes\n", (long)st.st_size);
    fclose(fp);

    remove(filename);
    printf("\n");
}

/* Performance: writev vs multiple write calls */
void performance_comparison(void) {
    printf("=== Performance: writev vs write ===\n");

    const char *filename = "perf_test.txt";
    const int iterations = 1000;

    char buf1[100], buf2[100], buf3[100];
    memset(buf1, 'A', sizeof(buf1));
    memset(buf2, 'B', sizeof(buf2));
    memset(buf3, 'C', sizeof(buf3));

    /* Multiple write() calls */
    struct timeval start, end;
    gettimeofday(&start, NULL);

    int fd = open(filename, O_WRONLY | O_CREAT | O_TRUNC, 0644);
    for (int i = 0; i < iterations; i++) {
        write(fd, buf1, sizeof(buf1));
        write(fd, buf2, sizeof(buf2));
        write(fd, buf3, sizeof(buf3));
    }
    close(fd);

    gettimeofday(&end, NULL);
    double write_time = (end.tv_sec - start.tv_sec) +
                        (end.tv_usec - start.tv_usec) / 1000000.0;
    printf("  Multiple write(): %.6f seconds\n", write_time);

    /* writev() */
    gettimeofday(&start, NULL);

    fd = open(filename, O_WRONLY | O_CREAT | O_TRUNC, 0644);

    struct iovec iov[3];
    iov[0].iov_base = buf1;
    iov[0].iov_len = sizeof(buf1);
    iov[1].iov_base = buf2;
    iov[1].iov_len = sizeof(buf2);
    iov[2].iov_base = buf3;
    iov[2].iov_len = sizeof(buf3);

    for (int i = 0; i < iterations; i++) {
        writev(fd, iov, 3);
    }
    close(fd);

    gettimeofday(&end, NULL);
    double writev_time = (end.tv_sec - start.tv_sec) +
                         (end.tv_usec - start.tv_usec) / 1000000.0;
    printf("  writev():         %.6f seconds\n", writev_time);

    printf("  Speedup:          %.2fx\n", write_time / writev_time);

    remove(filename);
    printf("\n");
}

/* Partial I/O handling */
void partial_io_handling(void) {
    printf("=== Partial I/O Handling ===\n");

    const char *filename = "partial_test.txt";
    int fd = open(filename, O_WRONLY | O_CREAT | O_TRUNC, 0644);

    char buf1[1000], buf2[1000], buf3[1000];
    memset(buf1, 'A', sizeof(buf1));
    memset(buf2, 'B', sizeof(buf2));
    memset(buf3, 'C', sizeof(buf3));

    struct iovec iov[3];
    iov[0].iov_base = buf1;
    iov[0].iov_len = sizeof(buf1);
    iov[1].iov_base = buf2;
    iov[1].iov_len = sizeof(buf2);
    iov[2].iov_base = buf3;
    iov[2].iov_len = sizeof(buf3);

    size_t total_requested = sizeof(buf1) + sizeof(buf2) + sizeof(buf3);
    printf("Requested write: %zu bytes\n", total_requested);

    ssize_t nwritten = writev(fd, iov, 3);

    if (nwritten == -1) {
        perror("writev");
    } else if (nwritten < (ssize_t)total_requested) {
        printf("Partial write: %zd of %zu bytes\n", nwritten, total_requested);
    } else {
        printf("Complete write: %zd bytes\n", nwritten);
    }

    close(fd);
    remove(filename);
    printf("\n");
}

/* Network-style message handling */
void message_handling(void) {
    printf("=== Message Handling Pattern ===\n");

    const char *filename = "message_test.txt";

    typedef struct {
        int msg_type;
        int msg_length;
    } MessageHeader;

    typedef struct {
        char payload[50];
    } MessageBody;

    /* Write message */
    int fd = open(filename, O_WRONLY | O_CREAT | O_TRUNC, 0644);

    MessageHeader header = {1, 13};
    MessageBody body;
    strcpy(body.payload, "Hello, World!");

    struct iovec iov[2];
    iov[0].iov_base = &header;
    iov[0].iov_len = sizeof(header);
    iov[1].iov_base = &body;
    iov[1].iov_len = header.msg_length;

    writev(fd, iov, 2);
    close(fd);

    printf("Wrote message: type=%d, length=%d\n", header.msg_type, header.msg_length);

    /* Read message */
    fd = open(filename, O_RDONLY);

    MessageHeader read_header;
    MessageBody read_body;

    iov[0].iov_base = &read_header;
    iov[0].iov_len = sizeof(read_header);
    iov[1].iov_base = &read_body;
    iov[1].iov_len = sizeof(read_body);

    readv(fd, iov, 2);
    close(fd);

    printf("Read message: type=%d, length=%d, payload='%s'\n",
           read_header.msg_type, read_header.msg_length, read_body.payload);

    remove(filename);
    printf("\n");
}

int main() {
    printf("=== Vectored I/O (Scatter-Gather) ===\n\n");

    writev_demo();
    readv_demo();
    scatter_input();
    gather_output();
    performance_comparison();
    partial_io_handling();
    message_handling();

    printf("Vectored I/O benefits:\n");
    printf("  - Fewer system calls (better performance)\n");
    printf("  - Atomic writes from multiple buffers\n");
    printf("  - Scatter: Read into multiple destinations\n");
    printf("  - Gather: Write from multiple sources\n");
    printf("  - Ideal for structured data (headers + payload)\n");
    printf("  - Reduces need for intermediate buffering\n");

    return 0;
}
