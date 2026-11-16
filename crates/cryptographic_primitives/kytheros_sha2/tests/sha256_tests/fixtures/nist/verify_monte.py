"""Offline SHAVS §6.4 reference execution with Python hashlib, not Rust.

Official expectations remain untouched. Run verify.py separately for archive
and extraction integrity. README defines displayed i = 3..7, with no offset.
"""

import hashlib
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parent


def parse(path, detailed):
    lines = iter(line for line in path.read_text().splitlines() if line.strip())
    line = next(lines)
    while line.startswith('#'):
        line = next(lines)
    assert line == '[L = 32]', path

    def field(prefix, size):
        match = re.fullmatch(re.escape(prefix) + r'([0-9a-fA-F]+)', next(lines))
        assert match and len(match[1]) == size * 2, path
        return bytes.fromhex(match[1])

    seed = field('Seed = ', 32)
    checkpoints = []
    for count in range(100):
        assert next(lines) == f'COUNT = {count}', path
        inner = {}
        if detailed:
            for index in range(3, 8):
                assert next(lines) == f'\ti = {index}', path
                inner[index] = (field('\tM = ', 96), field('\tMDi = ', 32))
        checkpoints.append((field('MD = ', 32), inner))
    assert next(lines, None) is None, path
    return seed, checkpoints


def verify(collection):
    response = ROOT / collection / 'SHA256Monte.rsp'
    details = ROOT / collection / 'SHA256Monte.txt'
    seed, checkpoints = parse(response, False)
    detail_seed, intermediate = parse(details, True)
    assert seed == detail_seed
    hashes = comparisons = 0
    for count, (expected, _) in enumerate(checkpoints):
        # A list of all digests provides a different state arrangement from
        # Rust's rolling triple. Only 1003 digests are retained per count.
        history = [seed, seed, seed]
        final, inner = intermediate[count]
        assert final == expected
        for index in range(3, 1003):
            message = b''.join(history[index - 3:index])
            digest = hashlib.sha256(message).digest()
            hashes += 1
            if index in inner:
                assert (message, digest) == inner[index], (details, count, index)
                comparisons += 1
            history.append(digest)
        seed = history[1002]
        assert seed == expected, (
            str(response), count, expected.hex(), seed.hex()
        )
    assert (hashes, comparisons) == (100000, 500)
    print(f'{collection}: PASS, {hashes} hashes, 100 checkpoints, '
          f'{comparisons} intermediate M/MDi pairs (hashlib reference)')


if __name__ == '__main__':
    for name in ('shabittestvectors', 'shabytetestvectors'):
        verify(name)
