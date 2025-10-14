// Linked list operations inspired by real data structure implementations
// Testing: structs, pointers, malloc/free patterns

struct Node {
    int data;
    struct Node* next;
};

struct Node* create_node(int value) {
    struct Node* node;
    node = malloc(sizeof(struct Node));
    node->data = value;
    node->next = 0;
    return node;
}

int list_length(struct Node* head) {
    int count;
    count = 0;
    while (head != 0) {
        count = count + 1;
        head = head->next;
    }
    return count;
}

int list_sum(struct Node* head) {
    int sum;
    sum = 0;
    while (head != 0) {
        sum = sum + head->data;
        head = head->next;
    }
    return sum;
}
