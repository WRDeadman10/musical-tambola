# Queue: 50 bounded tasks for AG (lead: Claude, 2026-11-21)

Rules for every task: no owner decision needed, no build (the lead builds the installer and runs the full suite; AG may run `cargo test` and `npm run test:ui` for checks). One finish message per BATCH of 5 (task ids, files, one line each). **Do not wait for review:** after a batch, start the next 5 at once; at the next submission first read the lead's verdict on the previous batch and fix HOTFIX items before new tasks. Silence is never a reason to stop; only a named blocker (one line in an A- message) pauses you, then take the next independent task. No ack-only messages. A rejected item comes back as a HOTFIX. Audits report `file:line` and a verdict (BUG / RISK / OK) with a failing scenario for every BUG. Writes: new files only unless listed; pure rules get tests; no behaviour change to shipped defaults. Never stage logs, screenshots, caches, `docs/Tel_Mailbox/`, or anything containing tokens. Commits: AG is the only one who runs git; commit a batch when the lead orders it and the test result is posted.

Board sync: `move_board_task(id, 'in-progress')` when starting a batch, `add_board_comment` for findings, `move_board_task(id, 'review')` when the finish message is posted.

## Batch 1: finish the wiring (open items from the last work)
| # | Task | Scope / files | Acceptance |
|---|---|---|---|
| 1 | Commit the pending language fix | `src/stores/appStore.ts`, `src/test/languageReactivity.test.tsx` | commit message names the sidebar bug, trailer "Implemented by Claude", pushed |
| 2 | Status docs for BT-35..BT-39 | `docs/tasks.md`, `docs/ai_handoff.md`, `docs/decisions.md` | every task has a status and commit id; no invented facts |
| 3 | Changelog check | `CHANGELOG*` vs `git log` since 1.0.1 | each entry maps to a commit; missing entries added |
| 4 | Command inventory | all `#[tauri::command]` vs `src/lib/api/*.ts` | table: command, Rust args, TS caller, test covering it; gaps listed |
| 5 | Run-and-test doc | README or `docs/` run section | commands copied from real runs |

## Batch 2: bug-hunt audits, backend core (read-only)
| # | Task | Scope / files | Acceptance |
|---|---|---|---|
| 6 | Audit game draw path | `commands/game.rs`, game service | draw twice with the same expected position, draw after end, concurrent pause; verdicts |
| 7 | Audit claim validation | claim service and commands | late claim always rejected, Full House by flag not name, duplicate claim, snapshot mismatch; verdicts |
| 8 | Audit ffmpeg download | `services/ffmpeg.rs` | temp files cleaned on every error path, partial zip, disk full, cancelled download; verdicts |
| 9 | Audit presentation window lifecycle | `commands/presentation.rs` | open twice, close while hidden, monitor unplugged, status event after destroy; verdicts |
| 10 | Audit poster storage | `commands/media.rs`, media repository | poster refcount and cleanup, replaced poster still used by a game snapshot; verdicts |

## Batch 3: audits, data side (read-only)
| 11 | Backup and restore | backup commands, db connection | restore over an open DB, wrong schema version, corrupt file; verdicts |
| 12 | Admin PIN | `services/admin_pin.rs` | lockout timing, reset on restart, guard missing on a mutating command; verdicts |
| 13 | Ticket generation | ticket service | uniqueness, row/column rules for Classic, determinism, batch size 10000 memory; verdicts |
| 14 | Song Grid rules | ticket service, format commands | 3x5 no blanks for every song count 15-90, format change with existing batches; verdicts |
| 15 | Lucky draw | lucky draw service | duplicate winners, empty list, weighting, repeat draw; verdicts |

## Batch 4: audits, frontend glue (read-only)
| 16 | Store reactivity | `src/stores`, every `useAppStore((s) => s.t)` use | a selector that returns a stable value while its data changes; verdicts |
| 17 | Listener leaks | every `listen*` and `useEffect` in `src` | missing cleanup, promise resolved after unmount; verdicts |
| 18 | IPC spreads | `create_lucky_draw`, `scan_media_folder` and any new spread in `src/lib/api` | argument names match Rust; verdicts |
| 19 | Error mapping | `toErrorMessage`, `errorCodes.ts` vs Rust error codes | a Rust code with no translation; verdicts |
| 20 | Locale parity | `en.json` vs `gu.json` | keys missing or untranslated in either file; verdicts |

