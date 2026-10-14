#!/usr/bin/env bash
# Cross-consumer build sweep: build and test every repo that depends on
# agnostik against the IN-FLIGHT tree (the working tree, uncommitted changes
# included) before a release, under each consumer's OWN cyrius pin.
#
# Consumers are discovered, not listed: every sibling checkout whose
# cyrius.cyml declares `[deps.agnostik]`. Each one is copied twice into a
# scratch directory and run twice:
#
#   base — its manifest untouched, so it resolves the agnostik tag it pins;
#   new  — its `[deps.agnostik]` rewritten to `path = "../agnostik"`, with a
#          copy of this working tree as that sibling, so the consumer's own
#          resolve vendors the in-flight code exactly as it vendors a release.
#
# Swapping lib/agnostik.cyr by hand does NOT work: `cyrius build` and
# `cyrius test` re-resolve deps first and put the pinned tag's copy back from
# ~/.cyrius/deps. So after the new build, every vendored agnostik module is
# compared with the in-flight source and a mismatch fails the sweep rather
# than reporting a vacuous pass.
#
# Per consumer it reports build and test results for both runs, the binary
# size, the warnings that appear only in the new run, and every duplicate-
# definition warning that involves an agnostik file (a symbol collision with
# another library the consumer vendors).
#
# Usage:
#   scripts/consumer-sweep.sh [--keep] [--list] [consumer ...]
#     --keep   keep the scratch directory and print its path (it is kept
#              anyway when a consumer regresses or a run is invalid)
#     --list   print the discovered consumers (census) and exit
#     consumer limit the sweep to these directory names
#
# Environment:
#   AGNOSTIK_CONSUMERS_DIR  where the consumer checkouts live
#                           (default: the directory holding this checkout)
#   SWEEP_TIMEOUT           seconds per build or test file (default 900)
#
# Exit status: 0 when no consumer regressed (a consumer green in base is green
# in new, its assertion count did not drop, and no new agnostik-involved
# warning appeared); 1 on any regression or invalid run; 2 on usage or
# preflight errors. A consumer already red in base is reported, not counted.
#
# The base run resolves the pinned tag from ~/.cyrius/deps, or clones it, so a
# cold cache needs network access.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONSUMERS_DIR="${AGNOSTIK_CONSUMERS_DIR:-$(dirname "$REPO_ROOT")}"
TIMEOUT_S="${SWEEP_TIMEOUT:-900}"
KEEP=0
LIST_ONLY=0
ONLY=()

for a in "$@"; do
    case "$a" in
        --keep) KEEP=1 ;;
        --list) LIST_ONLY=1 ;;
        -h|--help) sed -n '2,45p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
        -*) echo "consumer-sweep: unknown option $a" >&2; exit 2 ;;
        *) ONLY+=("$a") ;;
    esac
done

