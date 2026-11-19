"""Verify the SHA-512 test LFSR with standard-library GF(2) arithmetic.

Bit i encodes the coefficient of x^i. Multiplication by x modulo p
shifts the coefficient word left and XORs the low coefficients of p
when bit 63 was set. Reflecting all 64 bits conjugates this operation
to the right-shifting Galois recurrence. The low coefficients 0x1b
reflect to 0xd800000000000000. The modulus remains p throughout.

Once p is shown irreducible, multiplication by x is a permutation
of the nonzero field elements. If x has order 2^64 - 1, then for
every nonzero element a, x^k * a = a exactly when x^k = 1.

Run with python3 and assertions enabled.
"""

from math import isqrt, prod

WIDTH = 64
POLYNOMIAL = (1 << WIDTH) | 0x1B
ORDER = (1 << WIDTH) - 1
FACTORS = (3, 5, 17, 257, 641, 65537, 6700417)
MASK = 0xD800000000000000
SEED = 0x8000000000000000
SAMPLES = 50_000


def reduce_polynomial(value, modulus):
    while value.bit_length() >= modulus.bit_length():
        value ^= modulus << (value.bit_length() - modulus.bit_length())
    return value


def multiply(left, right):
    product = 0
    while right:
        if right & 1:
            product ^= left
        left <<= 1
        right >>= 1
    return product


def power_mod(base, exponent):
    result = 1
    while exponent:
        if exponent & 1:
            result = reduce_polynomial(multiply(result, base), POLYNOMIAL)
        base = reduce_polynomial(multiply(base, base), POLYNOMIAL)
        exponent >>= 1
    return result


def polynomial_gcd(left, right):
    while right:
        left, right = right, reduce_polynomial(left, right)
    return left


def verify_polynomial():
    x = 2
    # Rabin's irreducibility criterion. The sole prime divisor of 64 is 2.
    assert power_mod(x, 1 << WIDTH) == x
    assert polynomial_gcd(POLYNOMIAL, power_mod(x, 1 << (WIDTH // 2)) ^ x) == 1

    assert len(set(FACTORS)) == len(FACTORS)
    assert prod(FACTORS) == ORDER
    for q in FACTORS:
        assert q >= 2 and all(q % d for d in range(2, isqrt(q) + 1)), q
    # Excluding every prime-factor quotient proves that x has order ORDER.
    assert power_mod(x, ORDER) == 1
    for q in FACTORS:
        assert power_mod(x, ORDER // q) != 1, q


def reflect(word):
    return int(f"{word:064b}"[::-1], 2)


def galois_next(state):
    return (state >> 1) ^ (MASK if state & 1 else 0)


def verify_states():
    assert reflect(POLYNOMIAL ^ (1 << WIDTH)) == MASK
    # Compare each transition with the independent polynomial recurrence.
    expected_basis = [1 << i for i in reversed(range(WIDTH))]
    checkpoints = {
        0: 0x8000000000000000,
        1: 0x4000000000000000,
        62: 0x0000000000000002,
        63: 0x0000000000000001,
        64: 0xD800000000000000,
        65: 0x6C00000000000000,
    }
    seen = set()
    state = SEED
    coefficient_word = 1
    for i in range(SAMPLES):
        assert state == reflect(coefficient_word), i
        if i < WIDTH:
            assert state == expected_basis[i], i
        if i in checkpoints:
            assert state == checkpoints[i], i
        assert state != 0 and state not in seen, i
        seen.add(state)
        state = galois_next(state)
        coefficient_word = reduce_polynomial(coefficient_word << 1, POLYNOMIAL)
    assert len(seen) == SAMPLES


if __name__ == "__main__":
    verify_polynomial()
    verify_states()
    print(f"Verified irreducibility, prime factorization, and order {ORDER}.")
    print(f"Verified 64 basis states, six checkpoints, and {SAMPLES:,} "
          "distinct nonzero states.")
