// Small fixed-point trigonometry for SVM-C.
// Angle representation: one full turn == 65536 units.
//   0x0000 =   0 degrees
//   0x4000 =  90 degrees
//   0x8000 = 180 degrees
//   0xc000 = 270 degrees
// Results are signed Q15 values stored in u16.
// sin/cos use a corrected parabolic approximation; no table/ROM is required.

include "q15.sc";

u16 sin(u16 angle) {
    u16 x;
    u16 ax;
    u16 one_minus;
    u16 y;
    u16 y2;
    u16 correction;

    // The angle bit-pattern is already Q15 x = angle/pi in [-1,+1).
    x = angle;
    ax = q15_abs(x);
    one_minus = 0x7fff - ax;

    // y ~= 4*|x|*(1-|x|). Product is <= 0.25, so <<2 is safe.
    y = mul_q15(ax, one_minus);
    y <<= 2;
    if (y & 0x8000) { y = 0x7fff; }

    // Common low-cost correction: y += 0.225*(y - y*y).
    y2 = mul_q15(y, y);
    correction = mul_q15(7373, y - y2);
    y += correction;
    if (y & 0x8000) { y = 0x7fff; }

    if (x & 0x8000) { return 0 - y; }
    return y;
}

u16 cos(u16 angle) {
    return sin(angle + 0x4000);
}

u16 tan(u16 angle) {
    return q15_div(sin(angle), cos(angle));
}
