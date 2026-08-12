#!/usr/bin/env python3
"""Compare a rendered PDF against a native Office export on three axes.

No single measure is trustworthy alone, and each one's blind spot is
another's strength:

- **Geometry** catches position, size, and pitch. It is what actually
  changes when a layout bug is fixed, and it is the only axis that can
  distinguish "moved to the right place" from "moved somewhere else".
  Blind to colour and to elements that are absent entirely.
- **Histogram** catches fill colour, recolouring, and missing elements,
  because it counts what is drawn without caring where. Blind to position,
  size, and font: those keep the ink total the same.
- **Pixel difference** is the catch-all that notices what the other two
  were not looking for. It is the weakest signal of the three: `AE` counts
  differing pixels without weighing how different they are, so it scores a
  layout shift and a colour inversion alike, and it can *rise* when a fix is
  correct but the element is still displaced by an unrelated defect.

Read them together. A fix that improves geometry while leaving the
histogram flat has moved something without changing what is drawn, which is
usually exactly what a positioning fix should do.

Usage:
    compare_render.py GT.pdf OUTPUT.pdf [--page N] [--dpi 150] [--audit]
"""

from __future__ import annotations

import argparse
import re
import shutil
import subprocess
import sys
import tempfile
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path

from spatial_match import minimum_cost_pairs

PAGE_RE = re.compile(r'<page width="[\d.]+" height="[\d.]+">(.*?)</page>', re.S)
WORD_RE = re.compile(
    r'<word xMin="([\d.-]+)" yMin="([\d.-]+)" xMax="([\d.-]+)" yMax="([\d.-]+)">(.*?)</word>',
    re.S,
)
FILL_TEXT_RE = re.compile(
    r'<fill_text[^>]*transform="([-0-9.]+) ([-0-9.]+) ([-0-9.]+) ([-0-9.]+) '
    r'([-0-9.]+) ([-0-9.]+)"[^>]*>(.*?)</fill_text>',
    re.S,
)
# mutool 1.23.x opens a page as `<page mediabox="...">`; later builds add a
# `number` attribute. Splitting on the numbered form alone yields zero pages and
# silently drops the geometry axis.
TRACE_PAGE_RE = re.compile(r"<page\b")
GLYPH_RE = re.compile(r'<g unicode="([^"]*)" glyph="[^"]*" x="([-0-9.]+)" y="([-0-9.]+)"')
HISTOGRAM_BINS = 32
# Every ImageMagick tool the colour and pixel axes reach for. Named here so the
# availability check and the call sites cannot drift apart.
IMAGEMAGICK_TOOLS = ("convert", "identify", "compare")


@dataclass(frozen=True)
class TextLine:
    page: int
    x_min: float
    y_min: float
    text: str


@dataclass(frozen=True)
class TextLineMatch:
    reference: TextLine
    candidate: TextLine
    occurrence: int
    occurrences: int

    @property
    def label(self) -> str:
        if self.occurrences == 1:
            return self.reference.text
        return f"{self.reference.text} [{self.occurrence}/{self.occurrences}]"

    @property
    def dx(self) -> float:
        return self.candidate.x_min - self.reference.x_min

    @property
    def dy(self) -> float:
        return self.candidate.y_min - self.reference.y_min


def render_page(pdf: Path, page: int, dpi: int, out_dir: Path, role: str) -> Path:
    """Rasterise one page, returning the PNG path.

    `role` names the output, because the GT and the candidate usually share
    a file stem: rendering both under that stem made the second overwrite
    the first, and every comparison then ran an image against itself and
    reported a perfect match.
    """
    prefix = out_dir / role
    subprocess.run(
        ["pdftoppm", "-r", str(dpi), "-png", "-f", str(page), "-l", str(page),
         str(pdf), str(prefix)],
        check=True,
        capture_output=True,
    )
    pages = sorted(out_dir.glob(f"{role}-*.png"))
    if not pages:
        raise SystemExit(f"{pdf}: page {page} did not render")
    return pages[0]


def has_mutool() -> bool:
    """Whether `mutool` is on PATH (it ships in `mupdf-tools`)."""
    return shutil.which("mutool") is not None


