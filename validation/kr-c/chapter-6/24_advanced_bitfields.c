/* K&R C Chapter 6: Advanced Bit-Fields
 * K&R §6.9: Bit-field usage patterns
 * Tests complex bit-field applications
 */

#include <stdio.h>
#include <stdint.h>
#include <string.h>

/* Network packet header with bit-fields */
typedef struct {
    unsigned int version:4;       /* 4 bits */
    unsigned int ihl:4;           /* 4 bits */
    unsigned int dscp:6;          /* 6 bits */
    unsigned int ecn:2;           /* 2 bits */
    unsigned int total_length:16; /* 16 bits */
    unsigned int identification:16;
    unsigned int flags:3;
    unsigned int fragment_offset:13;
    unsigned int ttl:8;
    unsigned int protocol:8;
    unsigned int checksum:16;
} IPHeader;

/* CPU flags register */
typedef struct {
    unsigned int carry:1;
    unsigned int zero:1;
    unsigned int interrupt:1;
    unsigned int direction:1;
    unsigned int overflow:1;
    unsigned int sign:1;
    unsigned int trap:1;
    unsigned int reserved:25;
} CPUFlags;

/* File permissions */
typedef struct {
    unsigned int user_read:1;
    unsigned int user_write:1;
    unsigned int user_execute:1;
    unsigned int group_read:1;
    unsigned int group_write:1;
    unsigned int group_execute:1;
    unsigned int other_read:1;
    unsigned int other_write:1;
    unsigned int other_execute:1;
    unsigned int setuid:1;
    unsigned int setgid:1;
    unsigned int sticky:1;
} FilePermissions;

/* Graphics pixel (RGB565 format) */
typedef struct {
    unsigned int blue:5;
    unsigned int green:6;
    unsigned int red:5;
} RGB565;

/* Device control register */
typedef struct {
    unsigned int enable:1;
    unsigned int mode:2;
    unsigned int speed:3;
    unsigned int duplex:1;
    unsigned int auto_negotiate:1;
    unsigned int reserved:24;
} DeviceControl;

/* Date packed into 16 bits */
typedef struct {
    unsigned int day:5;     /* 1-31 */
    unsigned int month:4;   /* 1-12 */
    unsigned int year:7;    /* 0-127 (offset from 2000) */
} PackedDate;

/* IP header demo */
void ip_header_demo(void) {
    printf("=== IP Header Demo ===\n");

    IPHeader header = {0};
    header.version = 4;
    header.ihl = 5;
    header.total_length = 60;
    header.ttl = 64;
    header.protocol = 6;  /* TCP */

    printf("IP Header:\n");
    printf("  Version:       %u\n", header.version);
    printf("  IHL:           %u\n", header.ihl);
    printf("  Total Length:  %u\n", header.total_length);
    printf("  TTL:           %u\n", header.ttl);
    printf("  Protocol:      %u (TCP)\n", header.protocol);
    printf("  Size:          %zu bytes\n", sizeof(IPHeader));
    printf("\n");
}

/* CPU flags demo */
void cpu_flags_demo(void) {
    printf("=== CPU Flags Demo ===\n");

    CPUFlags flags = {0};

    printf("Initial flags: ");
    printf("C=%u Z=%u I=%u D=%u O=%u S=%u T=%u\n",
           flags.carry, flags.zero, flags.interrupt,
           flags.direction, flags.overflow, flags.sign, flags.trap);

    /* Simulate arithmetic operation */
    flags.zero = 1;
    flags.carry = 0;

    printf("After operation: ");
    printf("C=%u Z=%u I=%u D=%u O=%u S=%u T=%u\n",
           flags.carry, flags.zero, flags.interrupt,
           flags.direction, flags.overflow, flags.sign, flags.trap);

    printf("Size: %zu bytes\n", sizeof(CPUFlags));
    printf("\n");
}

/* File permissions demo */
void file_permissions_demo(void) {
    printf("=== File Permissions Demo ===\n");

    FilePermissions perms = {0};

    /* Set rwxr-xr-- (754) */
    perms.user_read = 1;
    perms.user_write = 1;
    perms.user_execute = 1;
    perms.group_read = 1;
    perms.group_execute = 1;
    perms.other_read = 1;

    printf("Permissions:\n");
    printf("  User:   %c%c%c\n",
           perms.user_read ? 'r' : '-',
           perms.user_write ? 'w' : '-',
           perms.user_execute ? 'x' : '-');
    printf("  Group:  %c%c%c\n",
           perms.group_read ? 'r' : '-',
           perms.group_write ? 'w' : '-',
           perms.group_execute ? 'x' : '-');
    printf("  Other:  %c%c%c\n",
           perms.other_read ? 'r' : '-',
           perms.other_write ? 'w' : '-',
           perms.other_execute ? 'x' : '-');

    /* Convert to octal */
    int octal = (perms.user_read << 2 | perms.user_write << 1 | perms.user_execute) << 6 |
                (perms.group_read << 2 | perms.group_write << 1 | perms.group_execute) << 3 |
                (perms.other_read << 2 | perms.other_write << 1 | perms.other_execute);
    printf("  Octal:  0%o\n", octal);
    printf("  Size:   %zu bytes\n", sizeof(FilePermissions));
    printf("\n");
}

