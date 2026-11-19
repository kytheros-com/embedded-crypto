"""Replay both SHA-512 SHAVS §6.4 collections with Python hashlib.

The reference retains a full digest history for each COUNT. Each README
defines displayed i = 3..7 and the unindented final MD. Run verify.py
separately to check extracted bytes against the archives and manifest.
"""

import hashlib
from pathlib import Path
import re


ROOT = Path(__file__).resolve().parent


def require(condition, diagnostic):
    if not condition:
        raise SystemExit(diagnostic)


def parse(path, detailed):
    lines = iter(line for line in path.read_text(encoding="ascii").splitlines()
                 if line.strip())
    line = next(lines, None)
    while line is not None and line.startswith("#"):
        line = next(lines, None)
    require(line == "[L = 64]", f"{path}: missing digest section")

    def field(prefix, size):
        row = next(lines, "")
        match = re.fullmatch(re.escape(prefix) + r"([0-9a-fA-F]+)", row)
        require(match is not None and len(match[1]) == size * 2,
                f"{path}: expected {size}-byte field {prefix!r}")
        return bytes.fromhex(match[1])

    seed = field("Seed = ", 64)
    checkpoints = []
    for count in range(100):
        require(next(lines, None) == f"COUNT = {count}", f"{path}: COUNT {count}")
        inner = {}
        if detailed:
            for index in range(3, 8):
                require(next(lines, None) == f"\ti = {index}",
                        f"{path}: COUNT {count}, i = {index}")
                inner[index] = (field("\tM = ", 192), field("\tMDi = ", 64))
        checkpoints.append((field("MD = ", 64), inner))
    require(next(lines, None) is None, f"{path}: extra data")
    return seed, checkpoints


def verify(collection):
    response = ROOT / collection / "SHA512Monte.rsp"
    details = ROOT / collection / "SHA512Monte.txt"
    seed, checkpoints = parse(response, False)
    detail_seed, detail_values = parse(details, True)
    require(seed == detail_seed, f"{collection}: seeds differ")
    hashes = pairs = outer_comparisons = final_agreements = 0
    for count, (expected, _) in enumerate(checkpoints):
        history = [seed, seed, seed]
        final, inner = detail_values[count]
        require(final == expected, f"{collection}: COUNT {count}: final MD differs")
        final_agreements += 1
        for index in range(3, 1003):
            message = b"".join(history[index - 3:index])
            digest = hashlib.sha512(message).digest()
            hashes += 1
            if index in inner:
                require((message, digest) == inner[index],
                        f"{collection}: COUNT {count}, i={index}: M or MDi mismatch")
                pairs += 1
            history.append(digest)
        seed = history[1002]
        require(seed == expected, f"{collection}: COUNT {count}: "
                f"expected {expected.hex()}, observed {seed.hex()}")
        outer_comparisons += 1
    require((hashes, pairs, outer_comparisons, final_agreements)
            == (100_000, 500, 100, 100), f"{collection}: incomplete coverage")
    print(f"{collection}: {hashes} hashlib.sha512 hashes, "
          f"{outer_comparisons} response checkpoints, {pairs} M/MDi pairs, "
          f"{final_agreements} detailed/response final MD agreements")


if __name__ == "__main__":
    for name in ("shabittestvectors", "shabytetestvectors"):
        verify(name)
