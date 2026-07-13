# `cyrius audit` invokes `~/.cyrius/bin/check.sh` but install never ships it

**Discovered:** 2026-04-26 during agnostik 1.0.0 closeout pass (mid-pass toolchain bump 5.7.6 → 5.7.12)
**Severity:** Low (tooling — `cyrius audit` is broken on a fresh install of any 5.7.x; users must run the constituents `build` / `test` / `fmt --check` / `lint` individually as a workaround. No correctness or runtime impact on shipped agnostik.)
**Affects:** Cyrius toolchain 5.7.x (verified on 5.7.12; the `cmd_audit` codepath
in `cbt/commands.cyr:395-398` calls `make_path(_scripts_dir, "check.sh")`
without ever shipping that script in the install bundle).
**Filed by:** agnostik (1.0.0 audit, [`docs/audit/2026-04-26-audit.md`](../../audit/2026-04-26-audit.md))

## Summary

`cyrius audit` is the documented entry point for the project's
"full check" gate (`self-host, test, fmt, lint`) — see `cyrius --help`:

```
Quality:
  audit                           full check: self-host, test, fmt, lint
```

On any fresh `cyriusly install 5.7.x`, the command immediately fails:

```
$ cyrius audit
/bin/sh: /home/macro/.cyrius/bin/check.sh: No such file or directory
```

Root cause is in the upstream cyrius repo at two cooperating sites:

1. `cbt/commands.cyr:395-398` —
   ```cyrius
   fn cmd_audit() {
       var script = make_path(_scripts_dir, "check.sh");
       return run_script(script, 0, 0);
   }
   ```
   Looks for `check.sh` next to the cyrius binary
   (`$CYRIUS_HOME/bin/`).

2. `cyrius.cyml`'s release manifest declares which scripts the
   installer copies into `$CYRIUS_HOME/versions/<v>/bin/`:
   ```
   scripts = ["cyriusly", "cyrius-init.sh", "cyrius-port.sh",
              "cyrius-repl.sh", "cyrius-watch.sh"]
   ```
   `check.sh` is **not** in the array, even though `scripts/check.sh`
   exists in the repo and is the canonical "full check" runner used
   by upstream's CI.

The two halves disagree: `cmd_audit` expects the script to be
deployed alongside the binary; the release pipeline never deploys
it.

## Reproduction

```bash
$ cyriusly install 5.7.12
Cyrius 5.7.12 installed successfully!

$ ls $HOME/.cyrius/bin/check.sh
ls: cannot access '/home/macro/.cyrius/bin/check.sh': No such file or directory

$ cyrius audit
/bin/sh: /home/macro/.cyrius/bin/check.sh: No such file or directory
$ echo $?
127
```

Verified on a freshly-`cyriusly install`'d 5.7.12 (no manual file
removal performed; the bin directory is the installer's output as-is).

## Proposed fix

Either of the following would close the gap:

1. **Add `check.sh` to the release-manifest scripts array.** Single
   line edit in cyrius's `cyrius.cyml`:

   ```diff
   - scripts = ["cyriusly", "cyrius-init.sh", "cyrius-port.sh",
   -            "cyrius-repl.sh", "cyrius-watch.sh"]
   + scripts = ["cyriusly", "cyrius-init.sh", "cyrius-port.sh",
   +            "cyrius-repl.sh", "cyrius-watch.sh", "check.sh"]
   ```

   `scripts/install.sh:109-110` already iterates this array and
   copies each entry into `$CYRIUS_HOME/versions/$VERSION/bin/` with
   chmod +x. No installer changes needed.

2. **Inline the audit subcommand into `cbt/commands.cyr`.** Run the
   self-host, test, fmt, lint sequence directly from
   `cmd_audit()` instead of shelling out to `check.sh`. Decouples
   cyrius from a shipped script and makes the audit command robust
   to install-bundle drift.

(1) is the smaller change and matches the existing release-manifest
mechanism. (2) is more durable but a larger refactor.

## Workaround used by agnostik

For the 1.0.0 closeout audit, agnostik ran the audit-equivalent gate
manually:

```sh
cyrius self                      # self-host
for t in tests/tcyr/*.tcyr; do cyrius test "$t"; done   # test
for f in src/*.cyr; do cyrius fmt --check "$f"; done    # fmt
for f in src/*.cyr; do cyrius lint "$f"; done           # lint
```

Same coverage; just no single entry point.

## Update 2026-07-13 — cyrius 6.4.62 (during the 6.3.15 → 6.4.62 pin bump)

The **original `check.sh`-missing failure is resolved** on 6.4.62.
`cyrius audit` now runs to completion — it executes `fmt`, `lint`,
`docs`, `tests`, and `bench` phases inline (proposed-fix option 2
above appears to have landed; `$CYRIUS_HOME/versions/6.4.62/bin/check.sh`
still does not exist, yet the command no longer shells out to it). The
`fmt` and `lint` phases pass clean.

**A different limitation replaced it, so `cyrius audit` is still not a
usable single-command gate for this project.** Its `tests` and `bench`
sub-phases compile the `.tcyr`/`.bcyr` files **without resolving the
manifest `[deps] stdlib` preamble**, so functions the test files rely
on the manifest to inject go undefined:

```
── tests ──
warning: undefined function 'bayan_json_get'
warning: undefined function 'clock_now_ns'
error: refusing to emit binary with 1 reachable undefined function(s)
       (pass --allow-undef to downgrade)
  FAIL: tests/tcyr/agnostik.tcyr (compile error)
10 passed, 5 failed
```

The 5 "failures" are the test files that reach `clock_now_ns` (the
`chrono` stdlib fn — called from `audit.cyr`/`agent.cyr`/`telemetry.cyr`/
`secrets.cyr`) as a live symbol. This is a **false failure**: the same
files compile and pass cleanly under the real gate —
`cyrius test tests/tcyr/agnostik.tcyr` → `223 passed, 0 failed` — because
`cyrius test`/`build`/`bench` resolve `chrono`/`bayan` from
`cyrius.cyml [deps] stdlib` (the test files deliberately do **not**
manually `include "lib/chrono.cyr"` etc.). `audit`'s stricter compile
skips that resolution.

**Net status:** the specific bug this file was filed for (missing
`check.sh`) is **closed** on 6.4.62. A new, distinct `audit`
stdlib-resolution defect is open. The agnostik workaround (run
`self`/`test`/`fmt --check`/`lint` individually — all green on 6.4.62)
is unchanged. Re-check whether the resolution defect is fixed at the
next pin bump; if so, this file can move to `archive/`.
