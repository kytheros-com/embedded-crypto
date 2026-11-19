"""Verify both retained SHA-512 archives and their exact extracted bytes.

Only Python standard-library facilities are used. This is a manual local
integrity check. Publication authenticity requires separate evidence.
"""

import hashlib
import json
from pathlib import Path
from zipfile import ZipFile


ROOT = Path(__file__).resolve().parent
CRATE = ROOT.parents[3]
COUNTS = {
    "shabittestvectors": (1025, 1024),
    "shabytetestvectors": (129, 128),
}
NIST = (
    "https://csrc.nist.gov/CSRC/media/Projects/"
    "Cryptographic-Algorithm-Validation-Program/documents/shs/"
)


def require(condition, diagnostic):
    if not condition:
        raise SystemExit(diagnostic)


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def verify():
    manifest = json.loads((ROOT / "provenance.json").read_text(encoding="utf-8"))
    require(manifest["algorithm"] == "SHA-512", "incorrect manifest algorithm")
    for field in ("origin", "integrity_limit", "readme_interpretation"):
        require(bool(manifest[field]), f"missing provenance field: {field}")
    archives = manifest["archives"]
    require(
        sorted(entry["filename"] for entry in archives)
        == sorted(name + ".zip" for name in COUNTS),
        "manifest archive list is incomplete or contains extra entries",
    )
    for entry in archives:
        filename = entry["filename"]
        collection = Path(filename).stem
        path = f"tests/NIST_KAT_Files/{filename}"
        require(entry["repository_path"] == path, f"wrong source path: {filename}")
        require(entry["url"] == NIST + filename, f"wrong source URL: {filename}")
        archive = CRATE / path
        require(sha256(archive.read_bytes()) == entry["sha256"], f"checksum: {archive}")
        print(f"{filename}: SHA-256 {entry['sha256']}")

        short, long = COUNTS[collection]
        expected = {
            f"{collection}/{name}": (cases, pairs)
            for name, cases, pairs in (
                ("Readme.txt", None, 0),
                ("SHA512ShortMsg.rsp", short, 0),
                ("SHA512LongMsg.rsp", long, 0),
                ("SHA512Monte.rsp", 100, 0),
                ("SHA512Monte.txt", 100, 500),
            )
        }
        require(
            sorted(item["path"] for item in entry["files"]) == sorted(expected),
            f"incomplete or extra manifest entries: {collection}",
        )
        require(
            sorted(file.name for file in (ROOT / collection).iterdir())
            == sorted(Path(member).name for member in expected),
            f"missing or unexpected extracted files: {collection}",
        )

        with ZipFile(archive) as source:
            for item in entry["files"]:
                member = item["path"]
                require(source.namelist().count(member) == 1, f"archive member: {member}")
                data = (ROOT / member).read_bytes()
                require(data == source.read(member), f"archive byte mismatch: {member}")
                require(sha256(data) == item["sha256"], f"checksum: {member}")
                require(
                    (data.count(b"\r\n"), data.count(b"\n") - data.count(b"\r\n"))
                    == (item["crlf"], item["isolated_lf"]),
                    f"line endings: {member}",
                )
                lines = data.splitlines()
                lengths = sum(line.startswith(b"Len = ") for line in lines)
                counts = sum(line.startswith(b"COUNT = ") for line in lines)
                messages = sum(line.startswith(b"\tM = ") for line in lines)
                digests = sum(line.startswith(b"\tMDi = ") for line in lines)
                cases = lengths + counts or None
                require(
                    (cases, messages) == expected[member]
                    == (item["cases"], item["intermediate_pairs"])
                    and messages == digests,
                    f"case or intermediate count: {member}",
                )
                print(f"{member}: exact bytes, SHA-256 {item['sha256']}, "
                      f"cases={cases}, intermediate pairs={messages}")
    print("Verified 2 original archives and 10 extracted members.")


if __name__ == "__main__":
    verify()
