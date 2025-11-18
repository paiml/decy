/* K&R C Chapter 5: Graph with Adjacency List
 * K&R §5.10: Pointer-based graph representation
 * Tests graph operations with adjacency lists
 */

#include <stdio.h>
#include <stdlib.h>

typedef struct adj_node {
    int vertex;
    int weight;
    struct adj_node *next;
} AdjNode;

typedef struct {
    AdjNode **adj_lists;
    int num_vertices;
} Graph;

/* Create graph */
Graph *graph_create(int num_vertices) {
    Graph *graph = malloc(sizeof(Graph));
    graph->num_vertices = num_vertices;
    graph->adj_lists = malloc(num_vertices * sizeof(AdjNode*));

    for (int i = 0; i < num_vertices; i++) {
        graph->adj_lists[i] = NULL;
    }

    return graph;
}

/* Add edge */
void graph_add_edge(Graph *graph, int src, int dest, int weight) {
    AdjNode *node = malloc(sizeof(AdjNode));
    node->vertex = dest;
    node->weight = weight;
    node->next = graph->adj_lists[src];
    graph->adj_lists[src] = node;
}

/* Add undirected edge */
void graph_add_undirected_edge(Graph *graph, int v1, int v2, int weight) {
    graph_add_edge(graph, v1, v2, weight);
    graph_add_edge(graph, v2, v1, weight);
}

/* Print graph */
void graph_print(Graph *graph) {
    printf("Graph adjacency lists:\n");
    for (int i = 0; i < graph->num_vertices; i++) {
        printf("  %d: ", i);
        AdjNode *node = graph->adj_lists[i];
        while (node) {
            printf("-> %d(w=%d) ", node->vertex, node->weight);
            node = node->next;
        }
        printf("\n");
    }
}

/* DFS helper */
void graph_dfs_helper(Graph *graph, int vertex, bool *visited) {
    visited[vertex] = true;
    printf("%d ", vertex);

    AdjNode *node = graph->adj_lists[vertex];
    while (node) {
        if (!visited[node->vertex]) {
            graph_dfs_helper(graph, node->vertex, visited);
        }
        node = node->next;
    }
}

/* DFS */
void graph_dfs(Graph *graph, int start) {
    bool *visited = calloc(graph->num_vertices, sizeof(bool));
    printf("DFS from %d: ", start);
    graph_dfs_helper(graph, start, visited);
    printf("\n");
    free(visited);
}

/* BFS */
void graph_bfs(Graph *graph, int start) {
    bool *visited = calloc(graph->num_vertices, sizeof(bool));
    int *queue = malloc(graph->num_vertices * sizeof(int));
    int front = 0, rear = 0;

    visited[start] = true;
    queue[rear++] = start;

    printf("BFS from %d: ", start);

    while (front < rear) {
        int vertex = queue[front++];
        printf("%d ", vertex);

        AdjNode *node = graph->adj_lists[vertex];
        while (node) {
            if (!visited[node->vertex]) {
                visited[node->vertex] = true;
                queue[rear++] = node->vertex;
            }
            node = node->next;
        }
    }

    printf("\n");
    free(visited);
    free(queue);
}

int main() {
    printf("=== Graph with Adjacency List ===\n\n");

    Graph *graph = graph_create(6);

    printf("Adding edges:\n");
    graph_add_undirected_edge(graph, 0, 1, 5);
    graph_add_undirected_edge(graph, 0, 2, 3);
    graph_add_undirected_edge(graph, 1, 3, 7);
    graph_add_undirected_edge(graph, 2, 3, 2);
    graph_add_undirected_edge(graph, 3, 4, 6);
    graph_add_undirected_edge(graph, 4, 5, 1);

    graph_print(graph);
    printf("\n");

    graph_dfs(graph, 0);
    graph_bfs(graph, 0);

    return 0;
}
