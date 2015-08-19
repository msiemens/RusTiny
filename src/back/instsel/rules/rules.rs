rules!{
    // %(foo) refers to a register
    // 0(foo) refers to an immediate value
    // @(foo) refers to a static variable
    // FIXME: `load` -> handle statics

    // --- Arithmetics & binary operations  -------------------------------------

    // Addition
    [%(dst) = add %(lhs), %(rhs); ..] => {
        mov $dst, $lhs;
        add $dst, $rhs;
    },
    [%(dst) = add %(lhs), 0(rhs); ..] => {
        mov $dst, $lhs;
        add $dst, $rhs;
    },
    [%(dst) = add 0(lhs), %(rhs)]; ..] => {
        mov $dst, $lhs;
        add $dst, $rhs;
    },

    // Subtraction
    [%(dst) = sub %(lhs), %(rhs); ..] => {
        mov $dst, $lhs;
        sub $dst, $rhs;
    },
    [%(dst) = sub %(lhs), 0(rhs); ..] => {
        mov $dst, $lhs;
        sub $dst, $rhs;
    },
    [%(dst) = sub 0(lhs), %(rhs)]; ..] => {
        mov $dst, $lhs;
        sub $dst, $rhs;
    },

    // Multiplication
    [%(dst) = mul %(lhs), %(rhs); ..] => {
        mov $dst, $lhs;
        imul $dst, $rhs;
    },
    [%(dst) = mul %(lhs), 0(rhs); ..] => {
        // Use the three-operand form
        imu $dst, $lhs, $rhs;
    },
    [%(dst) = mul 0(lhs), %(rhs)]; ..] => {
        // Use the three-operand form
        imu $dst, $rhs, $lhs;
    },

    // Integer division
    [%(dst) = sub %(lhs), %(rhs); ..] => {
        xor rdx, rdx;
        mov rax, $lhs;
        idiv $rhs;
        mov $dst, rax;
    },
    [%(dst) = sub %(lhs), 0(rhs); ..] => {
        mov %(tmp), $rhs;  // Create a temporary virtual register
        xor rdx, rdx;
        mov rax, $lhs;
        idiv $tmp;
        mov $dst, rax;
    },
    [%(dst) = sub 0(lhs), %(rhs)]; ..] => {
        xor rdx, rdx;
        mov rax, $lhs;
        idiv rhs;
        mov $dst, rax;
    },

    // Note: pow will be implemented as an intrinsic

    // Modulo
    // Like div but use the remainder of the division
    [%(dst) = sub %(lhs), %(rhs); ..] => {
        xor rdx, rdx;
        mov rax, $lhs;
        idiv $rhs;
        mov $dst, rdx;
    },
    [%(dst) = sub %(lhs), 0(rhs); ..] => {
        mov %(tmp), $rhs;  // Create a temporary virtual register
        xor rdx, rdx;
        mov rax, $lhs;
        idiv $tmp;
        mov $dst, rdx;
    },
    [%(dst) = sub 0(lhs), %(rhs)]; ..] => {
        xor rdx, rdx;
        mov rax, $lhs;
        idiv rhs;
        mov $dst, rdx;
    },

    // Shift left
    [%(dst) = shl %(lhs), %(rhs); ..] => {
        mov $dst, $lhs;
        mov rcx, $rhs;
        sal $dst, cl;
    },
    [%(dst) = shl %(lhs), 0(rhs); ..] => {
        mov $dst, $lhs;
        sal $dst, $rhs;
    },
    [%(dst) = shl 0(lhs), %(rhs)]; ..] => {
        mov $dst, $lhs;
        mov rcx, $rhs;
        sal $dst, cl;
    },

    // Shift right
    [%(dst) = shr %(lhs), %(rhs); ..] => {
        mov $dst, $lhs;
        mov rcx, $rhs;
        sar $dst, cl;
    },
    [%(dst) = shr %(lhs), 0(rhs); ..] => {
        mov $dst, $lhs;
        sar $dst, $rhs;
    },
    [%(dst) = shr 0(lhs), %(rhs)]; ..] => {
        mov $dst, $lhs;
        mov rcx, $rhs;
        sar $dst, cl;
    },

    // And
    [%(dst) = and %(lhs), %(rhs); ..] => {
        mov $dst, $lhs;
        and $dst, $rhs;
    },
    [%(dst) = and %(lhs), 0(rhs); ..] => {
        mov $dst, $lhs;
        and $dst, $rhs;
    },
    [%(dst) = and 0(lhs), %(rhs)]; ..] => {
        mov $dst, $lhs;
        and $dst, $rhs;
    },

    // Or
    [%(dst) = or %(lhs), %(rhs); ..] => {
        mov $dst, $lhs;
        or $dst, $rhs;
    },
    [%(dst) = or %(lhs), 0(rhs); ..] => {
        mov $dst, $lhs;
        or $dst, $rhs;
    },
    [%(dst) = or 0(lhs), %(rhs)]; ..] => {
        mov $dst, $lhs;
        or $dst, $rhs;
    },

    // Xor
    [%(dst) = xor %(lhs), %(rhs); ..] => {
        mov $dst, $lhs;
        xor $dst, $rhs;
    },
    [%(dst) = xor %(lhs), 0(rhs); ..] => {
        mov $dst, $lhs;
        xor $dst, $rhs;
    },
    [%(dst) = xor 0(lhs), %(rhs)]; ..] => {
        mov $dst, $lhs;
        xor $dst, $rhs;
    },

    // Unary negation
    [%(dst) = not %(item); ..] => {
        mov $dst $item;
        not $dst;
    },

    // --- Comparisons ----------------------------------------------------------

    // Lower than: With branch
    [%(dst) = cmp lt %(lhs), %(rhs); br %(cond), conseq, altern; ..] => {
        cmp $lhs, $rhs;
        jl $conseq;
        jmp $altern;
    },
    [%(dst) = cmp lt %(lhs), 0(rhs); br %(cond), conseq, altern; ..] => {
        cmp $lhs, $rhs;
        jl $conseq;
        jmp $altern;
    },
    [%(dst) = cmp lt 0(lhs), %(rhs); br %(cond), conseq, altern; ..] => {
        // Inverted cmp
        cmp $rhs, $lhs;
        jge $altern;
        jmp $conseq;
    },

    // Lower than: Without branch
    [%(dst) = cmp lt %(lhs), %(rhs); ..] => {
        cmp $lhs, $rhs;
        setl cl;
        and cl, 1;      // Truncate to first bit
        movzx $dst, cl; // Move with zero extension
    },
    [%(dst) = cmp lt %(lhs), 0(rhs); ..] => {
        cmp $lhs, $rhs;
        setl cl;
        and cl, 1;
        movzx $dst, cl;
    },
    [%(dst) = cmp lt 0(lhs), %(rhs); ..] => {
        // Inverted cmp
        cmp $rhs, $lhs;
        setge cl;
        and cl, 1;
        movzx $dst, cl;
    },

    // Lower than or equal: With branch
    [%(dst) = cmp le %(lhs), %(rhs); br %(cond), conseq, altern; ..] => {
        cmp $lhs, $rhs;
        jle $conseq;
        jmp $altern;
    },
    [%(dst) = cmp le %(lhs), 0(rhs); br %(cond), conseq, altern; ..] => {
        cmp $lhs, $rhs;
        jle $conseq;
        jmp $altern;
    },
    [%(dst) = cmp le 0(lhs), %(rhs); br %(cond), conseq, altern; ..] => {
        // Inverted cmp
        cmp $rhs, $lhs;
        jg $altern;
        jmp $conseq;
    },

    // Lower than or equal: Without branch
    [%(dst) = cmp le %(lhs), %(rhs); ..] => {
        cmp $lhs, $rhs;
        setle cl;
        and cl, 1;      // Truncate to first bit
        movzx $dst, cl; // Move with zero extension
    },
    [%(dst) = cmp le %(lhs), 0(rhs); ..] => {
        cmp $lhs, $rhs;
        setle cl;
        and cl, 1;
        movzx $dst, cl;
    },
    [%(dst) = cmp le 0(lhs), %(rhs); ..] => {
        // Inverted cmp
        cmp $rhs, $lhs;
        setg cl;
        and cl, 1;
        movzx $dst, cl;
    },

    // Greater than or equal: With branch
    [%(dst) = cmp ge %(lhs), %(rhs); br %(cond), conseq, altern; ..] => {
        cmp $lhs, $rhs;
        jge $conseq;
        jmp $altern;
    },
    [%(dst) = cmp ge %(lhs), 0(rhs); br %(cond), conseq, altern; ..] => {
        cmp $lhs, $rhs;
        jge $conseq;
        jmp $altern;
    },
    [%(dst) = cmp ge 0(lhs), %(rhs); br %(cond), conseq, altern; ..] => {
        // Inverted cmp
        cmp $rhs, $lhs;
        jl $altern;
        jmp $conseq;
    },

    // Greater than or equal: Without branch
    [%(dst) = cmp ge %(lhs), %(rhs); ..] => {
        cmp $lhs, $rhs;
        setge cl;
        and cl, 1;      // Truncate to first bit
        movzx $dst, cl; // Move with zero extension
    },
    [%(dst) = cmp ge %(lhs), 0(rhs); ..] => {
        cmp $lhs, $rhs;
        setge cl;
        and cl, 1;
        movzx $dst, cl;
    },
    [%(dst) = cmp ge 0(lhs), %(rhs); ..] => {
        // Inverted cmp
        cmp $rhs, $lhs;
        setl cl;
        and cl, 1;
        movzx $dst, cl;
    },

    // Greater than: With branch
    [%(dst) = cmp gt %(lhs), %(rhs); br %(cond), conseq, altern; ..] => {
        cmp $lhs, $rhs;
        jg $conseq;
        jmp $altern;
    },
    [%(dst) = cmp gt %(lhs), 0(rhs); br %(cond), conseq, altern; ..] => {
        cmp $lhs, $rhs;
        jg $conseq;
        jmp $altern;
    },
    [%(dst) = cmp gt 0(lhs), %(rhs); br %(cond), conseq, altern; ..] => {
        // Inverted cmp
        cmp $rhs, $lhs;
        jle $altern;
        jmp $conseq;
    },

    // Greater than: Without branch
    [%(dst) = cmp gt %(lhs), %(rhs); ..] => {
        cmp $lhs, $rhs;
        setg cl;
        and cl, 1;      // Truncate to first bit
        movzx $dst, cl; // Move with zero extension
    },
    [%(dst) = cmp gt %(lhs), 0(rhs); ..] => {
        cmp $lhs, $rhs;
        setg cl;
        and cl, 1;
        movzx $dst, cl;
    },
    [%(dst) = cmp gt 0(lhs), %(rhs); ..] => {
        // Inverted cmp
        cmp $rhs, $lhs;
        setle cl;
        and cl, 1;
        movzx $dst, cl;
    },

    // Equality: With branch
    [%(dst) = cmp eq %(lhs), %(rhs); br %(cond), conseq, altern; ..] => {
        cmp $lhs, $rhs;
        je $conseq;
        jmp $altern;
    },
    [%(dst) = cmp eq %(lhs), 0(rhs); br %(cond), conseq, altern; ..] => {
        cmp $lhs, $rhs;
        je $conseq;
        jmp $altern;
    },
    [%(dst) = cmp eq 0(lhs), %(rhs); br %(cond), conseq, altern; ..] => {
        // Inverted cmp
        cmp $rhs, $lhs;
        jne $altern;
        jmp $conseq;
    },

    // Equality: Without branch
    [%(dst) = cmp eq %(lhs), %(rhs); ..] => {
        cmp $lhs, $rhs;
        sete cl;
        and cl, 1;      // Truncate to first bit
        movzx $dst, cl; // Move with zero extension
    },
    [%(dst) = cmp eq %(lhs), 0(rhs); ..] => {
        cmp $lhs, $rhs;
        sete cl;
        and cl, 1;
        movzx $dst, cl;
    },
    [%(dst) = cmp eq 0(lhs), %(rhs); ..] => {
        // Inverted cmp
        cmp $rhs, $lhs;
        setne cl;
        and cl, 1;
        movzx $dst, cl;
    },

    // Inequality: With branch
    [%(dst) = cmp eq %(lhs), %(rhs); br %(cond), conseq, altern; ..] => {
        cmp $lhs, $rhs;
        jne $conseq;
        jmp $altern;
    },
    [%(dst) = cmp eq %(lhs), 0(rhs); br %(cond), conseq, altern; ..] => {
        cmp $lhs, $rhs;
        jne $conseq;
        jmp $altern;
    },
    [%(dst) = cmp eq 0(lhs), %(rhs); br %(cond), conseq, altern; ..] => {
        // Inverted cmp
        cmp $rhs, $lhs;
        je $altern;
        jmp $conseq;
    },

    // Inequality: Without branch
    [%(dst) = cmp eq %(lhs), %(rhs); ..] => {
        cmp $lhs, $rhs;
        setne cl;
        and cl, 1;      // Truncate to first bit
        movzx $dst, cl; // Move with zero extension
    },
    [%(dst) = cmp eq %(lhs), 0(rhs); ..] => {
        cmp $lhs, $rhs;
        setne cl;
        and cl, 1;
        movzx $dst, cl;
    },
    [%(dst) = cmp eq 0(lhs), %(rhs); ..] => {
        // Inverted cmp
        cmp $rhs, $lhs;
        sete cl;
        and cl, 1;
        movzx $dst, cl;
    },

    // TODO: Special case: a == 0 => jz

    // --- Alloca/load/store ----------------------------------------------------

    [%(dst) = alloca; ..] => {
        // TODO: Collect allocas and replace usage with indirect arguments
        mov $dst, rsp;
        sub rsi, 4;
    },
    [%(dst) = load %(src); ..] => {
        mov $dst, qword ptr [src];
    },
    [store %(val), %(dst); ..] => {
        mov qword ptr [dst], $val;
    },
    [store 0(val), %(dst); ..] => {
        mov qword ptr [dst], $val;
    },

    // --- Call -----------------------------------------------------------------

    [%dst = call func [args ...]] -> {
        //
    }

    // TODO: How are phi nodes handeled?
    // TODO: return/branch/jump
    // FIXME: How is neg handled?
}

/*
Result:

pub fn trans_instr(instr: &mut [ir::Instruction],
               last: &ir::ControlFlowInstruction,
               code: &mut machine::MachineCode)
               -> usize
{
    match instr {
        ...
    }
}