.org 00H

start:
    lcall print
    ljmp shutdown

print:
    mov 0x99, #'H'
    mov 0x99, #'e'
    mov 0x99, #'l'
    mov 0x99, #'l'
    mov 0x99, #'o'
    mov 0x99, #'\n'
    ret

shutdown:
    mov dptr, #0xFFFF
    mov a, #1
    movx @dptr, a
