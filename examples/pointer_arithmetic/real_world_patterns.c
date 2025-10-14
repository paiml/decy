// DECY-041: Real-world pointer arithmetic patterns
// Common patterns found in C codebases

// Array traversal with pointer arithmetic
int sum_array(int* arr, int size) {
    int sum = 0;
    int* end = arr + size;
    while (arr < end) {
        sum += *arr;
        arr++;
    }
    return sum;
}

// Find first occurrence in array
int find_first(int* arr, int size, int target) {
    int i;
    for (i = 0; i < size; i++) {
        if (arr[i] == target) {
            return i;
        }
    }
    return -1;
}

// Count occurrences with continue
int count_even(int* arr, int size) {
    int count = 0;
    int i;
    for (i = 0; i < size; i++) {
        if (arr[i] % 2 == 1) {
            continue;  // Skip odd numbers
        }
        count++;
    }
    return count;
}

// Early exit with break
int linear_search(int* arr, int size, int target) {
    int found = 0;
    int i;
    for (i = 0; i < size; i++) {
        if (arr[i] == target) {
            found = 1;
            break;  // Early exit when found
        }
    }
    return found;
}

// String length calculation (classic pointer arithmetic)
int string_length(char* str) {
    char* start = str;
    while (*str != '\0') {
        str++;
    }
    return str - start;
}
