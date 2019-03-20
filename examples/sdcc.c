#include <serial_IO.h>

void puts(const char * s) {
    char c;
    while (c = *(s++)) {
        putchar(c);
    }
}

void shutdown() {
    *((char *)0xFFFF) = 1;
}

const char * static_string = "A Test\n";

void main() {
    puts(static_string);
    shutdown();
}
