#!/usr/bin/env python3
"""Reproduce local arbitrary-bit SHA-512 fixtures for engineering review.

FIPS 180-4 (August 2015), §§4.1.3, 4.2.3, 5.1.2, 5.2.2, 5.3.5, 6.4.2.
A text bitstring in MSb-first byte order supplies the message. The reference
pads the whole string, uses a flat 80-word schedule and integer arithmetic
modulo 2**64. The IV is derived with integer square roots. The round constants
use the exact integer derivation in the adjacent verify_round_constants.py.
No Rust implementation or tables are used in fixture generation.

hashlib verifies byte-aligned cases. Its interface cannot check arbitrary
bit lengths. The fixtures are local development evidence and the script is
run manually. Use --emit to print candidate fixture rows for review.
"""

import argparse
import hashlib
from math import isqrt
from pathlib import Path

from verify_round_constants import first_primes, round_constant


MASK = (1 << 64) - 1
LENGTHS = (
    0, 1, 7, 8, 9, 63, 127, 511, 767, 894, 895, 896, 897,
    1022, 1023, 1024, 1025, 1537, 2047, 2048, 2049,
)
SOURCE_OFFSET = 3
SOURCE_BYTES = 257

PRIMES = first_primes(80)
INITIAL = [isqrt(p << 128) & MASK for p in PRIMES[:8]]
CONSTANTS = [round_constant(i, p) for i, p in enumerate(PRIMES)]


def rotate(word, count):
    upper, lower = divmod(word, 1 << count)
    return upper | (lower << (64 - count))


def digest(bits):
    length = len(bits)
    padded = bits + "1" + "0" * ((895 - length) % 1024) + f"{length:0128b}"
    state = INITIAL.copy()
    for offset in range(0, len(padded), 1024):
        words = [int(padded[i:i + 64], 2) for i in range(offset, offset + 1024, 64)]
        for t in range(16, 80):
            x, y = words[t - 15], words[t - 2]
            s0 = rotate(x, 1) ^ rotate(x, 8) ^ (x >> 7)
            s1 = rotate(y, 19) ^ rotate(y, 61) ^ (y >> 6)
            words.append((s0 + s1 + words[t - 16] + words[t - 7]) & MASK)

        a, b, c, d, e, f, g, h = state
        for word, constant in zip(words, CONSTANTS):
            choose = (e & f) | ((MASK ^ e) & g)
            majority = (a & b) ^ (a & c) ^ (b & c)
            s0 = rotate(a, 28) ^ rotate(a, 34) ^ rotate(a, 39)
            s1 = rotate(e, 14) ^ rotate(e, 18) ^ rotate(e, 41)
            total = (word + constant + h + s1 + choose) & MASK
            a, b, c, d, e, f, g, h = (
                (total + s0 + majority) & MASK, a, b, c, (d + total) & MASK, e, f, g
            )
        state = [(old + new) & MASK for old, new in zip(state, (a, b, c, d, e, f, g, h))]
    return "".join(f"{word:016x}" for word in state)


def check_hashlib(bits, expected):
    message = bytes(int(bits[i:i + 8], 2) for i in range(0, len(bits), 8))
    observed = hashlib.sha512(message).hexdigest()
    if observed != expected:
        raise SystemExit(f"hashlib mismatch at length {len(bits)}: {observed} != {expected}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--emit", action="store_true", help="print candidate rows")
    args = parser.parse_args()
    source = "".join(f"{(37 * i + 0x81) % 256:08b}" for i in range(SOURCE_BYTES))
    rows = []
    aligned_count = 0
    for length in LENGTHS:
        bits = source[SOURCE_OFFSET:SOURCE_OFFSET + length]
        expected = digest(bits)
        if length % 8 == 0:
            check_hashlib(bits, expected)
            aligned_count += 1
        rows.append(f"{length} {expected}")

    if args.emit:
        print("\n".join(rows))
        return

    path = Path(__file__).with_name("context_bit_fixtures.txt")
    observed = path.read_text(encoding="ascii").splitlines()
    if len(observed) != len(rows):
        raise SystemExit(f"{path.name}: expected {len(rows)} rows, found {len(observed)}")
    for index, (expected, actual) in enumerate(zip(rows, observed), 1):
        if expected != actual:
            raise SystemExit(f"{path.name}:{index}: expected {expected}, found {actual}")

    # Also sample both sides of the padding and block boundaries.
    extra_sizes = (1, 111, 112, 113, 127, 128, 129, 257)
    for size in extra_sizes:
        bits = source[:8 * size]
        check_hashlib(bits, digest(bits))

    print(f"Source: {SOURCE_BYTES} bytes, byte i = (37*i + 0x81) mod 256, "
          f"MSb-first, start bit {SOURCE_OFFSET}")
    print("Verified fixture lengths (bits):", ", ".join(map(str, LENGTHS)))
    print(f"Matched {len(rows)} committed digests and "
          f"{aligned_count + len(extra_sizes)} hashlib comparisons.")


if __name__ == "__main__":
    main()
