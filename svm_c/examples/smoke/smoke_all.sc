include "arithmetic.sc";

void s32(u16 p,u16 lo,u16 hi){store16(p,lo);store16(p+2,hi);}
u16 e32(u16 p,u16 lo,u16 hi){return load16(p)==lo && load16(p+2)==hi;}
u16 e64lo(u16 p,u16 w0,u16 w1){return load16(p)==w0&&load16(p+2)==w1;}
u16 e64hi(u16 p,u16 w2,u16 w3){return load16(p+4)==w2&&load16(p+6)==w3;}

u16 main(){
    u32 a;u32 b;u32 c;u32 q;u32 r;u64 p;u16 h;
    puts("ALL NUMERIC SMOKE");

    if((0xffff+1)!=0||(1000/7)!=142||(1000%7)!=6){puts("FAIL SCALAR");return 1;}
    if(i8_sext(0x80)!=0xff80||i16_div(0xff9c,7)!=0xfff2||!i16_lt(0xffff,1)){puts("FAIL SIGNED16");return 2;}

    s32(&a,0xffff,0xffff);s32(&b,1,0);u32_add(&c,&a,&b);
    if(!e32(&c,0,0)){puts("FAIL U32 ADD");return 3;}
    s32(&a,1,1);u32_shr1(&c,&a);
    if(!e32(&c,0x8000,0)){puts("FAIL U32 SHR1");return 4;}
    s32(&a,0x86a0,1);s32(&b,300,0);u32_divmod(&q,&r,&a,&b);
    if(!e32(&q,333,0)||!e32(&r,100,0)){puts("FAIL U32 DIVMOD");return 5;}
    s32(&a,2,1);s32(&b,4,3);u32_mul_u64(&p,&a,&b);
    if(!e64lo(&p,0x0008,0x000a)||!e64hi(&p,0x0003,0)){puts("FAIL U64 PRODUCT");return 6;}

    if(q15_mul(0x4000,0x4000)!=0x2000||q15_div(0x2000,0x4000)!=0x4000){puts("FAIL Q15");return 7;}
    if(sin(0)!=0||q15_abs(sin(0x4000)-0x7fff)>4||q15_abs(cos(0)-0x7fff)>4){puts("FAIL TRIG");return 8;}

    if(f16_add(0x3c00,0x3800)!=0x3e00||f16_mul(0x3c00,0x3800)!=0x3800){puts("FAIL F16");return 9;}
    h=f16_from_u16(10000);if(h!=0x70e2||f16_to_u16(h)!=10000){puts("FAIL F16 CONV");return 10;}

    s32(&a,0,0x3f80);s32(&b,0,0x3f00);f32_add(&c,&a,&b);
    if(!e32(&c,0,0x3fc0)){puts("FAIL F32 ADD");return 11;}
    f32_mul(&c,&a,&b);if(!e32(&c,0,0x3f00)){puts("FAIL F32 MUL");return 12;}
    f32_from_u16(&c,10000);if(!e32(&c,0x4000,0x461c)||f32_to_u16(&c)!=10000){puts("FAIL F32 CONV");return 13;}

    if(isqrt(65535)!=255||powu(3,5)!=243||gcd(84,30)!=6||lcm(21,6)!=42){puts("FAIL ARITH");return 14;}
    srand(1);if(rand()!=19511||rand()!=30543||rand()!=10098){puts("FAIL RAND");return 15;}

    puts("OK");return 0;
}
