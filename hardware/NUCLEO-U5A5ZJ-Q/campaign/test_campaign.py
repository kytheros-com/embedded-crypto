"""Host-only checks for campaign acceptance and evidence integrity."""

import copy
import json
import tempfile
import unittest
from pathlib import Path

import build
import runner


SYMBOLS = {
    "flt_results": 0x20002000,
    "mpu_work": 0x20001000,
    "protected_sentinel": 0x20002100,
    "expected_fault": 0x20002104,
    "udf_one": 0x080002F2,
    "cont_one": 0x080002F4,
    "udf_two": 0x08000332,
    "cont_two": 0x08000334,
    "svc_finish_return": 0x0800033E,
}


def repeated_capture():
    record = [0] * 64
    record[:7] = [1, 5, 2, 2, 3, 5, 0]
    record[9:16] = [0x50000, 0x201, runner.SENTINEL,
                     runner.SENTINEL, 5, 0, 0]
    for start, fault, continuation, test_id in (
        (16, SYMBOLS["udf_one"], SYMBOLS["cont_one"], 1),
        (36, SYMBOLS["udf_two"], SYMBOLS["cont_two"], 2),
    ):
        record[start:start + 20] = [
            3, 0, 0x20002000, 0, 0, 0x08001FE1,
            fault, 0x01000000, 0x10000, 0, 0x50008,
            0xFFFFFFBC, 0x20001FE0, 0x20003000,
            1, 6, fault, continuation, test_id, 1,
        ]
    return {
        "configuration": "008",
        "result": "PROVISIONAL PASS",
        "led": "three",
        "record": record,
        "user": [1, 2, 3, 4, 5, 0],
        "frame": [0, 0, 0, 0, 0, 0,
                  SYMBOLS["svc_finish_return"], 0x01000000],
        "descriptor": [5, 0, 2, 2, 6, 0x10000, 3,
                       SYMBOLS["udf_two"], SYMBOLS["cont_two"],
                       0x20001FE0, 0xFFFFFFBC, 5],
        "sentinel": runner.SENTINEL,
        "ccr": 0x201,
        "scb": [0x50080, 0, 0],
        "cpuid": runner.CPUID,
    }


def accept(capture):
    runner.accept_repeated(
        SYMBOLS, capture["record"], capture["user"],
        capture["frame"], capture["descriptor"],
        capture["sentinel"], capture["ccr"],
        capture["scb"], capture["cpuid"],
    )


