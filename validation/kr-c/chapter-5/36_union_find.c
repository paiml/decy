/* K&R C Chapter 5: Union-Find (Disjoint Set)
 * K&R §5.10: Path compression and union by rank
 * Tests union-find for connectivity problems
 */

#include <stdio.h>
#include <stdlib.h>

typedef struct {
    int *parent;
    int *rank;
    int size;
} UnionFind;

/* Create union-find */
UnionFind *uf_create(int size) {
    UnionFind *uf = malloc(sizeof(UnionFind));
    uf->parent = malloc(size * sizeof(int));
    uf->rank = malloc(size * sizeof(int));
    uf->size = size;

    for (int i = 0; i < size; i++) {
        uf->parent[i] = i;
        uf->rank[i] = 0;
    }

    return uf;
}

/* Find with path compression */
int uf_find(UnionFind *uf, int x) {
    if (uf->parent[x] != x) {
        uf->parent[x] = uf_find(uf, uf->parent[x]);
    }
    return uf->parent[x];
}

/* Union by rank */
void uf_union(UnionFind *uf, int x, int y) {
    int root_x = uf_find(uf, x);
    int root_y = uf_find(uf, y);

    if (root_x == root_y) return;

    if (uf->rank[root_x] < uf->rank[root_y]) {
        uf->parent[root_x] = root_y;
    } else if (uf->rank[root_x] > uf->rank[root_y]) {
        uf->parent[root_y] = root_x;
    } else {
        uf->parent[root_y] = root_x;
        uf->rank[root_x]++;
    }
}

/* Check connected */
int uf_connected(UnionFind *uf, int x, int y) {
    return uf_find(uf, x) == uf_find(uf, y);
}

/* Count components */
int uf_count_components(UnionFind *uf) {
    int count = 0;
    for (int i = 0; i < uf->size; i++) {
        if (uf_find(uf, i) == i) {
            count++;
        }
    }
    return count;
}

/* Print sets */
void uf_print(UnionFind *uf) {
    printf("Disjoint sets:\n");
    for (int i = 0; i < uf->size; i++) {
        int root = uf_find(uf, i);
        printf("  %d -> root %d\n", i, root);
    }
}

/* Destroy */
void uf_destroy(UnionFind *uf) {
    free(uf->parent);
    free(uf->rank);
    free(uf);
}

int main() {
    printf("=== Union-Find (Disjoint Set) ===\n\n");

    UnionFind *uf = uf_create(10);

    printf("Initial: 10 disjoint elements\n");
    printf("Components: %d\n\n", uf_count_components(uf));

    printf("Union operations:\n");
    printf("  Union(0, 1)\n");
    uf_union(uf, 0, 1);
    printf("  Union(2, 3)\n");
    uf_union(uf, 2, 3);
    printf("  Union(0, 2)\n");
    uf_union(uf, 0, 2);
    printf("  Union(4, 5)\n");
    uf_union(uf, 4, 5);
    printf("  Union(6, 7)\n");
    uf_union(uf, 6, 7);
    printf("\n");

    uf_print(uf);
    printf("\nComponents: %d\n", uf_count_components(uf));

    printf("\nConnectivity queries:\n");
    printf("  Connected(0, 3): %s\n", uf_connected(uf, 0, 3) ? "Yes" : "No");
    printf("  Connected(0, 4): %s\n", uf_connected(uf, 0, 4) ? "Yes" : "No");
    printf("  Connected(6, 7): %s\n", uf_connected(uf, 6, 7) ? "Yes" : "No");

    uf_destroy(uf);
    return 0;
}
