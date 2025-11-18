/* K&R C Chapter 6: Structure Serialization
 * Writing/reading structures to/from files
 */

#include <stdio.h>
#include <string.h>

typedef struct {
    int id;
    char name[50];
    float balance;
} Account;

int save_accounts(const char *filename, Account *accounts, int n) {
    FILE *fp = fopen(filename, "wb");
    if (!fp) {
        perror("Cannot open file for writing");
        return 0;
    }

    /* Write count */
    fwrite(&n, sizeof(int), 1, fp);

    /* Write accounts */
    fwrite(accounts, sizeof(Account), n, fp);

    fclose(fp);
    return 1;
}

int load_accounts(const char *filename, Account *accounts, int max) {
    FILE *fp = fopen(filename, "rb");
    if (!fp) {
        perror("Cannot open file for reading");
        return 0;
    }

    /* Read count */
    int n;
    if (fread(&n, sizeof(int), 1, fp) != 1) {
        fclose(fp);
        return 0;
    }

    if (n > max)
        n = max;

    /* Read accounts */
    int read = fread(accounts, sizeof(Account), n, fp);

    fclose(fp);
    return read;
}

void print_account(Account *acc) {
    printf("  ID:%d %-30s Balance:$%.2f\n",
           acc->id, acc->name, acc->balance);
}

int main() {
    const char *filename = "accounts.dat";

    /* Create accounts */
    Account accounts[] = {
        {1001, "Alice Johnson", 5000.00},
        {1002, "Bob Smith", 3500.50},
        {1003, "Charlie Brown", 12000.75}
    };
    int n = sizeof(accounts) / sizeof(accounts[0]);

    printf("=== Structure Serialization ===\n\n");

    /* Save to file */
    printf("Saving %d accounts to '%s'...\n", n, filename);
    if (save_accounts(filename, accounts, n)) {
        printf("Saved successfully\n\n");
    } else {
        printf("Save failed\n");
        return 1;
    }

    /* Modify in-memory data */
    accounts[0].balance = 9999.99;
    accounts[1].balance = 8888.88;
    accounts[2].balance = 7777.77;

    printf("Modified in-memory data:\n");
    for (int i = 0; i < n; i++)
        print_account(&accounts[i]);
    printf("\n");

    /* Load from file */
    Account loaded[10];
    int loaded_count = load_accounts(filename, loaded, 10);

    printf("Loaded %d accounts from '%s':\n", loaded_count, filename);
    for (int i = 0; i < loaded_count; i++)
        print_account(&loaded[i]);

    printf("\nFile size: %ld bytes\n",
           sizeof(int) + n * sizeof(Account));
    printf("  Header: %zu bytes\n", sizeof(int));
    printf("  Per account: %zu bytes\n", sizeof(Account));

    return 0;
}