def imagemagick_command(tool: str) -> list[str] | None:
    """Argv prefix invoking an ImageMagick tool, or None if it is unavailable.

    ImageMagick 7 dispatches every tool through a single `magick` binary;
    ImageMagick 6 installs them under their own names and has no `magick` at
    all. Hardcoding the IM7 spelling left the colour and pixel axes dying with
    FileNotFoundError on an IM6 host, after the geometry axis had already
    printed a report that looked whole.
    """
    if shutil.which("magick") is not None:
        # `magick convert` is deprecated in 7.1 and warns; plain conversion is
        # the bare dispatcher, so only the named subtools take an argument.
        return ["magick"] if tool == "convert" else ["magick", tool]
    if shutil.which(tool) is not None:
        return [tool]
    return None


def has_imagemagick() -> bool:
    """Whether every tool the colour and pixel axes need can be invoked."""
    return all(imagemagick_command(tool) is not None for tool in IMAGEMAGICK_TOOLS)


def require_vision_artifact_dependencies(artifacts_dir: Path | None) -> None:
    """Fail rather than silently omit requested model-vision evidence."""
    if artifacts_dir is not None and not has_imagemagick():
        raise SystemExit(
            "--artifacts-dir requires ImageMagick to preserve full pages, the "
            "pixel diff, and matched crops; install imagemagick and rerun"
        )


def baseline_lines(pdf: Path) -> list[TextLine]:
    """Text-line anchors from `mutool draw -F trace` affine coordinates.

    A `<fill_text>` carries the complete text-space matrix, so each glyph maps
    to `x = a * gx + c * gy + tx` and `y = b * gx + d * gy + ty`. Office
    exports on macOS often use scaled or rotated text spaces while ours are
    commonly plain translations; the same affine formula covers both.

    For non-rotated text, baselines jitter by fractions of a point inside one
    visual line, so rows are bucketed to 1pt before being joined. A rotated or
    skewed `<fill_text>` is already one visual run and cannot share a horizontal
    baseline bucket; it stays intact and uses the minimum transformed glyph
    coordinates as its comparable spatial anchor.
    """
    trace = subprocess.run(
        ["mutool", "draw", "-F", "trace", "-o", "-", str(pdf)],
        capture_output=True,
        text=True,
    )
    if trace.returncode != 0:
        return []
    lines: list[TextLine] = []
    for page_index, page in enumerate(TRACE_PAGE_RE.split(trace.stdout)[1:]):
        rows: dict[int, list[tuple[float, float, str]]] = {}
        rotated_lines: list[TextLine] = []
        for match in FILL_TEXT_RE.finditer(page):
            scale_x = float(match.group(1))
            shear_y = float(match.group(2))
            shear_x = float(match.group(3))
            scale_y = float(match.group(4))
            translate_x, translate_y = float(match.group(5)), float(match.group(6))
            transformed_glyphs: list[tuple[float, float, str]] = []
            for glyph in GLYPH_RE.finditer(match.group(7)):
                char = glyph.group(1)
                if not char.strip():
                    continue
                glyph_x, glyph_y = float(glyph.group(2)), float(glyph.group(3))
                x = translate_x + scale_x * glyph_x + shear_x * glyph_y
                y = translate_y + shear_y * glyph_x + scale_y * glyph_y
                transformed_glyphs.append((x, y, char))
            if abs(shear_x) > 1e-9 or abs(shear_y) > 1e-9:
                text = re.sub(
                    r"\s+", " ", "".join(glyph[2] for glyph in transformed_glyphs)
                ).strip()
                if text:
                    rotated_lines.append(
                        TextLine(
                            page_index,
                            min(glyph[0] for glyph in transformed_glyphs),
                            min(glyph[1] for glyph in transformed_glyphs),
                            text,
                        )
                    )
            else:
                for x, baseline, char in transformed_glyphs:
                    rows.setdefault(round(baseline), []).append((x, baseline, char))
        for key in sorted(rows):
            glyphs = sorted(rows[key])
            text = re.sub(r"\s+", " ", "".join(glyph[2] for glyph in glyphs)).strip()
            if text:
                # The 1pt bucket only groups glyphs into a line; reporting its
                # key as the position would quantise every measurement to 1pt
                # and hide the sub-point row-pitch differences this axis exists
                # to find. Report the line's own baseline instead.
                lines.append(
                    TextLine(
                        page_index,
                        min(g[0] for g in glyphs),
                        min(g[1] for g in glyphs),
                        text,
                    )
                )
        lines.extend(rotated_lines)
    return lines


