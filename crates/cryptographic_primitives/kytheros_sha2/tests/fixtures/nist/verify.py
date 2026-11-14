"""Verify retained NIST bytes with hashlib, independently of Rust SHA-256.

Run from any directory with Python 3. No network or third-party packages.
This checks integrity against the manifest, not independent authenticity.
"""

import hashlib
import json
from pathlib import Path
import zipfile

root = Path(__file__).resolve().parent
crate = root.parents[2]
manifest = json.loads((root / "provenance.json").read_text())
for entry in manifest["archives"]:
    archive = crate / "src/sha256/NIST_KAT_Files" / entry["filename"]
    assert hashlib.sha256(archive.read_bytes()).hexdigest() == entry["sha256"]
    with zipfile.ZipFile(archive) as source:
        for fixture in entry["files"]:
            data = (root / fixture["path"]).read_bytes()
            assert data == source.read(fixture["path"]), fixture["path"]
            assert hashlib.sha256(data).hexdigest() == fixture["sha256"]
            if fixture["cases"] is not None:
                key = b"COUNT =" if fixture.get("record_type") == "monte" else b"Len ="
                assert data.count(key) == fixture["cases"]
            print(f"verified {fixture['path']}")
