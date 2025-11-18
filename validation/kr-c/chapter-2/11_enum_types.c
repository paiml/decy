/* K&R C Chapter 2.3: Constants - Enumeration
 * Page 39-40
 * Enumeration constants
 */

#include <stdio.h>

/* Day of week enumeration */
enum day {
    MONDAY = 1,
    TUESDAY,
    WEDNESDAY,
    THURSDAY,
    FRIDAY,
    SATURDAY,
    SUNDAY
};

/* Month enumeration */
enum month {
    JAN = 1, FEB, MAR, APR, MAY, JUN,
    JUL, AUG, SEP, OCT, NOV, DEC
};

/* Boolean enumeration */
enum boolean {
    FALSE = 0,
    TRUE = 1
};

const char *day_name(enum day d) {
    switch (d) {
        case MONDAY:    return "Monday";
        case TUESDAY:   return "Tuesday";
        case WEDNESDAY: return "Wednesday";
        case THURSDAY:  return "Thursday";
        case FRIDAY:    return "Friday";
        case SATURDAY:  return "Saturday";
        case SUNDAY:    return "Sunday";
        default:        return "Unknown";
    }
}

int main() {
    enum day today = FRIDAY;
    enum month current_month = NOV;
    enum boolean is_weekend;

    printf("Today is: %s (%d)\n", day_name(today), today);
    printf("Current month: %d\n", current_month);

    is_weekend = (today == SATURDAY || today == SUNDAY) ? TRUE : FALSE;
    printf("Is weekend: %s\n", is_weekend ? "Yes" : "No");

    /* Enum arithmetic */
    enum day tomorrow = today + 1;
    printf("Tomorrow is: %s (%d)\n", day_name(tomorrow), tomorrow);

    /* Loop through days */
    printf("\nDays of week:\n");
    for (enum day d = MONDAY; d <= SUNDAY; d++) {
        printf("  %d: %s\n", d, day_name(d));
    }

    return 0;
}
