// Math utility functions inspired by real numerical code
// Testing: loops, conditionals, edge cases

int absolute(int x) {
    if (x < 0) {
        return -x;
    } else {
        return x;
    }
}

int power(int base, int exp) {
    int result;
    int i;
    result = 1;
    for (i = 0; i < exp; i = i + 1) {
        result = result * base;
    }
    return result;
}

int gcd(int a, int b) {
    int temp;
    while (b != 0) {
        temp = b;
        b = a % b;
        a = temp;
    }
    return a;
}

int is_prime(int n) {
    int i;
    if (n <= 1) {
        return 0;
    }
    for (i = 2; i * i <= n; i = i + 1) {
        if (n % i == 0) {
            return 0;
        }
    }
    return 1;
}
