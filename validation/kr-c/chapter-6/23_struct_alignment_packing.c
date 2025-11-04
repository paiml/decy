/* K&R C Chapter 6: Structure Alignment and Packing
 * K&R §6.9: Memory layout and alignment
 * Tests structure padding, alignment, and packing
 */

#include <stdio.h>
#include <stddef.h>
#include <stdint.h>

/* Natural alignment - compiler adds padding */
typedef struct {
    char c;       /* 1 byte */
    /* 3 bytes padding */
    int i;        /* 4 bytes */
    short s;      /* 2 bytes */
    /* 2 bytes padding */
} Natural;

/* Packed structure - no padding */
typedef struct __attribute__((packed)) {
    char c;       /* 1 byte */
    int i;        /* 4 bytes */
    short s;      /* 2 bytes */
} Packed;

/* Reordered for better packing */
typedef struct {
    int i;        /* 4 bytes */
    short s;      /* 2 bytes */
    char c;       /* 1 byte */
    /* 1 byte padding */
} Reordered;

/* Example with multiple alignment requirements */
typedef struct {
    char c1;      /* 1 byte */
    /* 7 bytes padding */
    double d;     /* 8 bytes - requires 8-byte alignment */
    char c2;      /* 1 byte */
    /* 3 bytes padding */
    int i;        /* 4 bytes */
} AlignmentDemo;

/* Demonstrate offsetof */
void show_offsets(void) {
    printf("=== Structure Member Offsets ===\n");

    printf("Natural structure:\n");
    printf("  offsetof(c): %zu\n", offsetof(Natural, c));
    printf("  offsetof(i): %zu\n", offsetof(Natural, i));
    printf("  offsetof(s): %zu\n", offsetof(Natural, s));
    printf("  Total size:  %zu bytes\n", sizeof(Natural));
    printf("\n");

    printf("Packed structure:\n");
    printf("  offsetof(c): %zu\n", offsetof(Packed, c));
    printf("  offsetof(i): %zu\n", offsetof(Packed, i));
    printf("  offsetof(s): %zu\n", offsetof(Packed, s));
    printf("  Total size:  %zu bytes\n", sizeof(Packed));
    printf("\n");

    printf("Reordered structure:\n");
    printf("  offsetof(i): %zu\n", offsetof(Reordered, i));
    printf("  offsetof(s): %zu\n", offsetof(Reordered, s));
    printf("  offsetof(c): %zu\n", offsetof(Reordered, c));
    printf("  Total size:  %zu bytes\n", sizeof(Reordered));
    printf("\n");
}

/* Calculate padding */
void show_padding(void) {
    printf("=== Structure Padding ===\n");

    size_t natural_members = sizeof(char) + sizeof(int) + sizeof(short);
    size_t natural_padding = sizeof(Natural) - natural_members;

    printf("Natural structure:\n");
    printf("  Members:     %zu bytes\n", natural_members);
    printf("  Padding:     %zu bytes\n", natural_padding);
    printf("  Total:       %zu bytes\n", sizeof(Natural));
    printf("  Efficiency:  %.1f%%\n", (natural_members * 100.0) / sizeof(Natural));
    printf("\n");

    size_t packed_members = sizeof(char) + sizeof(int) + sizeof(short);
    printf("Packed structure:\n");
    printf("  Members:     %zu bytes\n", packed_members);
    printf("  Padding:     0 bytes\n");
    printf("  Total:       %zu bytes\n", sizeof(Packed));
    printf("  Efficiency:  100%%\n");
    printf("\n");

    size_t reordered_members = sizeof(int) + sizeof(short) + sizeof(char);
    size_t reordered_padding = sizeof(Reordered) - reordered_members;

    printf("Reordered structure:\n");
    printf("  Members:     %zu bytes\n", reordered_members);
    printf("  Padding:     %zu bytes\n", reordered_padding);
    printf("  Total:       %zu bytes\n", sizeof(Reordered));
    printf("  Efficiency:  %.1f%%\n", (reordered_members * 100.0) / sizeof(Reordered));
    printf("\n");
}

/* Alignment requirements */
void show_alignment(void) {
    printf("=== Alignment Requirements ===\n");

    printf("Type alignments:\n");
    printf("  char:   %zu bytes\n", _Alignof(char));
    printf("  short:  %zu bytes\n", _Alignof(short));
    printf("  int:    %zu bytes\n", _Alignof(int));
    printf("  long:   %zu bytes\n", _Alignof(long));
    printf("  float:  %zu bytes\n", _Alignof(float));
    printf("  double: %zu bytes\n", _Alignof(double));
    printf("  void*:  %zu bytes\n", _Alignof(void*));
    printf("\n");

    printf("Structure alignments:\n");
    printf("  Natural:       %zu bytes\n", _Alignof(Natural));
    printf("  Packed:        %zu bytes\n", _Alignof(Packed));
    printf("  Reordered:     %zu bytes\n", _Alignof(Reordered));
    printf("  AlignmentDemo: %zu bytes\n", _Alignof(AlignmentDemo));
    printf("\n");
}