class CampaignChecks(unittest.TestCase):
    def test_valid_repeated_and_initial_cfsr_capture(self):
        capture = repeated_capture()
        capture["record"][7] = 0x10000
        accept(capture)

    def test_divide_trap_bit_and_ccr_equality(self):
        capture = repeated_capture()
        capture["ccr"] = capture["record"][10] = 0x211
        with self.assertRaisesRegex(RuntimeError, "Startup state"):
            accept(capture)
        capture["ccr"] = 0x201
        with self.assertRaisesRegex(RuntimeError, "Startup state"):
            accept(capture)

    def test_reserved_word(self):
        capture = repeated_capture()
        capture["record"][63] = 1
        with self.assertRaisesRegex(RuntimeError, "Reserved"):
            accept(capture)

    def test_cross_host_source_mismatch(self):
        item = {"sources_sha256": {"main.rs": "a"},
                "payload": {"bytes": 16, "sha256": "b"},
                "image": {"bytes": 16, "sha256": "c"}}
        prior = copy.deepcopy(item)
        prior["sources_sha256"]["main.rs"] = "different"
        with self.assertRaisesRegex(RuntimeError, "sources_sha256"):
            build.compare_reference(item, prior, "008")

    def test_complete_repeat_comparison(self):
        first = repeated_capture()
        second = copy.deepcopy(first)
        second["record"][19] ^= 1  # Incidental stacked R3.
        second["record"][23] ^= 0x20000000  # Incidental flags.
        second["user"][0] ^= 1
        second["frame"][3] ^= 1  # Final SVC frame, not UDF.
        second["descriptor"][0] ^= 1
        second["sentinel"] ^= 1
        second["ccr"] ^= 1
        second["scb"][0] ^= 1
        second["cpuid"] ^= 1
        result = runner.compare_runs("008", SYMBOLS, first, second)
        self.assertEqual(
            [region["words"] for region in result.values()],
            [64, 6, 8, 12, 1, 1, 3, 1],
        )
        self.assertTrue(all(region["differences"]
                            for region in result.values()))
        self.assertEqual(
            result["supervisor"]["differences"][0]["classification"],
            "unconstrained",
        )
        self.assertEqual(
            result["supervisor"]["differences"][1]["classification"],
            "unconstrained flags",
        )
        self.assertEqual(
            result["final_svc_frame"]["differences"][0]["field"],
            "R3",
        )
        self.assertEqual(
            result["descriptor"]["differences"][0]["address"],
            "0x20002104",
        )

    def test_alignment_direct_frame_comparison(self):
        symbols = dict(SYMBOLS)
        symbols["protected_sentinel"] = 0x200020E8
        first = repeated_capture()
        first["record"] = [0] * 58
        first["user"] = [0] * 5
        first["descriptor"] = None
        second = copy.deepcopy(first)
        second["frame"][3] = 0x1234
        result = runner.compare_runs("005-enabled", symbols,
                                     first, second)
        self.assertEqual(result["physical_psp_frame"]["words"], 8)
        self.assertEqual(
            result["physical_psp_frame"]["differences"][0]["classification"],
            "unconstrained",
        )

    def test_colored_dump_and_fail_closed_parser(self):
        colored = ("\x1b[36m0x20002000\x1b[0m : "
                   "\x1b[32m00000001 00000002\x1b[0m\n")
        self.assertEqual(runner.parse_words(colored, 0x20002000, 2),
                         [1, 2])
        with self.assertRaisesRegex(RuntimeError, "Incomplete"):
            runner.parse_words(colored, 0x20002000, 3)
        with self.assertRaisesRegex(RuntimeError, "Duplicate"):
            runner.parse_words(colored + colored, 0x20002000, 2)
        with self.assertRaisesRegex(RuntimeError, "Incomplete"):
            runner.parse_words(colored, 0x20002004, 1)
        with self.assertRaisesRegex(RuntimeError, "Malformed"):
            runner.parse_words(colored + "20002008 : GARBAGE\n",
                               0x20002000, 2)
        with self.assertRaisesRegex(RuntimeError, "Malformed"):
            runner.parse_words(colored + "2002008 : 00000003\n",
                               0x20002000, 2)

    def test_manifest_identity_changes_on_resume(self):
        with tempfile.TemporaryDirectory() as tmp:
            base = Path(tmp)
            first = base / "manifest.json"
            first.write_text("first\n")
            second = base / "reference.json"
            second.write_text("reference\n")
            manifest = {"configurations": {
                name: {"image": {"bytes": 16, "sha256": name}}
                for name in build.CONFIGS
            }}
            identity = runner.evidence_identity(first, second, manifest)
            first.write_text("changed\n")
            self.assertNotEqual(
                identity,
                runner.evidence_identity(first, second, manifest),
            )
            first.write_text("first\n")
            second.write_text("changed reference\n")
            self.assertNotEqual(
                identity,
                runner.evidence_identity(first, second, manifest),
            )
            second.write_text("reference\n")
            manifest["configurations"]["008"]["image"]["sha256"] = "new"
            self.assertNotEqual(
                identity,
                runner.evidence_identity(first, second, manifest),
            )

    def test_saved_execution_requires_raw_transcripts(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder = Path(tmp)
            capture = repeated_capture()
            (folder / "decoded.json").write_text(json.dumps(capture))
            with self.assertRaisesRegex(RuntimeError, "raw evidence"):
                runner.validate_saved_run(folder, "008", SYMBOLS)
            names = {"supervisor": "supervisor-record",
                     "user": "user-record",
                     "final_svc_frame": "physical-frame",
                     "descriptor": "descriptor", "sentinel": "sentinel",
                     "ccr": "ccr", "scb": "scb", "cpuid": "cpuid"}
            for label, address, words in runner.observation_regions(
                    "008", SYMBOLS, capture):
                lines = [
                    f"{address + 4 * index:08X} : {word:08X}"
                    for index, word in enumerate(words)
                ]
                (folder / f"{names[label]}.txt").write_text(
                    "exit=0\n" + "\n".join(lines) + "\n")
            self.assertEqual(
                runner.validate_saved_run(folder, "008", SYMBOLS),
                capture,
            )
            (folder / "ccr.txt").write_text(
                "exit=0\nE000ED14 : 00000211\n")
            with self.assertRaisesRegex(RuntimeError,
                                        "raw/decoded evidence differs"):
                runner.validate_saved_run(folder, "008", SYMBOLS)


if __name__ == "__main__":
    unittest.main()
