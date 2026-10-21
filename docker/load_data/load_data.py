#!/usr/bin/env python3
"""LoadData node algorithm (minimal runnable version).

Conventions (matching LocalDockerExecutor):
  - The input directory is mounted at /data/input (read-only) and contains
    the raw mass spectrometry data. For local experiments, a .txt file is
    used to simulate the raw file, with each line simulating one spectrum.
  - The output directory is mounted at /data/output (writable); the
    algorithm must write `result.pb` there: a serialized LoadDataResult
    Protobuf binary file.
  - Filter parameters are passed via environment variables:
      MIN_INTENSITY_THRESHOLD: a float; lines may carry "intensity=<val>",
        and lines below this threshold are filtered out and not counted
        towards spectrum_count.
      ALLOWED_FILE_EXTENSIONS: comma-separated list of extensions, e.g.
        ".txt,.mzml".
"""

import os
import sys
from pathlib import Path

import workflow_pb2 as pb

INPUT_DIR = Path("/data/input")
OUTPUT_DIR = Path("/data/output")


def parse_allowed_extensions() -> list[str]:
    raw = os.environ.get("ALLOWED_FILE_EXTENSIONS", "")
    exts = [e.strip().lower() for e in raw.split(",") if e.strip()]
    return exts or [".txt"]


def parse_min_intensity() -> float:
    raw = os.environ.get("MIN_INTENSITY_THRESHOLD", "0")
    try:
        return float(raw)
    except ValueError:
        return 0.0


def line_intensity(line: str) -> float:
    """Parse `intensity=<value>` from a line; if parsing fails, treat it as large enough to not be filtered."""
    for token in line.split():
        if token.startswith("intensity="):
            try:
                return float(token.split("=", 1)[1])
            except ValueError:
                return float("inf")
    return float("inf")


def find_input_files(allowed_extensions: list[str]) -> list[Path]:
    return sorted(
        p
        for p in INPUT_DIR.iterdir()
        if p.is_file() and p.suffix.lower() in allowed_extensions
    )


def main() -> int:
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    allowed_extensions = parse_allowed_extensions()
    min_intensity = parse_min_intensity()

    result = pb.LoadDataResult()

    try:
        input_files = find_input_files(allowed_extensions)
        if not input_files:
            raise FileNotFoundError(
                f"No files with extensions in {allowed_extensions} found in {INPUT_DIR}"
            )

        spectrum_count = 0
        for file_path in input_files:
            with file_path.open("r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if not line:
                        continue
                    if line_intensity(line) >= min_intensity:
                        spectrum_count += 1

        result.status = pb.NodeStatus.SUCCESS
        result.message = (
            f"Parsed {len(input_files)} file(s), "
            f"min_intensity_threshold={min_intensity}"
        )
        result.spectrum_count = spectrum_count
        # The current minimal implementation does not distinguish features
        # from spectra, so feature_count mirrors spectrum_count.
        result.feature_count = spectrum_count

    except Exception as exc:  # noqa: BLE001 - top-level catch, converted into a structured failure result
        result.status = pb.NodeStatus.FAILED
        result.message = f"LoadData failed: {exc}"
        result.spectrum_count = 0
        result.feature_count = 0

    output_path = OUTPUT_DIR / "result.pb"
    result.output_data.storage_type = pb.StorageType.LOCAL_FILE
    result.output_data.uri = str(output_path)

    with output_path.open("wb") as f:
        f.write(result.SerializeToString())

    print(f"[load_data] status={pb.NodeStatus.Name(result.status)} "
          f"spectrum_count={result.spectrum_count} -> {output_path}")

    return 0 if result.status == pb.NodeStatus.SUCCESS else 1


if __name__ == "__main__":
    sys.exit(main())
