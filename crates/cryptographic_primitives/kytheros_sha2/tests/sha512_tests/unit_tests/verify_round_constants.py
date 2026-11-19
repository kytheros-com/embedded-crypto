#!/usr/bin/env python3
"""Reproduce the SHA-512 round constants for engineering review.

NIST FIPS 180-4 (August 2015), §4.2.3, defines K_0 through K_79 using the
first 64 fractional bits of the cube roots of the first 80 primes.
https://doi.org/10.6028/NIST.FIPS.180-4

For each prime p, scaling its cube root by 2**64 is equivalent to taking
the cube root of p * 2**192. Take the floor using exact integer comparisons
and reduce modulo 2**64 to discard the scaled integer part. The first
fractional bit maps to bit 63 and the 64th maps to bit 0. Values are
compared as integers independent of byte order.

This standalone artifact checks the source table against the mathematical
definition. Comparing against the published FIPS table is a separate
check. Compression, byte serialization, and algorithm validation are
outside the scope of this artifact.
"""

import re
from itertools import zip_longest
from pathlib import Path


ROUND_COUNT = 80
WORD_SIZE_IN_BITS = 64
DECLARATION = re.compile(
    r"^pub\(super\) const SHA_512_ROUND_CONSTANTS: \[u64; 80\] = \["
    r"([^\[\]]*)^\];$",
    re.MULTILINE,
)


def first_primes(count):
    """Test candidates in ascending order using trial division through sqrt."""
    primes = []
    candidate = 2
    while len(primes) < count:
        if all(candidate % p for p in primes if p * p <= candidate):
            primes.append(candidate)
        candidate += 1
    return primes


def integer_cube_root(value):
    """Bisect bounds satisfying lower**3 <= value < upper**3."""
    lower = 0
    upper = 1 << ((value.bit_length() + 2) // 3)
    while upper - lower > 1:
        midpoint = (lower + upper) // 2
        if midpoint**3 <= value:
            lower = midpoint
        else:
            upper = midpoint
    return lower


def round_constant(index, prime):
    scaled = prime << (3 * WORD_SIZE_IN_BITS)
    root = integer_cube_root(scaled)
    if not root**3 <= scaled < (root + 1)**3:
        raise SystemExit(f"K_{index}: integer cube-root bounds failed for {prime}")
    return root % (1 << WORD_SIZE_IN_BITS)


def main():
    crate = Path(__file__).resolve().parents[3]
    path = crate / "src" / "sha512" / "constants.rs"
    try:
        declarations = DECLARATION.findall(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError) as error:
        raise SystemExit(f"Cannot read SHA-512 table: {error}") from error
    if len(declarations) != 1:
        raise SystemExit(f"{path}: expected one SHA_512_ROUND_CONSTANTS declaration")

    entries = [entry.strip() for entry in declarations[0].split(",")]
    if entries[-1] == "":
        entries.pop()
    expected = [
        round_constant(index, prime)
        for index, prime in enumerate(first_primes(ROUND_COUNT))
    ]
    for index, (wanted, observed) in enumerate(zip_longest(expected, entries)):
        if observed is not None and re.fullmatch(r"0x[0-9a-f]{16}", observed):
            if wanted == int(observed, 16):
                continue
        wanted_text = "<end of table>" if wanted is None else f"{wanted:#018x}"
        observed_text = "<missing>" if observed is None else repr(observed)
        raise SystemExit(
            f"K_{index}: expected {wanted_text}, observed {observed_text}"
        )
    print("All 80 SHA-512 round constants match the exact integer derivation.")


if __name__ == "__main__":
    main()
