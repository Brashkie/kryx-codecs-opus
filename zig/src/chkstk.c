/*
 * chkstk.c — Provide __chkstk_ms for the MSVC link.
 *
 * Zig 0.14.x, compiling C for x86_64-windows-msvc, emits calls to a
 * GCC/MinGW-style stack-probe helper (__chkstk_ms) for functions with large
 * stack frames — which libopus' DSP code has plenty of. That symbol does not
 * exist in the MSVC runtime (MSVC uses __chkstk), so the final Rust link fails
 * with "unresolved external ___chkstk_ms".
 *
 * __chkstk_ms only probes (touches) each page of the requested stack frame to
 * grow the stack safely; it does NOT move the stack pointer itself. This is
 * the canonical libgcc/compiler-rt implementation. Providing it makes the
 * static archive self-contained on Windows x86-64.
 *
 * Compiled to nothing on non-Windows / non-x86_64 targets.
 */

#if defined(_WIN32) && defined(__x86_64__)

/*
 * The assembler symbol name. On x86-64 Windows the C symbol `__chkstk_ms`
 * is emitted by the compiler backend as `__chkstk_ms`; the MSVC linker error
 * shows it as `___chkstk_ms` due to its own leading-underscore convention.
 * Defining the C function `__chkstk_ms` produces the symbol the linker wants.
 */
__attribute__((used))
void __chkstk_ms(void)
{
    __asm__ volatile(
        "push %%rcx\n\t"
        "push %%rax\n\t"
        "cmp  $0x1000, %%rax\n\t"
        "lea  24(%%rsp), %%rcx\n\t"
        "jb   1f\n\t"
        "2:\n\t"
        "sub  $0x1000, %%rcx\n\t"
        "orq  $0, (%%rcx)\n\t"
        "sub  $0x1000, %%rax\n\t"
        "cmp  $0x1000, %%rax\n\t"
        "ja   2b\n\t"
        "1:\n\t"
        "sub  %%rax, %%rcx\n\t"
        "orq  $0, (%%rcx)\n\t"
        "pop  %%rax\n\t"
        "pop  %%rcx\n\t"
        "ret\n\t"
        ::: "memory"
    );
}

#endif
