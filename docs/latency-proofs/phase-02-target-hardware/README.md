# Phase 2 Target Hardware Latency Proof Artifacts

This directory is the canonical in-repo archive location for **Phase 2 target-hardware evidence** supporting the Phase 2 latency claim: **p95 < 500ms** for **end-of-speech → transcript JSONL emit**.

The intent is that a human can run the baseline procedure once on target hardware and commit the resulting evidence in a consistent structure.

## What must be captured (required artifacts)

For each run, capture and commit:

- `transcript.jsonl` — stdout capture (**JSONL-only**, one JSON object per line)
- `timing.log` — stderr capture (instrumentation / timing logs)
- Computed **p95** for `e2e_ms` across the run’s utterances

**Important:** stdout must remain JSONL-only and logs must go to stderr (D-20). Capture output like:

```bash
./target/release/hd-linux-voice --config /path/to/config.yaml -v \
  > transcript.jsonl \
  2> timing.log
```

## Run naming convention

Create a new folder per target-hardware run, using:

`run-YYYYMMDD-HHMM-{hostname-or-machine}/`

Example:

`run-20260422-1249-steamdeck/`

Inside each run folder, commit:

```
run-YYYYMMDD-HHMM-{machine}/
  transcript.jsonl
  timing.log
  RESULTS.md
```

Where `RESULTS.md` is filled from `../RESULTS.template.md` and includes the computed p50/p95/p99 and machine + build metadata.

## Template

- Use `../RESULTS.template.md` as the starting point.
- Keep artifacts **text-only** and repo-friendly (no binary attachments).
