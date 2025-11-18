/* K&R C Chapter 8: System Information
 * K&R §8.7: System calls for system information
 * Tests uname, sysconf, getrlimit
 */

#include <stdio.h>
#include <sys/utsname.h>
#include <unistd.h>
#include <sys/resource.h>

void demo_uname(void) {
    printf("=== uname() Demo ===\n");

    struct utsname info;
    if (uname(&info) == 0) {
        printf("System: %s\n", info.sysname);
        printf("Node:   %s\n", info.nodename);
        printf("Release: %s\n", info.release);
        printf("Version: %s\n", info.version);
        printf("Machine: %s\n", info.machine);
    }

    printf("\n");
}

void demo_sysconf(void) {
    printf("=== sysconf() Demo ===\n");

    long page_size = sysconf(_SC_PAGESIZE);
    printf("Page size: %ld bytes\n", page_size);

    long num_cpus = sysconf(_SC_NPROCESSORS_ONLN);
    printf("CPUs online: %ld\n", num_cpus);

    long max_open_files = sysconf(_SC_OPEN_MAX);
    printf("Max open files: %ld\n", max_open_files);

    printf("\n");
}

void demo_getrlimit(void) {
    printf("=== getrlimit() Demo ===\n");

    struct rlimit limit;

    /* CPU time limit */
    if (getrlimit(RLIMIT_CPU, &limit) == 0) {
        printf("CPU time limit: soft=%ld, hard=%ld\n",
               limit.rlim_cur, limit.rlim_max);
    }

    /* File size limit */
    if (getrlimit(RLIMIT_FSIZE, &limit) == 0) {
        printf("File size limit: soft=%ld, hard=%ld\n",
               limit.rlim_cur, limit.rlim_max);
    }

    /* Number of open files */
    if (getrlimit(RLIMIT_NOFILE, &limit) == 0) {
        printf("Open files limit: soft=%ld, hard=%ld\n",
               limit.rlim_cur, limit.rlim_max);
    }

    printf("\n");
}

int main() {
    printf("=== System Information ===\n\n");

    demo_uname();
    demo_sysconf();
    demo_getrlimit();

    printf("System info functions:\n");
    printf("  - uname(): OS and machine info\n");
    printf("  - sysconf(): System configuration\n");
    printf("  - getrlimit(): Resource limits\n");

    return 0;
}
