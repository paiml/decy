/* K&R C Chapter 6: Flexible Array Members
 * Structure with variable-length array at end (C99)
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

struct packet {
    int id;
    size_t length;
    char data[];  /* Flexible array member */
};

struct packet *create_packet(int id, const char *data) {
    size_t len = strlen(data);
    struct packet *pkt = malloc(sizeof(struct packet) + len + 1);

    if (pkt) {
        pkt->id = id;
        pkt->length = len;
        strcpy(pkt->data, data);
    }

    return pkt;
}

void print_packet(struct packet *pkt) {
    printf("Packet %d [%zu bytes]: \"%s\"\n",
           pkt->id, pkt->length, pkt->data);
}

int main() {
    printf("=== Flexible Array Member Demo ===\n\n");

    struct packet *p1 = create_packet(1, "Hello");
    struct packet *p2 = create_packet(2, "This is a longer message");
    struct packet *p3 = create_packet(3, "X");

    print_packet(p1);
    print_packet(p2);
    print_packet(p3);

    printf("\nSizes:\n");
    printf("  sizeof(struct packet) = %zu\n", sizeof(struct packet));
    printf("  Packet 1 allocated: %zu bytes\n",
           sizeof(struct packet) + p1->length + 1);
    printf("  Packet 2 allocated: %zu bytes\n",
           sizeof(struct packet) + p2->length + 1);

    free(p1);
    free(p2);
    free(p3);

    return 0;
}