# ── discovery ────────────────────────────────────────────────────────────
# The value of `key = "…"` inside the [deps.agnostik] section of a manifest.
dep_field() {
    awk -v key="$2" '
        /^\[/ { insec = ($0 == "[deps.agnostik]"); next }
        insec && $0 ~ "^" key "[ \t]*=" {
            sub(/^[^=]*=[ \t]*"/, ""); sub(/".*$/, ""); print; exit
        }' "$1"
}
# The [deps.agnostik] modules list, one path per line (multi-line arrays too).
dep_modules() {
    awk '
        /^\[/ { insec = ($0 == "[deps.agnostik]"); next }
        insec && /^modules[ \t]*=/ { inmod = 1 }
        inmod {
            line = $0
            while (match(line, /"[^"]+"/)) {
                print substr(line, RSTART + 1, RLENGTH - 2)
                line = substr(line, RSTART + RLENGTH)
            }
            if ($0 ~ /\]/) { exit }
        }' "$1"
}
pkg_pin() { grep -m1 -oE '^cyrius[ \t]*=[ \t]*"[^"]+"' "$1" | sed -E 's/.*"([^"]+)"/\1/'; }

CONSUMERS=()
for m in "$CONSUMERS_DIR"/*/cyrius.cyml; do
    [ -f "$m" ] || continue
    d="$(dirname "$m")"
    name="$(basename "$d")"
    [ "$(cd "$d" && pwd)" = "$REPO_ROOT" ] && continue
    grep -q '^\[deps\.agnostik\]' "$m" || continue
    if [ ${#ONLY[@]} -gt 0 ]; then
        keep=0
        for o in "${ONLY[@]}"; do [ "$o" = "$name" ] && keep=1; done
        [ $keep -eq 1 ] || continue
    fi
    CONSUMERS+=("$name")
done

if [ ${#ONLY[@]} -gt 0 ]; then
    for o in "${ONLY[@]}"; do
        found=0
        for c in "${CONSUMERS[@]}"; do [ "$c" = "$o" ] && found=1; done
        [ $found -eq 1 ] || { echo "consumer-sweep: '$o' is not a consumer under $CONSUMERS_DIR" >&2; exit 2; }
    done
fi
if [ ${#CONSUMERS[@]} -eq 0 ]; then
    echo "consumer-sweep: no [deps.agnostik] consumers under $CONSUMERS_DIR" >&2
    exit 2
fi

if [ $LIST_ONLY -eq 1 ]; then
    printf '%-12s %-9s %-8s %s\n' consumer tag pin modules
    for c in "${CONSUMERS[@]}"; do
        m="$CONSUMERS_DIR/$c/cyrius.cyml"
        printf '%-12s %-9s %-8s %s\n' "$c" "$(dep_field "$m" tag)" "$(pkg_pin "$m")" \
            "$(dep_modules "$m" | paste -sd ' ' -)"
    done
    exit 0
fi

# ── preflight ────────────────────────────────────────────────────────────
command -v timeout > /dev/null || { echo "consumer-sweep: needs coreutils timeout" >&2; exit 2; }
if ! (cd "$REPO_ROOT" && cyrius distlib --check > /dev/null 2>&1); then
    echo "consumer-sweep: dist/agnostik.cyr is stale against src/ — run 'cyrius distlib' first" >&2
    exit 2
fi

SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/agnostik-sweep.XXXXXX")"
if [ $KEEP -eq 0 ]; then trap 'rm -rf "$SCRATCH"' EXIT; fi
mkdir -p "$SCRATCH/base" "$SCRATCH/new" "$SCRATCH/logs"

# Copy a checkout's tracked and untracked-but-not-ignored files, plus its
# lib/ (often git-ignored, and holding the vendored deps a resolve reuses).
copy_tree() {
    mkdir -p "$2"
    (cd "$1" && git ls-files -co --exclude-standard -z | tar --null -T - -cf -) | tar -xf - -C "$2"
    if [ -d "$1/lib" ]; then cp -a "$1/lib/." "$2/lib/"; fi
}

# The in-flight agnostik, as the sibling the rewritten manifests point at.
copy_tree "$REPO_ROOT" "$SCRATCH/new/agnostik"

# Point a manifest's [deps.agnostik] at ../agnostik: drop its git / tag /
# branch / rev / path lines and add the path right after the header.
rewrite_to_path() {
    awk '
        /^\[/ { insec = ($0 == "[deps.agnostik]"); print; if (insec) print "path = \"../agnostik\""; next }
        insec && /^(git|tag|branch|rev|path)[ \t]*=/ { next }
        { print }' "$1" > "$1.sweep" && mv "$1.sweep" "$1"
}

# Where `cyrius deps` vendors an agnostik module: dist/agnostik.cyr lands as
# lib/agnostik.cyr, src/<x>.cyr as lib/agnostik_<x>.cyr.
vendored_path() {
    local b
    b="$(basename "$1" .cyr)"
    if [ "$b" = agnostik ]; then echo "lib/agnostik.cyr"; else echo "lib/agnostik_$b.cyr"; fi
}

# The test files a consumer runs: its [build] test entry, then tests/**/*.tcyr.
test_files() {
    local t
    t="$(awk '/^\[/ { inb = ($0 == "[build]"); next }
              inb && /^test[ \t]*=/ { sub(/^[^=]*=[ \t]*"/, ""); sub(/".*$/, ""); print; exit }' cyrius.cyml)"
    if [ -n "$t" ] && [ -f "$t" ]; then echo "$t"; fi
    if [ -d tests ]; then find tests -name '*.tcyr' | LC_ALL=C sort; fi
}

# Run build + tests in the current directory. Writes <log>.build, <log>.test
# and a <log>.sum line: build_rc size test_files test_fail assertions.
run_phase() {
    local log="$1" brc size tf=0 tfail=0 asserts=0 out t
    timeout "$TIMEOUT_S" cyrius build > "$log.build" 2>&1
    brc=$?
    size=0
    local o
    o="$(awk '/^\[build\]/ { inb = 1; next } /^\[/ { inb = 0 }
              inb && /^output[ \t]*=/ { sub(/^[^=]*=[ \t]*"/, ""); sub(/".*$/, ""); print; exit }' cyrius.cyml)"
    if [ $brc -eq 0 ] && [ -n "$o" ] && [ -f "$o" ]; then size=$(wc -c < "$o"); fi
    : > "$log.test"
    if [ $brc -eq 0 ]; then
        while IFS= read -r t; do
            tf=$((tf + 1))
            out="$(timeout "$TIMEOUT_S" cyrius test "$t" 2>&1)"
            local rc=$?
            printf '=== %s (rc %d)\n%s\n' "$t" "$rc" "$out" >> "$log.test"
            [ $rc -eq 0 ] || tfail=$((tfail + 1))
            local n
            n="$(printf '%s\n' "$out" | grep -E '^[0-9]+ passed, [0-9]+ failed' | tail -1 | grep -oE '^[0-9]+')"
            asserts=$((asserts + ${n:-0}))
        done < <(test_files)
    fi
    echo "$brc $size $tf $tfail $asserts" > "$log.sum"
}

# Warning lines, positions and numbers normalised so a shifted line number or
# a changed byte count does not read as a new warning.
norm_warnings() { grep -iE 'warning' "$1" 2> /dev/null | sed -E 's/[0-9]+/N/g' | LC_ALL=C sort -u; }

regressions=0
invalid=0
declare -a SUMMARY

for c in "${CONSUMERS[@]}"; do
    src="$CONSUMERS_DIR/$c"
    man="$src/cyrius.cyml"
    tag="$(dep_field "$man" tag)"
    pin="$(pkg_pin "$man")"
    head="$(git -C "$src" rev-parse --short HEAD 2> /dev/null || echo '?')"
    dirty="$(git -C "$src" status --porcelain 2> /dev/null | wc -l)"
    echo "── $c (agnostik ${tag:-untagged}, cyrius $pin, HEAD $head$([ "$dirty" -gt 0 ] && echo ", $dirty uncommitted"))"

    copy_tree "$src" "$SCRATCH/base/$c"
    copy_tree "$src" "$SCRATCH/new/$c"
    rewrite_to_path "$SCRATCH/new/$c/cyrius.cyml"
    L="$SCRATCH/logs/$c"

    (cd "$SCRATCH/base/$c" && run_phase "$L.base")
    (cd "$SCRATCH/new/$c" && run_phase "$L.new")
    read -r bb bs btf btfail ba < "$L.base.sum"
    read -r nb ns ntf ntfail na < "$L.new.sum"

    # Guard against a vacuous pass: the new run must have vendored the
    # in-flight modules, byte for byte.
    stale=""
    while IFS= read -r mod; do
        v="$(vendored_path "$mod")"
        if ! cmp -s "$SCRATCH/new/$c/$v" "$SCRATCH/new/agnostik/$mod"; then stale="$stale $v"; fi
    done < <(dep_modules "$SCRATCH/new/$c/cyrius.cyml")

    newwarn="$(comm -13 <(norm_warnings "$L.base.build") <(norm_warnings "$L.new.build"))"
    dups="$(grep -iE 'duplicate' "$L.new.build" 2> /dev/null | grep -i 'agnostik' \
        | sed -E 's/^.*(warning:)/\1/' | LC_ALL=C sort -u)"
    newwarn_ag="$(printf '%s\n' "$newwarn" | grep -i 'agnostik')"

    verdict=ok
    if [ -n "$stale" ]; then
        verdict=INVALID; invalid=$((invalid + 1))
        echo "   INVALID: the new run did not vendor the in-flight tree:$stale"
    elif [ "$bb" -ne 0 ] || [ "$btfail" -ne 0 ]; then
        verdict="red-at-base"
        echo "   base already red (build rc $bb, $btfail failing test file(s)); not counted"
    elif [ "$nb" -ne 0 ] || [ "$ntfail" -ne 0 ] || [ "$na" -lt "$ba" ] || [ -n "$newwarn_ag" ]; then
        verdict=REGRESSED; regressions=$((regressions + 1))
    fi

    printf '   base: build rc %s, %s B, %s test file(s), %s failing, %s assertions\n' "$bb" "$bs" "$btf" "$btfail" "$ba"
    printf '   new:  build rc %s, %s B, %s test file(s), %s failing, %s assertions\n' "$nb" "$ns" "$ntf" "$ntfail" "$na"
    if [ -n "$newwarn" ]; then
        echo "   warnings only in new (normalised):"
        printf '%s\n' "$newwarn" | sed 's/^/     /'
    fi
    if [ -n "$dups" ]; then
        echo "   duplicate definitions involving agnostik:"
        printf '%s\n' "$dups" | sed 's/^/     /'
    fi
    if [ "$verdict" = REGRESSED ]; then
        echo "   REGRESSED — logs: $L.new.build, $L.new.test"
    fi
    SUMMARY+=("$(printf '%-12s %-9s %-8s %-11s %4s/%-4s %s' "$c" "${tag:--}" "$pin" "$verdict" "$ba" "$na" "$(printf '%s\n' "$dups" | grep -c . )")")
done

echo
printf '%-12s %-9s %-8s %-11s %-9s %s\n' consumer tag pin verdict asserts agnostik-dups
for l in "${SUMMARY[@]}"; do echo "$l"; done
echo
echo "summary: ${#CONSUMERS[@]} consumer(s), $regressions regressed, $invalid invalid"
if [ $regressions -eq 0 ] && [ $invalid -eq 0 ]; then
    if [ $KEEP -eq 1 ]; then echo "scratch kept: $SCRATCH"; fi
    exit 0
fi
trap - EXIT
echo "scratch kept for inspection: $SCRATCH"
exit 1
