void putchar(char c) {
    *((char *)0x400) = c;
}

void puts(const char * s) {
    while (1) {
        char c = *s;
        if (c == 0) {
            break;
        }
        putchar(c);
        s++;
    }
    /*
    char c;
    while (c = *(s++)) {
        putchar(c);
    }
    */
}

void shutdown() {
    *((char *)0xFFFF) = 1;
}

const char * static_string = "Test";

void main() {
    putchar('A');
    puts(static_string);
    shutdown();
}
