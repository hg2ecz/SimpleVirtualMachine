// Target-neutral raw memory/MMIO example.
// Exact addresses are platform-level constants; no target ISA syntax appears here.
fn main() -> u16 {
    vstore8(0xff00, 65);
    return vload8(0xff01);
}
