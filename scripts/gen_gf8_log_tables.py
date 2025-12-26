#!/usr/bin/env python3

_MODULUS = 0b100011011

def _main() -> None:
    exp = [0b1]
    for _ in range(1, 255):
        z = (exp[-1] << 1) ^ exp[-1]
        if (z & (1 << 8)) != 0:
            z ^= _MODULUS
        exp.append(z)
    
    formatted_exp = "[" + ", ".join(f"{value}u8" for value in exp) + "]"
    print(f"const EXP: [u8; 255] = {formatted_exp};")

    log = [255] * 256

    for i in range(255):
        log[exp[i]] = i
    
    formatted_log = "[" + ", ".join(f"{value}u8" for value in log) + "]"
    print(f"const LOG: [u8; 256] = {formatted_log};")


if __name__ == "__main__":
    _main()
