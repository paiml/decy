// Test control flow structures
int max(int a, int b) {
    if (a > b) {
        return a;
    } else {
        return b;
    }
}

int factorial(int n) {
    int result;
    int i;
    result = 1;
    for (i = 1; i <= n; i = i + 1) {
        result = result * i;
    }
    return result;
}
