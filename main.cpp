#include <iostream>
#include <cstdio>

using namespace std;

long long extract_bits(long long val, int from_id, int length) {
    val >>= from_id;
    val &= ((1LL << length) - 1);
    return val;
}

int main() {
    long long num = 0x48307ffe7fff8000LL;
    cout << num << endl;
    long long off_dst = extract_bits(num, 0, 16) - (1LL << 15);
    printf("off_dst: %lld\n", off_dst);
    long long off_op0 = extract_bits(num, 16, 16) - (1LL << 15);
    printf("off_op0: %lld\n", off_op0);
    long long off_op1 = extract_bits(num, 32, 16) - (1LL << 15);
    printf("off_op1: %lld\n", off_op1);
    long long dst_reg = extract_bits(num, 48, 1);
    printf("dst_reg: %lld\n", dst_reg);
    long long op0_reg = extract_bits(num, 49, 1);
    printf("op0_reg: %lld\n", op0_reg);
    long long op1_src = extract_bits(num, 50, 3);
    printf("op1_src: %lld\n", op1_src);
    long long res_logic = extract_bits(num, 53, 2);
    printf("res_logic: %lld\n", res_logic);
    long long pc_update = extract_bits(num, 55, 3);
    printf("pc_update: %lld\n", pc_update);
    long long ap_update = extract_bits(num, 58, 2);
    printf("ap_update: %lld\n", ap_update);
    long long opcode = extract_bits(num, 60, 3);
    printf("opcode: %lld\n", opcode);
}