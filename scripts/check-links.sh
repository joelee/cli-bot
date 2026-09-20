#!/usr/bin/env bash
# Checks the links in the repository's Markdown files and fails, listing
# every problem, when:
# - a relative link points at a file or folder that does not exist;
# - a link names a heading (#anchor) that the target Markdown file lacks;
# - a link to this repository's `main` branch on GitHub names a missing path;
# - the crate's README, which crates.io shows, has a relative link:
#   crates.io resolves those against the crate's own folder, where they
#   break.
# Links in fenced code blocks and inline code are ignored, and other
# absolute links are not fetched. `just check` runs it.
#
# Usage: scripts/check-links.sh [repository root]
set -euo pipefail
export LC_ALL=C

root="${1:-$(git rev-parse --show-toplevel)}"
root="$(cd "$root" && pwd -P)"
cd "$root"

# The repository URL from [package], such as https://github.com/owner/name.
repo="$(awk '
    /^\[package\]/ { inside = 1; next }
    /^\[/ { inside = 0 }
    inside && /^repository[ \t]*=/ {
        sub(/^repository[ \t]*=[ \t]*"/, ""); sub(/".*$/, ""); print; exit
    }' Cargo.toml 2>/dev/null || true)"

# The tracked Markdown files in a Git repository, otherwise every one.
if [ "$(git rev-parse --show-toplevel 2>/dev/null || true)" = "$root" ]; then
    files="$(git ls-files '*.md')"
else
    files="$(find . -name '*.md' -not -path './target/*' -not -path './.git/*' | sed 's|^\./||' | sort)"
fi

# The crate's `readme` file, as a path from the root.
readmes=""
readme="$(sed -n 's/^readme[ \t]*=[ \t]*"\(.*\)".*/\1/p' Cargo.toml | head -n 1)"
if [ -n "$readme" ] && [ -f "$readme" ]; then
    readmes="$readme"$'\n'
fi

# Prints `<line><TAB><target>` for each inline link or image in a file.
links() {
    awk '
        /^[ \t]*(```|~~~)/ { fence = !fence; next }
        fence { next }
        {
            line = $0
            gsub(/`[^`]*`/, "", line)
            while (match(line, /\]\([^) \t]+/)) {
                print FNR "\t" substr(line, RSTART + 2, RLENGTH - 2)
                line = substr(line, RSTART + RLENGTH)
            }
        }' "$1"
}

# Prints the anchor GitHub gives each heading of a Markdown file: lower
# case, punctuation removed, spaces turned into hyphens.
anchors() {
    awk '
        /^[ \t]*(```|~~~)/ { fence = !fence; next }
        !fence && /^#+[ \t]/ { sub(/^#+[ \t]+/, ""); sub(/[ \t]+#+[ \t]*$/, ""); print }' "$1" |
        tr '[:upper:]' '[:lower:]' | sed -e 's/[^a-z0-9 _-]//g' -e 's/ /-/g'
}

problems=0
report() {
    echo "error: $*" >&2
    problems=1
}

# Checks that `dest` exists and, for a Markdown file, has heading `anchor`.
check_target() {
    local where="$1" target="$2" dest="$3" anchor="$4"
    if [ ! -e "$dest" ]; then
        report "$where: $target: ${dest#./} does not exist"
    elif [ -n "$anchor" ] && [ "${dest%.md}" != "$dest" ] && ! anchors "$dest" | grep -qxF -- "$anchor"; then
        report "$where: $target: ${dest#./} has no heading for #$anchor"
    fi
}

count=0
while IFS= read -r file; do
    [ -n "$file" ] && [ -f "$file" ] || continue
    count=$((count + 1))
    in_readme=0
    case $'\n'"$readmes" in *$'\n'"$file"$'\n'*) in_readme=1 ;; esac
    dir="$(dirname "$file")"
    while IFS=$'\t' read -r line target; do
        where="$file:$line"
        anchor=""
        case "$target" in *'#'*) anchor="${target#*#}" ;; esac
        path="${target%%#*}"
        case "$target" in
            mailto:*) continue ;;
            http://* | https://*)
                if [ -n "$repo" ]; then
                    case "$path" in
                        "$repo"/blob/main/* | "$repo"/tree/main/*)
                            rel="${path#"$repo"/*/main/}"
                            check_target "$where" "$target" "${rel%%\?*}" "$anchor"
                            ;;
                    esac
                fi
                continue
                ;;
        esac
        if [ "$in_readme" = 1 ] && [ -n "$path" ]; then
            report "$where: relative link $target in the crate README; crates.io resolves it against the crate's folder, so use an absolute URL"
            continue
        fi
        if [ -z "$path" ]; then
            check_target "$where" "$target" "$file" "$anchor"
        else
            check_target "$where" "$target" "$dir/$path" "$anchor"
        fi
    done < <(links "$file")
done <<< "$files"

if [ "$problems" -ne 0 ]; then
    exit 1
fi
echo "links ok: $count Markdown files checked"
