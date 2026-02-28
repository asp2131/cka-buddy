# Golden Calibration Suite

This directory contains benchmark grading cases used to calibrate rubric-based evaluation.

## Purpose

- Ensure scoring consistency across evaluators and model revisions.
- Provide representative pass / borderline / fail examples per phase.
- Validate feedback quality expectations (what should and should not be said).

## Layout

- `index.yaml` — canonical manifest for suite metadata and case registry.
- `rubric/` — scoring policy and phase-specific interpretation notes.
- `cases/phase-{0,1,2}/` — gold cases with strict YAML schema.
- `reviews/` — calibration review templates and change logs.

## Case Authoring Rules

1. Follow the strict schema exactly (see existing case files).
2. Keep submissions realistic and concise.
3. Prefer concrete feedback requirements over vague guidance.
4. Use `expected.score_range` as a narrow calibration target.
5. Keep metadata fields current for auditability.

## Validation Checklist

- [ ] Every phase has one pass and one borderline/fail case.
- [ ] `expected.rubric_version` is `v1`.
- [ ] Criterion expectations are integers in `[0,4]`.
- [ ] Required feedback points are actionable and specific.
- [ ] Forbidden feedback patterns cover common evaluator mistakes.

## Ownership

Maintained by: `curriculum-team`
Last initialized: `2026-02-28`
