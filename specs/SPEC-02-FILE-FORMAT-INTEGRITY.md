# SPEC-02: File Format Integrity

**Priority:** P0 — Data loss risk
**File:** `agentic-planning/src/file_format.rs`
**Estimated changes:** ~120 lines

## Problem

The `.aplan` file format has broken metadata tracking and no integrity
verification. Corrupted files load silently. Crash leaves orphaned temp files.

## Requirements

### R1: Fix footer metadata

- `write_count` must increment on each save (read from existing file, +1)
- `last_session` must be set from engine session state
- `flags` should encode feature flags (e.g., bit 0 = compressed, bit 1 = encrypted)
- `footer_checksum` must be computed (blake3 of payload)

### R2: Checksum verification on load

When loading a `.aplan` file:
1. Read footer checksum
2. Compute blake3 of payload section
3. Compare — if mismatch, return `PlanningError::CorruptedFile` with details
4. If checksum is all-zeros (legacy file), warn but proceed

### R3: Section offsets

Populate `goal_offset`, `decision_offset`, `commitment_offset`, `dream_offset`,
`federation_offset` with actual byte offsets into the payload. This enables
future partial-read optimization.

### R4: Crash recovery

On `save()`:
1. Write to `path.aplan.tmp`
2. fsync the temp file
3. Rename temp → final (atomic on POSIX)
4. On `load()`, if `.aplan.tmp` exists and `.aplan` does not, recover from tmp
5. If both exist, use `.aplan` (the completed write)

## Acceptance Criteria

- [ ] `write_count` increments correctly across saves
- [ ] `footer_checksum` computed and verified
- [ ] Corrupted files produce clear error (not silent load)
- [ ] Crash recovery handles orphaned .tmp files
- [ ] Round-trip test: save → corrupt → load → error
