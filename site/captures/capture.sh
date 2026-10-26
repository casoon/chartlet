#!/usr/bin/env bash
# Regenerates the terminal captures on the start page: real chartlet runs.
#   site/captures/capture.sh   (after cargo build --release, or with CHARTLET_BIN set)
# 1. examples/energy-flow.json, the hero chart, renders without a message.
# 2. fixture/duplicate.json repeats a label and is refused with code and path.
# 3. narrow-flow.json is examples/release-flow.json with "width": 420 and "height": 1200; it renders, with a
#    warning that names the width it needs.
# Rebuild the scenes after a capture:
#   cd site/captures && for f in *.terminal.yaml; do npx -y -p @casoon/castwright@0.5.0 -p shiki castwright build "$f" --format svg -o .; done
set -euo pipefail

repo="$(cd "$(dirname "$0")/../.." && pwd)"
out="$repo/site/captures"
bin="${CHARTLET_BIN:-$(cargo metadata --format-version 1 --no-deps --manifest-path "$repo/Cargo.toml" | node -pe 'JSON.parse(require("fs").readFileSync(0)).target_directory')/release/chartlet}"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

cp "$repo/examples/energy-flow.json" "$out/fixture/duplicate.json" "$work/"
node -e 'const s=require(process.argv[1]); s.width=420; s.height=1200; process.stdout.write(JSON.stringify(s,null,2)+"\n")' \
  "$repo/examples/release-flow.json" > "$work/narrow-flow.json"

run() { # <name> <args…>: stdout+stderr to <name>.txt, exit code to <name>.exit
  local name="$1"; shift
  local status=0
  (cd "$work" && "$bin" "$@") > "$out/$name.txt" 2>&1 || status=$?
  echo "$status" > "$out/$name.exit"
}

run render-ok render energy-flow.json -o sankey.svg
(cd "$work" && ls ./*.svg | sed 's#^\./##') > "$out/render-ok-ls.txt"
(cd "$work" && grep '"label"' duplicate.json) > "$out/duplicate-labels.txt"
run render-error render duplicate.json -o chart.svg
run render-warning render narrow-flow.json -o flow.svg
