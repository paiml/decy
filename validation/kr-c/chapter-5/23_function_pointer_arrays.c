/* K&R C Chapter 5: Pointer to Function Arrays
 * Arrays of function pointers and dispatch tables
 */

#include <stdio.h>

/* Calculator operations */
int add(int a, int b) { return a + b; }
int subtract(int a, int b) { return a - b; }
int multiply(int a, int b) { return a * b; }
int divide(int a, int b) { return b != 0 ? a / b : 0; }

/* Menu actions */
void action_open(void) { printf("  Opening file...\n"); }
void action_save(void) { printf("  Saving file...\n"); }
void action_close(void) { printf("  Closing file...\n"); }
void action_quit(void) { printf("  Quitting...\n"); }

/* State machine */
typedef enum { STATE_IDLE, STATE_RUNNING, STATE_PAUSED, STATE_STOPPED } State;

void idle_handler(void) { printf("  Idle state\n"); }
void running_handler(void) { printf("  Running state\n"); }
void paused_handler(void) { printf("  Paused state\n"); }
void stopped_handler(void) { printf("  Stopped state\n"); }

int main() {
    /* Array of function pointers - calculator */
    int (*operations[4])(int, int) = {add, subtract, multiply, divide};
    const char *op_names[] = {"+", "-", "*", "/"};

    int x = 10, y = 5;

    printf("Calculator operations on %d and %d:\n", x, y);
    for (int i = 0; i < 4; i++) {
        int result = operations[i](x, y);
        printf("  %d %s %d = %d\n", x, op_names[i], y, result);
    }

    /* Array of function pointers - menu */
    void (*menu[])(void) = {action_open, action_save, action_close, action_quit};
    const char *menu_names[] = {"Open", "Save", "Close", "Quit"};
    int menu_size = sizeof(menu) / sizeof(menu[0]);

    printf("\nMenu dispatch:\n");
    for (int i = 0; i < menu_size; i++) {
        printf("Action %d (%s):\n", i, menu_names[i]);
        menu[i]();
    }

    /* Dispatch table - state machine */
    void (*state_handlers[])(void) = {
        idle_handler, running_handler, paused_handler, stopped_handler
    };

    printf("\nState machine:\n");
    for (State s = STATE_IDLE; s <= STATE_STOPPED; s++) {
        printf("State %d:\n", s);
        state_handlers[s]();
    }

    /* Function pointer table lookup */
    printf("\nDirect function calls:\n");
    int choice = 2;  /* Multiply */
    if (choice >= 0 && choice < 4) {
        printf("Selected operation: %s\n", op_names[choice]);
        int result = operations[choice](x, y);
        printf("Result: %d\n", result);
    }

    /* Pointer to array of function pointers */
    int (**op_ptr)(int, int) = operations;
    printf("\nUsing pointer to function array:\n");
    printf("First operation: %d %s %d = %d\n",
           x, op_names[0], y, op_ptr[0](x, y));

    return 0;
}