/* Array of structures */
void array_memory_layout(void) {
    printf("=== Array Memory Layout ===\n");

    Natural arr[3];

    printf("Natural array[3]:\n");
    printf("  Element 0: %p\n", (void*)&arr[0]);
    printf("  Element 1: %p (offset: %zu)\n",
           (void*)&arr[1],
           (char*)&arr[1] - (char*)&arr[0]);
    printf("  Element 2: %p (offset: %zu)\n",
           (void*)&arr[2],
           (char*)&arr[2] - (char*)&arr[0]);
    printf("  Total array size: %zu bytes\n", sizeof(arr));
    printf("\n");
}

/* Flexible array member with alignment */
typedef struct {
    int count;
    int data[];  /* Flexible array member */
} FlexArray;

void flexible_array_demo(void) {
    printf("=== Flexible Array Member ===\n");

    printf("FlexArray (without flexible member):\n");
    printf("  Size: %zu bytes\n", sizeof(FlexArray));
    printf("  offsetof(count): %zu\n", offsetof(FlexArray, count));
    printf("\n");

    /* Allocate with space for 5 elements */
    FlexArray *fa = malloc(sizeof(FlexArray) + 5 * sizeof(int));
    fa->count = 5;

    printf("Allocated FlexArray with 5 elements:\n");
    printf("  Base size:  %zu bytes\n", sizeof(FlexArray));
    printf("  Array size: %zu bytes\n", 5 * sizeof(int));
    printf("  Total:      %zu bytes\n", sizeof(FlexArray) + 5 * sizeof(int));

    free(fa);
    printf("\n");
}

/* Union with padding */
typedef union {
    char c;
    int i;
    double d;
} UnionDemo;

void union_alignment(void) {
    printf("=== Union Alignment ===\n");

    printf("Union size and alignment:\n");
    printf("  Size:      %zu bytes\n", sizeof(UnionDemo));
    printf("  Alignment: %zu bytes\n", _Alignof(UnionDemo));
    printf("\n");

    printf("Union is sized to largest member (double: %zu bytes)\n", sizeof(double));
    printf("Alignment is strictest requirement (double: %zu bytes)\n", _Alignof(double));
    printf("\n");
}

/* Cache line alignment */
typedef struct __attribute__((aligned(64))) {
    int data;
} CacheAligned;

void cache_alignment(void) {
    printf("=== Cache Line Alignment ===\n");

    printf("CacheAligned structure:\n");
    printf("  Size:      %zu bytes\n", sizeof(CacheAligned));
    printf("  Alignment: %zu bytes\n", _Alignof(CacheAligned));
    printf("\n");

    printf("Aligned to 64 bytes (typical cache line size)\n");
    printf("Reduces false sharing in multithreaded code\n");
    printf("\n");
}

/* Performance impact */
void performance_impact(void) {
    printf("=== Performance Impact ===\n");

    const int N = 1000000;

    /* Natural alignment - fast access */
    Natural *natural_arr = malloc(N * sizeof(Natural));
    for (int i = 0; i < N; i++) {
        natural_arr[i].i = i;
        natural_arr[i].s = i;
        natural_arr[i].c = i;
    }

    printf("Natural alignment:\n");
    printf("  Memory usage: %zu MB\n", (N * sizeof(Natural)) / (1024 * 1024));
    printf("  Fast access due to alignment\n\n");

    free(natural_arr);

    /* Packed - slower access, less memory */
    Packed *packed_arr = malloc(N * sizeof(Packed));
    for (int i = 0; i < N; i++) {
        packed_arr[i].i = i;
        packed_arr[i].s = i;
        packed_arr[i].c = i;
    }

    printf("Packed:\n");
    printf("  Memory usage: %zu MB\n", (N * sizeof(Packed)) / (1024 * 1024));
    printf("  Potentially slower due to unaligned access\n");
    printf("  Memory saved: %zu MB\n",
           ((N * sizeof(Natural)) - (N * sizeof(Packed))) / (1024 * 1024));

    free(packed_arr);
    printf("\n");
}

int main() {
    printf("=== Structure Alignment and Packing ===\n\n");

    show_offsets();
    show_padding();
    show_alignment();
    array_memory_layout();
    flexible_array_demo();
    union_alignment();
    cache_alignment();
    performance_impact();

    printf("Key takeaways:\n");
    printf("  - Compilers add padding for alignment\n");
    printf("  - Reorder members to reduce padding\n");
    printf("  - Use __attribute__((packed)) sparingly\n");
    printf("  - Alignment affects performance\n");
    printf("  - Cache line alignment prevents false sharing\n");
    printf("  - Use offsetof() to inspect layout\n");

    return 0;
}
