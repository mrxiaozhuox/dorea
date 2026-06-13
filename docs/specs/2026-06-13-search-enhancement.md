# Search Enhancement: Value Search + Simplified Syntax

**Date:** 2026-06-13
**Branch:** `feat/search-enhancement`

## Overview

Replace the current key-only wildcard search with a unified search command
that supports key/value/all source selection and simplified pattern matching.

## Current State

```bash
search <pattern> [limit]   # key-only, * and ? wildcards
```

## New Design

### Syntax

```bash
search <pattern>              # search key + value (default)
search key <pattern> [limit]  # search key only
search value <pattern> [limit] # search value only
```

### Pattern Matching

| Pattern | Meaning | Example |
|---------|---------|---------|
| `word` | substring match | `admin` matches `user:admin`, `admin_config` |
| `^word` | prefix match | `^user` matches `user:xxx`, not `xuser` |
| `word$` | suffix match | `.log$` matches `error.log`, not `log.txt` |
| `*` `?` | wildcards (legacy) | `user*:admin`, `L?u` |

### Backward Compatibility

- **Breaking:** Old `search <pattern> [limit]` syntax removed.
- Old CLI scripts need update: `search user*` → `search key user*`

## Implementation Plan

### Files Changed

1. **`src/tool.rs`** — `fuzzy_search()` refactored
   - Add `^` / `$` anchor detection
   - Default to substring matching (no `*` required)
   - Preserve `*` / `?` wildcard support

2. **`src/command.rs`** — Search command handler
   - Parse `key`/`value` source flag
   - Default: search both key and value
   - Value search: iterate keys → read value → string match

### Scope ~100 lines

| File | Lines |
|------|:-----:|
| `tool.rs` | ~30 |
| `command.rs` | ~35 |
| Tests | ~20 |
| CHANGELOG | ~5 |

### Performance

- Key search: O(n) over keys (unchanged)
- Value search: O(n×m) — reads every value
- Default (all): O(n + n×m) — both key and value

## Testing

- Unit tests for new pattern matching
- Integration test: `search admin` returns keys whose value contains "admin"
- Backward compat: verify `*` / `?` still work
