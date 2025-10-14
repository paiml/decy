// DECY-041: Test compound assignment operators
// This file tests +=, -=, *=, /=, %= operators

int increment_by(int value, int amount) {
    value += amount;
    return value;
}

int decrement_by(int value, int amount) {
    value -= amount;
    return value;
}

int multiply_by(int value, int factor) {
    value *= factor;
    return value;
}

int divide_by(int value, int divisor) {
    if (divisor != 0) {
        value /= divisor;
    }
    return value;
}

int modulo_by(int value, int modulus) {
    if (modulus != 0) {
        value %= modulus;
    }
    return value;
}

// Pointer arithmetic with compound assignments
void advance_pointer(int* ptr, int offset) {
    ptr += offset;
}