/* RGB565 color demo */
void rgb565_demo(void) {
    printf("=== RGB565 Color Demo ===\n");

    /* Red */
    RGB565 red = {0, 0, 31};
    /* Green */
    RGB565 green = {0, 63, 0};
    /* Blue */
    RGB565 blue = {31, 0, 0};
    /* White */
    RGB565 white = {31, 63, 31};

    printf("RGB565 colors:\n");
    printf("  Red:   R=%u G=%u B=%u\n", red.red, red.green, red.blue);
    printf("  Green: R=%u G=%u B=%u\n", green.red, green.green, green.blue);
    printf("  Blue:  R=%u G=%u B=%u\n", blue.red, blue.green, blue.blue);
    printf("  White: R=%u G=%u B=%u\n", white.red, white.green, white.blue);
    printf("  Size:  %zu bytes (16-bit color)\n", sizeof(RGB565));
    printf("\n");
}

/* Device control demo */
void device_control_demo(void) {
    printf("=== Device Control Register ===\n");

    DeviceControl ctrl = {0};

    /* Configure device */
    ctrl.enable = 1;
    ctrl.mode = 2;         /* Full duplex */
    ctrl.speed = 5;        /* 1000 Mbps */
    ctrl.duplex = 1;       /* Full */
    ctrl.auto_negotiate = 1;

    printf("Device configuration:\n");
    printf("  Enabled:        %s\n", ctrl.enable ? "Yes" : "No");
    printf("  Mode:           %u\n", ctrl.mode);
    printf("  Speed:          %u\n", ctrl.speed);
    printf("  Duplex:         %s\n", ctrl.duplex ? "Full" : "Half");
    printf("  Auto-negotiate: %s\n", ctrl.auto_negotiate ? "Yes" : "No");
    printf("  Size:           %zu bytes\n", sizeof(DeviceControl));
    printf("\n");
}

/* Packed date demo */
void packed_date_demo(void) {
    printf("=== Packed Date Demo ===\n");

    PackedDate today = {15, 11, 24};  /* November 15, 2024 */

    printf("Packed date:\n");
    printf("  Day:   %u\n", today.day);
    printf("  Month: %u\n", today.month);
    printf("  Year:  %u (20%02u)\n", today.year, today.year);
    printf("  Size:  %zu bytes (fits in 16 bits)\n", sizeof(PackedDate));
    printf("\n");
}

/* Bit-field manipulation */
void bitfield_manipulation(void) {
    printf("=== Bit-Field Manipulation ===\n");

    typedef struct {
        unsigned int a:4;
        unsigned int b:4;
        unsigned int c:4;
        unsigned int d:4;
    } Nibbles;

    Nibbles n = {0xA, 0xB, 0xC, 0xD};

    printf("Nibbles: a=%X b=%X c=%X d=%X\n", n.a, n.b, n.c, n.d);

    /* Swap nibbles */
    unsigned int temp = n.a;
    n.a = n.d;
    n.d = temp;

    printf("After swap: a=%X b=%X c=%X d=%X\n", n.a, n.b, n.c, n.d);
    printf("\n");
}

/* Bit-field limitations */
void bitfield_limitations(void) {
    printf("=== Bit-Field Limitations ===\n");

    typedef struct {
        unsigned int field1:10;
        unsigned int field2:10;
        unsigned int field3:12;
    } BitFields;

    BitFields bf = {1023, 1023, 4095};

    printf("Bit-field values:\n");
    printf("  field1: %u (max: 1023)\n", bf.field1);
    printf("  field2: %u (max: 1023)\n", bf.field2);
    printf("  field3: %u (max: 4095)\n", bf.field3);

    /* Test overflow */
    bf.field1 = 1024;  /* Overflow - only 10 bits */
    printf("After overflow:\n");
    printf("  field1: %u (wrapped to 0)\n", bf.field1);

    printf("\nLimitations:\n");
    printf("  - Cannot take address of bit-field\n");
    printf("  - Bit-field order is implementation-defined\n");
    printf("  - Cannot have arrays of bit-fields\n");
    printf("  - Limited portability\n");
    printf("\n");
}

/* Bit-field vs bit masking */
void bitfield_vs_masking(void) {
    printf("=== Bit-Field vs Bit Masking ===\n");

    /* Bit-field approach */
    typedef struct {
        unsigned int value:8;
        unsigned int flags:8;
    } BitFieldReg;

    BitFieldReg bf_reg = {0x42, 0xAB};

    printf("Bit-field approach:\n");
    printf("  Value: 0x%02X\n", bf_reg.value);
    printf("  Flags: 0x%02X\n", bf_reg.flags);

    /* Bit masking approach */
    uint16_t mask_reg = 0x42AB;

    uint8_t value = mask_reg & 0xFF;
    uint8_t flags = (mask_reg >> 8) & 0xFF;

    printf("Bit masking approach:\n");
    printf("  Value: 0x%02X\n", value);
    printf("  Flags: 0x%02X\n", flags);

    printf("\nBit-fields: More readable, compiler-dependent\n");
    printf("Bit masking: Portable, explicit control\n");
    printf("\n");
}

int main() {
    printf("=== Advanced Bit-Fields ===\n\n");

    ip_header_demo();
    cpu_flags_demo();
    file_permissions_demo();
    rgb565_demo();
    device_control_demo();
    packed_date_demo();
    bitfield_manipulation();
    bitfield_limitations();
    bitfield_vs_masking();

    printf("Bit-field use cases:\n");
    printf("  - Hardware register access\n");
    printf("  - Network protocol headers\n");
    printf("  - Packed data structures\n");
    printf("  - Memory-efficient flags\n");
    printf("  - Graphics/image formats\n");
    printf("\nBest practices:\n");
    printf("  - Use for hardware/protocol specifications\n");
    printf("  - Document bit ordering assumptions\n");
    printf("  - Consider portability constraints\n");
    printf("  - Test on target platforms\n");

    return 0;
}
