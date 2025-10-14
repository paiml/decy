// Buffer operations inspired by real systems programming
// Testing: arrays, pointer arithmetic, bounds checking patterns

void buffer_fill(int* buffer, int size, int value) {
    int i;
    for (i = 0; i < size; i = i + 1) {
        buffer[i] = value;
    }
}

int buffer_sum(int* buffer, int size) {
    int sum;
    int i;
    sum = 0;
    for (i = 0; i < size; i = i + 1) {
        sum = sum + buffer[i];
    }
    return sum;
}

int buffer_find(int* buffer, int size, int target) {
    int i;
    for (i = 0; i < size; i = i + 1) {
        if (buffer[i] == target) {
            return i;
        }
    }
    return -1;
}

void buffer_reverse(int* buffer, int size) {
    int i;
    int temp;
    for (i = 0; i < size / 2; i = i + 1) {
        temp = buffer[i];
        buffer[i] = buffer[size - 1 - i];
        buffer[size - 1 - i] = temp;
    }
}
