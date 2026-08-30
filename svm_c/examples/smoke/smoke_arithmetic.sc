include "arithmetic_int.sc";
include "random.sc";

u16 main() {
    puts("ARITHMETIC SMOKE");
    if (abs(0xff9c)!=100) { puts("FAIL ABS"); return 1; }
    if (min(3,7)!=3 || max(3,7)!=7) { puts("FAIL MINMAX"); return 2; }
    if (clamp(20,3,10)!=10 || clamp(1,3,10)!=3) { puts("FAIL CLAMP"); return 3; }
    if (isqrt(0)!=0 || isqrt(1)!=1 || isqrt(15)!=3 || isqrt(65535)!=255) { puts("FAIL ISQRT"); return 4; }
    if (powu(3,5)!=243) { puts("FAIL POWU"); return 5; }
    if (gcd(84,30)!=6) { puts("FAIL GCD"); return 6; }
    if (lcm(21,6)!=42) { puts("FAIL LCM"); return 7; }
    srand(1);
    if (rand()!=19511) { puts("FAIL RAND 1"); return 8; }
    if (rand()!=30543) { puts("FAIL RAND 2"); return 9; }
    if (rand()!=10098) { puts("FAIL RAND 3"); return 10; }
    srand(0); if (rand()!=19511) { puts("FAIL SRAND ZERO"); return 11; }
    puts("OK"); return 0;
}
