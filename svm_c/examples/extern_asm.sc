// Same C source for every SVM target.
asm_include "interop_demo.asm";
extern asm u16 asm_inc(u16 x);

u16 main() {
    return asm_inc(41);
}
