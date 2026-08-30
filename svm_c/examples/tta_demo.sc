// Compile with: svm-c --target tta -O2 svm_c/examples/tta_demo.sc tta_demo.svt
u16 main() {
    u16 a;
    u16 b;
    a = 10;
    b = 20;
    return (a + b) ^ 3;
}