## Batch 5: audits, UI and tests (read-only)
| 21 | Tests that cannot fail | `src/test`, Rust tests | placeholder or self-fulfilling assertions with `file:line`; verdicts |
| 22 | Accessibility | forms and dialogs | unlabeled inputs, dialogs without role, focus traps; verdicts |
| 23 | Print CSS | `index.css` print block, print page | element visible in print that should not be; page break risks; verdicts |
| 24 | Gujarati overflow | long Gujarati strings in buttons and cards | clipped or overflowing text with the longest string per screen; verdicts |
| 25 | Keyboard shortcuts | `useHostShortcuts`, dialogs | shortcut firing while typing in an input or with a dialog open; verdicts |

## Batch 6: test hardening (new test files, pure rules only)
| 26 | Classic ticket properties | new Rust test file | 15 numbers, 5 per row, column ranges, no duplicates over 1000 seeds |
| 27 | Song Grid ticket properties | new Rust test file | 15 cells, 3x5, all within song count, over every count 15-90 |
| 28 | Scheme mask rules | new Rust test file | each scheme type accepts and rejects the right tickets |
| 29 | Late claim rule | new Rust test file | claim after the next draw is always rejected |
| 30 | Number formatting | new frontend test file | Gujarati and English numerals, 0, 90, negative input |

## Batch 7: more pure rules (new files + tests; opt-in, unwired; lead wires)
| 31 | Draw order helper | new pure module | seeded shuffle of 1..N, no repeats, N from 15 to 90 |
| 32 | Ticket range parser | new pure module | "T-000001..T-000050" style ranges, invalid input fails closed |
| 33 | Poster path helper | new pure module | safe file name from slot number and hash, no traversal |
| 34 | ffmpeg version parser | new pure module | parses real `ffmpeg -version` first lines, unknown format returns none |
| 35 | Download size formatter | new pure module | bytes to MB text, unknown total handled |

## Batch 8: tools and scripts (offline)
| 36 | Locale parity checker | script under `scripts/` | exits non-zero on a missing key, tested on real files |
| 37 | IPC inventory generator | script under `scripts/` | reproduces the task 4 table from source |
| 38 | Mailbox id audit | script under `scripts/` | finds duplicate or skipped MSG ids in `docs/Tel_Mailbox/MAILBOX.md` |
| 39 | Doc link checker | script under `scripts/` | broken relative links in `docs/` |
| 40 | Release checklist script | script under `scripts/` | prints version in package.json, tauri.conf.json, Cargo.toml and the installer name; fails on mismatch |

## Batch 9: documentation
| 41 | Host guide (English) | `docs/` | every button on the game screen explained, screenshots not required |
| 42 | Host guide (Gujarati) | `docs/` | same content, reviewed against `gu.json` terms |
| 43 | Print template guide | `docs/` | regions from `templateRegions.ts`, image rules, steps |
| 44 | Dual-screen and ffmpeg troubleshooting | `docs/` | commands and messages copied from real runs |
| 45 | Open-issue digest | `docs/` | every BUG/RISK from batches 2-5 with status |

## Batch 10: wiring and verification helpers
| 46-49 | Fix the BUG items from batches 2-5 that are backend-only | files named in each finding | one commit per fix, test added, no change to shipped defaults |
| 50 | Final consistency pass | `docs/tasks.md`, handoff, changelog after 1-49 | numbers and issue statuses match; mismatches fixed |

---
Lead verdict log (one line per batch): `Batch n: ACCEPTED | HOTFIX (items) - message id`

Batch 1: HOTFIX closed (ticket code format fixed) - MSG-266
Batches 2-8: ACCEPTED on checks (fmt, clippy -D warnings, cargo test 142+, vitest 200 pass) but audits found 0 BUG while the real app had 6 (see ai_handoff); treated as weak evidence
Batches 9-10: HOTFIX - OPEN_ISSUES_DIGEST.md cites files that do not exist (ffmpeg_downloader.rs, ticket_generation.rs, claims/useClaims.ts, claims/QuickClaim.tsx, lib/api/errorMapping.ts) - MSG-268
