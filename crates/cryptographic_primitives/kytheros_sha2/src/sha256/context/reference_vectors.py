"""Local SHA-256 fixtures; run with Python 3, no third-party dependencies.

Reference: FIPS 180-4 (August 2015), sections 4.2.2, 5.1.1, 5.3.3, 6.2.2.
This standalone calculation does not read Rust sources or call production
code. Constants are derived with exact integer roots, padding uses textual
bits, and each block uses a 64-word schedule. Byte-aligned cases are also
checked with Python hashlib. This is not NIST-published test data, formal
verification, or independent human review. Output is embedded in tests.rs.
"""

from math import isqrt
import hashlib

MASK = 2**32 - 1


def primes():
    result = []
    candidate = 2
    while len(result) < 64:
        if all(candidate % p for p in result if p * p <= candidate):
            result.append(candidate)
        candidate += 1
    return result


def cube_root(n):
    low, high = 0, 2 ** ((n.bit_length() + 2) // 3)
    while low + 1 < high:
        middle = (low + high) // 2
        if middle**3 <= n:
            low = middle
        else:
            high = middle
    return low


PRIMES = primes()
INITIAL = [isqrt(p * 2**64) & MASK for p in PRIMES[:8]]
CONSTANTS = [cube_root(p * 2**96) & MASK for p in PRIMES]


def rotate(x, n):
    return x // 2**n + (x % 2**n) * 2 ** (32 - n)


def digest(bits):
    length = len(bits)
    bits += "1" + "0" * ((448 - length - 1) % 512)
    bits += f"{length:064b}"
    state = INITIAL[:]
    for offset in range(0, len(bits), 512):
        words = [int(bits[i:i + 32], 2)
                 for i in range(offset, offset + 512, 32)]
        for t in range(16, 64):
            x, y = words[t - 15], words[t - 2]
            s0 = rotate(x, 7) ^ rotate(x, 18) ^ (x // 8)
            s1 = rotate(y, 17) ^ rotate(y, 19) ^ (y // 1024)
            words.append((words[t - 16] + s0 + words[t - 7] + s1) & MASK)
        a, b, c, d, e, f, g, h = state
        for k, w in zip(CONSTANTS, words):
            s0 = rotate(a, 2) ^ rotate(a, 13) ^ rotate(a, 22)
            s1 = rotate(e, 6) ^ rotate(e, 11) ^ rotate(e, 25)
            choose = (e & f) ^ ((MASK ^ e) & g)
            majority = (a & b) ^ (a & c) ^ (b & c)
            t1 = (h + s1 + choose + k + w) & MASK
            t2 = (s0 + majority) & MASK
            a, b, c, d, e, f, g, h = (
                (t1 + t2) & MASK, a, b, c, (d + t1) & MASK, e, f, g)
        state = [(x + y) & MASK
                 for x, y in zip(state, [a, b, c, d, e, f, g, h])]
    return "".join(f"{word:08x}" for word in state)


if __name__ == "__main__":
    # Cross-check byte-oriented reference calculations with a separate library.
    for size in [0, 1, 55, 56, 64, 65, 128, 1000]:
        message = bytes((37 * i + 0x81) % 256 for i in range(size))
        bits = "".join(f"{byte:08b}" for byte in message)
        assert digest(bits) == hashlib.sha256(message).hexdigest()
    message = bytes((37 * i + 0x81) % 256 for i in range(130))
    bits = "".join(f"{byte:08b}" for byte in message)
    for length in [0, 1, 7, 8, 446, 447, 448, 449, 510, 511, 512, 513,
                   1023, 1024]:
        print(f'    ({length}, "{digest(bits[3:3 + length])}"),')