def descriptor_box_lines(pdf: Path) -> list[TextLine]:
    """Fallback line tops from `pdftotext -bbox`, used only without `mutool`.

    `yMin` is each glyph's *font-descriptor box*, not its ink or its baseline.
    The two PDFs always embed different subsets, so the drift this yields
    carries an error proportional to font size — on the newsletter mock it
    reported +2.90pt for a 22pt heading whose baseline is really 1.07pt the
    other way, which is how #501 came to be filed against a defect that did
    not exist (issue #505).
    """
    xml = subprocess.run(
        ["pdftotext", "-bbox", str(pdf), "-"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    lines: list[TextLine] = []
    for page_index, page in enumerate(PAGE_RE.findall(xml)):
        rows: dict[int, list[tuple[float, float, str]]] = {}
        for match in WORD_RE.finditer(page):
            x_min, y_min = float(match.group(1)), float(match.group(2))
            rows.setdefault(round(y_min), []).append((x_min, y_min, match.group(5)))
        for key in sorted(rows):
            words = sorted(rows[key])
            text = re.sub(r"\s+", " ", " ".join(word[2] for word in words)).strip()
            if text:
                lines.append(
                    TextLine(page_index, min(w[0] for w in words),
                             min(w[1] for w in words), text)
                )
    return lines


def text_lines(pdf: Path) -> list[TextLine]:
    """Lines for the geometry axis, by true baseline where `mutool` allows."""
    if has_mutool():
        lines = baseline_lines(pdf)
        if lines:
            return lines
    return descriptor_box_lines(pdf)


def match_text_line_instances(
    gt_lines: list[TextLine], other_lines: list[TextLine]
) -> list[TextLineMatch]:
    """Match equal text per page, including every repeated occurrence."""
    gt_groups: dict[tuple[int, str], list[TextLine]] = defaultdict(list)
    other_groups: dict[tuple[int, str], list[TextLine]] = defaultdict(list)
    for line in gt_lines:
        gt_groups[(line.page, line.text)].append(line)
    for line in other_lines:
        other_groups[(line.page, line.text)].append(line)

    matches: list[TextLineMatch] = []
    for key, references in gt_groups.items():
        candidates = other_groups.get(key, [])
        references = sorted(references, key=lambda line: (line.y_min, line.x_min))
        candidates = sorted(candidates, key=lambda line: (line.y_min, line.x_min))
        for reference_index, candidate_index in minimum_cost_pairs(
            [(line.x_min, line.y_min) for line in references],
            [(line.x_min, line.y_min) for line in candidates],
        ):
            matches.append(
                TextLineMatch(
                    reference=references[reference_index],
                    candidate=candidates[candidate_index],
                    occurrence=reference_index + 1,
                    occurrences=len(references),
                )
            )
    return sorted(
        matches,
        key=lambda match: (
            match.reference.page,
            match.reference.y_min,
            match.reference.x_min,
        ),
    )


def page_text_lines(pdf: Path, page: int) -> list[TextLine]:
    """Return only the requested 1-based page's text instances."""
    return [line for line in text_lines(pdf) if line.page == page - 1]


def report_geometry(
    gt: Path, other: Path, page: int = 1, large_shift: float = 5.0
) -> dict[str, float]:
    """Vertical and horizontal drift of spatially matched text instances."""
    gt_text_lines = page_text_lines(gt, page)
    other_text_lines = page_text_lines(other, page)
    matches = match_text_line_instances(gt_text_lines, other_text_lines)
    dy = [match.dy for match in matches]
    dx = [match.dx for match in matches]
    other_text_pages: dict[str, set[int]] = defaultdict(set)
    for line in other_text_lines:
        other_text_pages[line.text].add(line.page)
    matched_reference_ids = {id(match.reference) for match in matches}
    page_mismatch = sum(
        1
        for line in gt_text_lines
        if id(line) not in matched_reference_ids
        and line.text in other_text_pages
        and line.page not in other_text_pages[line.text]
    )

    print("## Geometry — position, size, pitch")
    if not has_mutool():
        print("  APPROXIMATE: mutool absent, so positions come from font-descriptor")
        print("  boxes rather than baselines. The error scales with font size and can")
        print("  invert the sign. Install mupdf-tools before trusting these numbers.")
    if not dy:
        print("  no text instances matched; compare pages manually")
        return {}
    mad_y = sum(abs(value) for value in dy) / len(dy)
    mad_x = sum(abs(value) for value in dx) / len(dx)
    coverage = len(dy) / len(gt_text_lines) if gt_text_lines else 0.0
    worst_dy = max(matches, key=lambda match: abs(match.dy))
    worst_dx = max(matches, key=lambda match: abs(match.dx))
    large_matches = [
        match for match in matches if abs(match.dx) > large_shift or abs(match.dy) > large_shift
    ]
    print(f"  matched instances  {len(dy)} of {len(gt_text_lines)} "
          f"({coverage * 100:.0f}% of the GT's text lines)")
    print(
        f"  vertical   MAD {mad_y:7.2f}pt   worst {worst_dy.dy:+8.2f}pt  "
        f"{worst_dy.label[:60]}"
    )
    print(
        f"  horizontal MAD {mad_x:7.2f}pt   worst {worst_dx.dx:+8.2f}pt  "
        f"{worst_dx.label[:60]}"
    )
    print(f"  large instance shifts (>{large_shift:.2f}pt): {len(large_matches)}")
    for match in sorted(
        large_matches, key=lambda item: max(abs(item.dx), abs(item.dy)), reverse=True
    ):
        print(
            f"    page {match.reference.page + 1}: {match.label[:52]}  "
            f"dx {match.dx:+.2f}pt  dy {match.dy:+.2f}pt"
        )
    if page_mismatch:
        print(f"  on a different page: {page_mismatch} line(s) — pagination differs")
    return {
        "mad_y": mad_y,
        "mad_x": mad_x,
        "page_mismatch": float(page_mismatch),
        "matched": float(len(dy)),
        "coverage": coverage,
        "worst_dx": worst_dx.dx,
        "worst_dy": worst_dy.dy,
        "large_shift_count": float(len(large_matches)),
        "large_shift_threshold": large_shift,
    }


def histogram(png: Path) -> tuple[list[int], int]:
    """Per-channel binned colour counts, flattened R|G|B, and the pixel total."""
    txt = subprocess.run(
        [*(imagemagick_command("convert") or []), str(png), "-depth", "8", "txt:-"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    counts = [0] * (3 * HISTOGRAM_BINS)
    total = 0
    step = 256 // HISTOGRAM_BINS
    for line in txt.splitlines():
        head, _, rest = line.partition("#")
        if not head or len(rest) < 6:
            continue
        try:
            channels = [int(rest[i : i + 2], 16) for i in (0, 2, 4)]
        except ValueError:
            continue
        for index, value in enumerate(channels):
            counts[index * HISTOGRAM_BINS + min(value // step, HISTOGRAM_BINS - 1)] += 1
        total += 1
    return counts, total


def ink_fraction(counts: list[int], total: int) -> float:
    """Share of pixels that are not near-white, averaged over the channels."""
    if total == 0:
        return 0.0
    dark = sum(
        counts[channel * HISTOGRAM_BINS + b]
        for channel in range(3)
        for b in range(HISTOGRAM_BINS - 2)
    )
    return dark / (3.0 * total)


def report_histogram(gt_png: Path, other_png: Path) -> dict[str, float]:
    """Colour-distribution agreement, independent of where the pixels sit."""
    gt_counts, gt_total = histogram(gt_png)
    counts, total = histogram(other_png)
    gt_sum = sum(gt_counts) or 1
    other_sum = sum(counts) or 1
    reference = [value / gt_sum for value in gt_counts]
    candidate = [value / other_sum for value in counts]
    intersection = sum(min(a, b) for a, b in zip(reference, candidate))
    # Bin-wise agreement punishes a one-level shift as hard as a recolour: a
    # smooth gradient dithers by a channel step or two between renderers, and
    # every one of those pixels lands in a neighbouring bin. Three decks
    # scored 0.9745-0.9860 on intersection with their gradients pixel-identical
    # to within +-2 per channel. Comparing the *cumulative* distributions
    # instead measures how far colour has to move, so a one-bin shift costs
    # almost nothing while a genuine recolour still shows.
    shift = cumulative_distance(reference, candidate)
    gt_ink = ink_fraction(gt_counts, gt_total)
    ink = ink_fraction(counts, total)

    print("## Histogram — fill colour, recolouring, missing elements")
    print(f"  intersection       {intersection:.4f}   (1.0000 = identical distribution)")
    print(f"  colour shift       {shift:.4f}   (0.0000 = identical; tolerates dithering)")
    print(f"  ink coverage       {ink * 100:6.3f}%  against GT {gt_ink * 100:6.3f}%"
          f"   ({(ink - gt_ink) * 100:+.3f}%)")
    return {
        "intersection": intersection,
        "shift": shift,
        "ink_delta": (ink - gt_ink) * 100.0,
    }


def cumulative_distance(reference: list[float], candidate: list[float]) -> float:
    """Mean per-channel distance between the cumulative distributions.

    Insensitive to a colour landing one bin either side of where it did in
    the reference, which is what renderer dithering produces, while still
    growing with a real change in what colour is present.
    """
    channels = 3
    total = 0.0
    for channel in range(channels):
        start = channel * HISTOGRAM_BINS
        run_reference = 0.0
        run_candidate = 0.0
        for index in range(start, start + HISTOGRAM_BINS):
            run_reference += reference[index]
            run_candidate += candidate[index]
            total += abs(run_reference - run_candidate)
    return total / (channels * HISTOGRAM_BINS)


def report_pixels(gt_png: Path, other_png: Path, out_dir: Path) -> None:
    """Whole-page difference, as a coarse catch-all."""
    normalised = out_dir / "gt-normalised.png"
    size = subprocess.run(
        [*(imagemagick_command("identify") or []), "-format", "%wx%h", str(other_png)],
        capture_output=True, text=True, check=True,
    ).stdout.strip()
    # pdftoppm can differ by a pixel between two PDFs of the same paper size,
    # which would otherwise make every comparison fail outright.
    subprocess.run(
        [*(imagemagick_command("convert") or []), str(gt_png),
         "-background", "white", "-extent", size, str(normalised)],
        check=True, capture_output=True,
    )

    print("## Pixel difference — coarse catch-all, read last")
    for label, args in (
        ("AE  5% fuzz", ["-metric", "AE", "-fuzz", "5%"]),
        ("AE  1% fuzz", ["-metric", "AE", "-fuzz", "1%"]),
        ("RMSE       ", ["-metric", "RMSE"]),
    ):
        result = subprocess.run(
            [*(imagemagick_command("compare") or []), *args,
             str(normalised), str(other_png), "null:"],
            capture_output=True, text=True,
        )
        print(f"  {label}      {result.stderr.strip()}")


def shift_crop_box(
    match: TextLineMatch, dpi: int, image_width: int, image_height: int
) -> tuple[int, int, int, int]:
    """Matched page-space crop containing both locations of one shifted line."""
    scale = dpi / 72.0
    text_extent_pt = max(72.0, min(240.0, len(match.reference.text) * 8.0))
    left = max(0, round((min(match.reference.x_min, match.candidate.x_min) - 24.0) * scale))
    right = min(
        image_width,
        round(
            (max(match.reference.x_min, match.candidate.x_min) + text_extent_pt + 24.0)
            * scale
        ),
    )
    top = max(0, round((min(match.reference.y_min, match.candidate.y_min) - 32.0) * scale))
    bottom = min(
        image_height,
        round((max(match.reference.y_min, match.candidate.y_min) + 24.0) * scale),
    )
    return left, top, max(1, right - left), max(1, bottom - top)


def preserve_vision_artifacts(
    gt_png: Path,
    other_png: Path,
    artifacts_dir: Path,
    page: int,
    dpi: int,
    large_matches: list[TextLineMatch],
) -> list[Path]:
    """Persist full pages, a pixel diff, and matched crops for model vision."""
    artifacts_dir.mkdir(parents=True, exist_ok=True)
    gt_artifact = artifacts_dir / f"page-{page}-gt.png"
    output_artifact = artifacts_dir / f"page-{page}-output.png"
    side_by_side = artifacts_dir / f"page-{page}-side-by-side.png"
    diff_artifact = artifacts_dir / f"page-{page}-diff-5pct.png"

    size = subprocess.run(
        [*(imagemagick_command("identify") or []), "-format", "%wx%h", str(other_png)],
        capture_output=True,
        text=True,
        check=True,
    ).stdout.strip()
    width, height = (int(value) for value in size.split("x", maxsplit=1))
    subprocess.run(
        [
            *(imagemagick_command("convert") or []),
            str(gt_png),
            "-background",
            "white",
            "-extent",
            size,
            str(gt_artifact),
        ],
        check=True,
        capture_output=True,
    )
    shutil.copy2(other_png, output_artifact)
    subprocess.run(
        [
            *(imagemagick_command("convert") or []),
            str(gt_artifact),
            str(output_artifact),
            "+append",
            str(side_by_side),
        ],
        check=True,
        capture_output=True,
    )
    diff = subprocess.run(
        [
            *(imagemagick_command("compare") or []),
            "-metric",
            "AE",
            "-fuzz",
            "5%",
            str(gt_artifact),
            str(output_artifact),
            str(diff_artifact),
        ],
        capture_output=True,
        text=True,
    )
    if diff.returncode not in (0, 1):
        raise SystemExit(f"ImageMagick failed to create {diff_artifact}: {diff.stderr.strip()}")

    paths = [gt_artifact, output_artifact, side_by_side, diff_artifact]
    for index, match in enumerate(large_matches, start=1):
        slug = re.sub(r"[^a-z0-9]+", "-", match.label.lower()).strip("-") or "text"
        crop_path = artifacts_dir / f"page-{page}-shift-{index:02d}-{slug[:48]}.png"
        left, top, crop_width, crop_height = shift_crop_box(match, dpi, width, height)
        crop = f"{crop_width}x{crop_height}+{left}+{top}"
        gt_crop = artifacts_dir / f".gt-crop-{index:02d}.png"
        output_crop = artifacts_dir / f".output-crop-{index:02d}.png"
        for source, destination in (
            (gt_artifact, gt_crop),
            (output_artifact, output_crop),
        ):
            subprocess.run(
                [
                    *(imagemagick_command("convert") or []),
                    str(source),
                    "-crop",
                    crop,
                    "+repage",
                    str(destination),
                ],
                check=True,
                capture_output=True,
            )
        subprocess.run(
            [
                *(imagemagick_command("convert") or []),
                str(gt_crop),
                str(output_crop),
                "+append",
                str(crop_path),
            ],
            check=True,
            capture_output=True,
        )
        gt_crop.unlink()
        output_crop.unlink()
        paths.append(crop_path)

    print("## Vision artifacts — open every image with Codex/Claude vision")
    print("  Numeric output does not complete the visual audit.")
    for path in paths:
        print(f"  {path}")
    return paths


def diagnose(
    geometry: dict[str, float], histogram_result: dict[str, float] | None
) -> None:
    """Say what the combination of axes means, and what to look at next.

    This is the point of running all three: a single number invites the
    wrong conclusion. A pixel count that rises can accompany a correct fix,
    and one that does not move at all can hide a large geometric
    improvement. The pattern across axes is what identifies the defect
    class.

    `histogram_result` is None when ImageMagick is absent. An axis that did not
    run must never read as an axis that agreed, so silence there is reported
    rather than folded into the verdict.
    """
    colour_measured: bool = histogram_result is not None
    histogram_result = histogram_result or {}
    print("## Reading")
    if not geometry:
        print("  Geometry could not be measured, so the other axes stand alone.")
        print("  Compare the pages by eye before trusting them.")
        return

    # A drift figure averaged over a handful of lines is noise. Korean
    # sheets match poorly because word segmentation differs between the two
    # PDFs, and one such page reported 4.11pt of "drift" from nine matched
    # lines out of sixty — a number with no meaning that would have sent the
    # next investigation chasing a row-height bug that is not there.
    matched: float = geometry.get("matched", 0.0)
    coverage: float = geometry.get("coverage", 0.0)
    if matched < 10 or coverage < 0.25:
        print(f"  Only {matched:.0f} lines matched ({coverage * 100:.0f}% of the GT).")
        print("  Treat the geometry figures as unreliable: too few samples, and")
        print("  the ones that matched are not a random selection. Compare the")
        print("  pages by eye, or measure a specific element directly, before")
        print("  drawing any conclusion from the drift above.")
        print()

    mad_y: float = geometry["mad_y"]
    mad_x: float = geometry["mad_x"]
    pages_differ: bool = geometry["page_mismatch"] > 0
    intersection: float = histogram_result.get("intersection", 1.0)
    ink_delta: float = histogram_result.get("ink_delta", 0.0)
    large_shift_count = int(geometry.get("large_shift_count", 0.0))
    large_shift_threshold = geometry.get("large_shift_threshold", 5.0)

    # Thresholds are deliberately loose: they route attention, they do not
    # decide correctness. A point of drift is invisible; ten is not.
    drifts_vertically: bool = mad_y > 2.0
    drifts_horizontally: bool = mad_x > 1.0
    # Judge colour on the shift, not the bin-wise intersection: the latter
    # flags smooth gradients that are pixel-identical to within dithering.
    # Measured separation on this corpus: renderer dithering across a smooth
    # gradient reaches 0.0003, and the half-width cell borders of #487
    # reached 0.0016 before the fix and 0.0004 after it.
    colour_differs: bool = colour_measured and histogram_result.get("shift", 0.0) > 0.001
    ink_differs: bool = colour_measured and abs(ink_delta) > 0.2

    findings: list[str] = []
    if large_shift_count:
        findings.append(
            f"{large_shift_count} matched text instance(s) move more than "
            f"{large_shift_threshold:.2f}pt. These named element-level differences "
            "remain valid even when the page has too few lines for aggregate MAD "
            "to be representative; inspect and track each one above."
        )
    if not colour_measured:
        findings.append(
            "The colour and pixel axes did not run — ImageMagick is absent, so "
            "a wrong fill, a recolour, or a missing element cannot be seen "
            "here at all. Geometry below stands alone; compare the pages by "
            "eye before concluding they agree."
        )
    if pages_differ:
        findings.append(
            "Pagination differs — content sits on the wrong page. Fix this "
            "first: every per-line measurement below it is contaminated by "
            "the accumulated drift that pushed it over."
        )
    if drifts_vertically:
        findings.append(
            f"Vertical drift {mad_y:.2f}pt — line advance, paragraph spacing, "
            "or row height. Compare consecutive-line pitch against the GT "
            "rather than absolute positions, so a constant offset near the "
            "top does not read as a spacing bug."
        )
    if drifts_horizontally:
        findings.append(
            f"Horizontal drift {mad_x:.2f}pt — indent, column width, or "
            "margin. If it grows across the page it is per-column and "
            "cumulative; if it is constant it is an indent or margin."
        )
    if colour_differs:
        findings.append(
            f"Colour distribution differs (shift {histogram_result.get('shift', 0.0):.4f}) — "
            "a fill, theme colour, or shading is wrong, or an element is "
            "missing entirely. Position measurements will not show this."
        )
    if ink_differs and not colour_differs:
        findings.append(
            f"Ink coverage is off by {ink_delta:+.3f}% while the colour "
            "distribution matches — the right things are drawn in the right "
            "colours but at the wrong size, or a font renders at a different "
            "weight."
        )

    if not findings:
        print("  No axis shows a material difference. What remains is font")
        print("  rasterisation and antialiasing; inspect crops at full")
        print("  resolution before concluding anything is wrong.")
        return
    for index, finding in enumerate(findings, start=1):
        print(f"  {index}. {finding}")

    if (
        colour_measured
        and not colour_differs
        and not ink_differs
        and (drifts_vertically or drifts_horizontally)
    ):
        print()
        print("  Colour and ink are unchanged while geometry moves: this is a")
        print("  pure layout difference. A pixel count may rise even as the")
        print("  fix is correct, because a displaced element that grows")
        print("  toward its true size overlaps GT less, not more.")


def report_matched_lines(gt: Path, other: Path, page: int = 1) -> None:
    """Per-instance positions for every text line matched spatially.

    Aggregate drift says a page is wrong; this says which line. Pairing the two
    PDFs by hand — taking the topmost line, or grepping for a prefix — silently
    matches the wrong line and produces impossible numbers, so the pairing here
    is the same duplicate-safe spatial match the geometry axis already trusts.
    """
    rows = match_text_line_instances(page_text_lines(gt, page), page_text_lines(other, page))

    print("## Matched lines — x/y position of each spatial text instance")
    if not rows:
        print("  no text instances matched")
        return
    print(f"  {'page':>4} {'GT x':>8} {'out x':>8} {'dx':>8} "
          f"{'GT y':>8} {'out y':>8} {'dy':>8}  text instance")
    for match in rows:
        reference = match.reference
        candidate = match.candidate
        print(
            f"  {reference.page + 1:>4} {reference.x_min:8.2f} {candidate.x_min:8.2f} "
            f"{match.dx:+8.2f} {reference.y_min:8.2f} {candidate.y_min:8.2f} "
            f"{match.dy:+8.2f}  {match.label[:52]}"
        )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("gt", type=Path, help="native Office export")
    parser.add_argument("output", type=Path, help="office2pdf output")
    parser.add_argument("--page", type=int, default=1)
    parser.add_argument("--dpi", type=int, default=150, help="at least 150")
    parser.add_argument(
        "--large-shift",
        type=float,
        default=5.0,
        metavar="PT",
        help="flag any matched text instance whose x or y moves by more than this (default: 5pt)",
    )
    parser.add_argument(
        "--audit",
        action="store_true",
        help="exit nonzero when a large per-instance text shift is found",
    )
    parser.add_argument(
        "--artifacts-dir",
        type=Path,
        help="preserve full GT/output pages, 5%% diff, and large-shift crops for model vision",
    )
    parser.add_argument(
        "--lines",
        action="store_true",
        help="list every matched text instance's x/y position without hand-pairing "
        "repeated labels between the two PDFs",
    )
    args = parser.parse_args()

    if args.dpi < 150:
        raise SystemExit("--dpi must be at least 150; hairlines vanish below that")
    require_vision_artifact_dependencies(args.artifacts_dir)

    print(f"GT     {args.gt}")
    print(f"output {args.output}")
    print(f"page {args.page} at {args.dpi} DPI\n")

    geometry = report_geometry(
        args.gt, args.output, page=args.page, large_shift=args.large_shift
    )
    print()
    if args.lines:
        report_matched_lines(args.gt, args.output, page=args.page)
        print()
    histogram_result: dict[str, float] | None = None
    if has_imagemagick():
        with tempfile.TemporaryDirectory() as raw_dir:
            out_dir = Path(raw_dir)
            gt_png = render_page(args.gt, args.page, args.dpi, out_dir, "gt")
            other_png = render_page(args.output, args.page, args.dpi, out_dir, "candidate")
            histogram_result = report_histogram(gt_png, other_png)
            print()
            report_pixels(gt_png, other_png, out_dir)
            if args.artifacts_dir is not None:
                matches = match_text_line_instances(
                    page_text_lines(args.gt, args.page),
                    page_text_lines(args.output, args.page),
                )
                page_matches = [
                    match
                    for match in matches
                    if abs(match.dx) > args.large_shift
                    or abs(match.dy) > args.large_shift
                ]
                print()
                preserve_vision_artifacts(
                    gt_png,
                    other_png,
                    args.artifacts_dir,
                    args.page,
                    args.dpi,
                    page_matches,
                )
    else:
        print("## Histogram and pixel difference — SKIPPED")
        print("  ImageMagick is absent: neither `magick` (7.x) nor all of "
              f"{', '.join(f'`{tool}`' for tool in IMAGEMAGICK_TOOLS)} (6.x)")
        print("  is on PATH. Install `imagemagick` to measure colour and ink.")
    print()
    diagnose(geometry, histogram_result)
    if args.audit and geometry.get("large_shift_count", 0.0):
        print()
        print(
            "AUDIT FAILED: large text-instance shifts are layout differences, not "
            "antialiasing; inspect and track every line above."
        )
        raise SystemExit(1)


if __name__ == "__main__":
    sys.exit(main())
