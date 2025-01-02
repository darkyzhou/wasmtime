// A WORD OF CAUTION
//
// This entire file basically needs to be kept in sync with itself. It's not
// really possible to modify just one bit of this file without understanding
// all the other bits. Documentation tries to reference various bits here and
// there but try to make sure to read over everything before tweaking things!

use wasmtime_asm_macros::asm_func;

// fn(top_of_stack(a0): *mut u8)
asm_func!(
    wasmtime_versioned_export_macros::versioned_stringify_ident!(wasmtime_fiber_switch),
    "
        // Note that this order for saving is important since we use CFI directives
        // below to point to where all the saved registers are.
        st.d    $ra,  $sp, -0x08
        st.d    $fp,  $sp, -0x10
        st.d    $s0,  $sp, -0x18
        st.d    $s1,  $sp, -0x20
        st.d    $s2,  $sp, -0x28
        st.d    $s3,  $sp, -0x30
        st.d    $s4,  $sp, -0x38
        st.d    $s5,  $sp, -0x40
        st.d    $s6,  $sp, -0x48
        st.d    $s7,  $sp, -0x50
        st.d    $s8,  $sp, -0x58
        fst.d   $fs0, $sp, -0x60
        fst.d   $fs1, $sp, -0x68
        fst.d   $fs2, $sp, -0x70
        fst.d   $fs3, $sp, -0x78
        fst.d   $fs4, $sp, -0x80
        fst.d   $fs5, $sp, -0x88
        fst.d   $fs6, $sp, -0x90
        fst.d   $fs7, $sp, -0x98
        addi.d  $sp,  $sp, -0xa0

        ld.d    $t0,  $a0, -0x10
        st.d    $sp,  $a0, -0x10
        move    $sp,  $t0

        fld.d   $fs7, $sp,  0x08
        fld.d   $fs6, $sp,  0x10
        fld.d   $fs5, $sp,  0x18
        fld.d   $fs4, $sp,  0x20
        fld.d   $fs3, $sp,  0x28
        fld.d   $fs2, $sp,  0x30
        fld.d   $fs1, $sp,  0x38
        fld.d   $fs0, $sp,  0x40
        ld.d    $s8,  $sp,  0x48
        ld.d    $s7,  $sp,  0x50
        ld.d    $s6,  $sp,  0x58
        ld.d    $s5,  $sp,  0x60
        ld.d    $s4,  $sp,  0x68
        ld.d    $s3,  $sp,  0x70
        ld.d    $s2,  $sp,  0x78
        ld.d    $s1,  $sp,  0x80
        ld.d    $s0,  $sp,  0x88
        ld.d    $fp,  $sp,  0x90
        ld.d    $ra,  $sp,  0x98
        addi.d  $sp,  $sp,  0xa0

        jr      $ra
    ",
);

// fn(
//    top_of_stack(a0): *mut u8,
//    entry_point(a1): extern fn(*mut u8, *mut u8),
//    entry_arg0(a2): *mut u8,
// )
#[rustfmt::skip]
asm_func!(
    wasmtime_versioned_export_macros::versioned_stringify_ident!(wasmtime_fiber_init),
    "
        la.pcrel $t0, {start}
        st.d     $t0, $a0, -0x18 // Store start function in ra slot
        st.d     $a0, $a0, -0x20 // Store stack top in fp slot
        st.d     $a1, $a0, -0x28 // Store entry_point in s0 slot
        st.d     $a2, $a0, -0x30 // Store entry_arg0 in s1 slot

        addi.d   $t0, $a0, -0xb0 // 16 bytes of reserved space per unix.rs + 0xa0 from wasmtime_fiber_switch
        st.d     $t0, $a0, -0x10
        jr       $ra
    ",
    start = sym super::wasmtime_fiber_start,
);

asm_func!(
    wasmtime_versioned_export_macros::versioned_stringify_ident!(wasmtime_fiber_start),
    "
        .cfi_startproc simple
        .cfi_def_cfa_offset 0

        // See https://dwarfstd.org/doc/Dwarf3.pdf for more details on CFA
        .cfi_escape 0x0f,   /* DW_CFA_def_cfa_expression */ \
            4,              /* length                    */ \
            0x53,           /* DW_OP_reg3 ($sp)          */ \
            0x06,           /* DW_OP_deref               */ \
            0x23, 0xa0      /* DW_OP_plus_uconst 0xa0    */

        // Register save locations relative to CFA
        // TODO: We cannot use register names here until rustc catches up with this commit:
        // https://github.com/llvm/llvm-project/commit/3f1e7ef5344c3236bcabf3982dbdc985c43bc078
        .cfi_rel_offset   1,  -0x08   // $ra
        .cfi_rel_offset  22,  -0x10   // $fp
        .cfi_rel_offset  23,  -0x18   // $s0
        .cfi_rel_offset  24,  -0x20   // $s1
        .cfi_rel_offset  25,  -0x28   // $s2
        .cfi_rel_offset  26,  -0x30   // $s3
        .cfi_rel_offset  27,  -0x38   // $s4
        .cfi_rel_offset  28,  -0x40   // $s5
        .cfi_rel_offset  29,  -0x48   // $s6
        .cfi_rel_offset  30,  -0x50   // $s7
        .cfi_rel_offset  31,  -0x58   // $s8
        .cfi_rel_offset  32,  -0x60   // $fs0
        .cfi_rel_offset  33,  -0x68   // $fs1
        .cfi_rel_offset  34,  -0x70   // $fs2
        .cfi_rel_offset  35,  -0x78   // $fs3
        .cfi_rel_offset  36,  -0x80   // $fs4
        .cfi_rel_offset  37,  -0x88   // $fs5
        .cfi_rel_offset  38,  -0x90   // $fs6
        .cfi_rel_offset  39,  -0x98   // $fs7

        // Call entry point with arguments
        move    $a0, $s1        // entry_arg0
        move    $a1, $fp        // stack top
        jirl    $ra, $s0, 0     // call entry_point
        
        // Safety trap
        break   0
        .cfi_endproc
    ",
);
