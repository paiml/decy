/* K&R C Chapter 6: Nested Structures
 * Complex nested structure definitions
 */

#include <stdio.h>
#include <string.h>

struct date {
    int day;
    int month;
    int year;
};

struct address {
    char street[100];
    char city[50];
    char state[3];
    int zip;
};

struct person {
    char name[50];
    struct date birthdate;
    struct address home_address;
    struct address work_address;
};

struct company {
    char name[100];
    struct address headquarters;
    struct person ceo;
    int employee_count;
};

void print_date(struct date d) {
    printf("%02d/%02d/%04d", d.month, d.day, d.year);
}

void print_address(struct address addr) {
    printf("%s, %s, %s %05d", addr.street, addr.city, addr.state, addr.zip);
}

void print_person(struct person p) {
    printf("Name: %s\n", p.name);
    printf("  Birth date: ");
    print_date(p.birthdate);
    printf("\n");
    printf("  Home: ");
    print_address(p.home_address);
    printf("\n");
    printf("  Work: ");
    print_address(p.work_address);
    printf("\n");
}

int main() {
    /* Initialize nested structure */
    struct person employee = {
        .name = "Alice Johnson",
        .birthdate = {15, 6, 1990},
        .home_address = {
            .street = "123 Maple St",
            .city = "Springfield",
            .state = "IL",
            .zip = 62701
        },
        .work_address = {
            .street = "456 Oak Ave",
            .city = "Springfield",
            .state = "IL",
            .zip = 62702
        }
    };

    printf("=== Employee Information ===\n");
    print_person(employee);

    /* Deeply nested structure */
    struct company tech_corp = {
        .name = "TechCorp Inc.",
        .headquarters = {
            .street = "789 Tech Blvd",
            .city = "San Jose",
            .state = "CA",
            .zip = 95110
        },
        .ceo = {
            .name = "Bob Smith",
            .birthdate = {20, 3, 1975},
            .home_address = {
                .street = "999 Executive Dr",
                .city = "Palo Alto",
                .state = "CA",
                .zip = 94301
            },
            .work_address = {
                .street = "789 Tech Blvd",
                .city = "San Jose",
                .state = "CA",
                .zip = 95110
            }
        },
        .employee_count = 5000
    };

    printf("\n=== Company Information ===\n");
    printf("Company: %s\n", tech_corp.name);
    printf("Headquarters: ");
    print_address(tech_corp.headquarters);
    printf("\n");
    printf("Employees: %d\n", tech_corp.employee_count);
    printf("\nCEO Information:\n");
    print_person(tech_corp.ceo);

    /* Accessing deeply nested members */
    printf("\n=== Accessing Nested Members ===\n");
    printf("CEO name: %s\n", tech_corp.ceo.name);
    printf("CEO birth year: %d\n", tech_corp.ceo.birthdate.year);
    printf("CEO home city: %s\n", tech_corp.ceo.home_address.city);

    /* Modifying nested members */
    tech_corp.ceo.birthdate.year = 1976;
    strcpy(tech_corp.ceo.home_address.city, "Mountain View");

    printf("\nAfter modification:\n");
    printf("CEO birth year: %d\n", tech_corp.ceo.birthdate.year);
    printf("CEO home city: %s\n", tech_corp.ceo.home_address.city);

    return 0;
}
