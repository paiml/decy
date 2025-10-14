// DECY-041: Test increment/decrement operators
// This file tests ++/-- operators in various contexts

// Post-increment
int post_increment_test() {
    int x = 5;
    x++;
    return x;  // Should return 6
}

// Pre-increment
int pre_increment_test() {
    int x = 5;
    ++x;
    return x;  // Should return 6
}

// Post-decrement
int post_decrement_test() {
    int x = 5;
    x--;
    return x;  // Should return 4
}

// Pre-decrement
int pre_decrement_test() {
    int x = 5;
    --x;
    return x;  // Should return 4
}

// Increment in for loop
int sum_to_n(int n) {
    int sum = 0;
    int i;
    for (i = 0; i < n; i++) {
        sum += i;
    }
    return sum;
}

// Decrement in for loop
int countdown_sum(int n) {
    int sum = 0;
    int i;
    for (i = n; i > 0; i--) {
        sum += i;
    }
    return sum;
}

// Multiple increments in loop body
void traverse_array(int* arr, int size) {
    int* ptr = arr;
    int i;
    for (i = 0; i < size; i++) {
        ptr++;
    }
}
