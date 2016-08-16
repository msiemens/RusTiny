.intel_syntax noprefix
.text
.globl func
func:
    enter 0, 0
    mov %ret_slot, rsp
    sub rsi, 4
    mov qword ptr [%ret_slot], 10
    jmp return
return:
    mov %0, qword ptr [%ret_slot]
    mov rax, %0
    leave
    ret
.globl main
main:
    enter 0, 0
    mov %i, rsp
    sub rsi, 4
    mov qword ptr [%i], 2
    mov %2, qword ptr [%i]
    cmp %2, 1
    jne lazy-rhs1
    jmp lazy-next1
    test %1, 1
    je lazy-next1
    jmp lazy-rhs1
lazy-rhs1:
    mov qword ptr [%3], 0
    jmp lazy-next1
lazy-next1:
    phi %0 = (entry-block, %1), (lazy-rhs1, %3)
    test %0, 1
    je conseq1
    jmp next1
conseq1:
    jmp next1
next1:
    leave
    ret