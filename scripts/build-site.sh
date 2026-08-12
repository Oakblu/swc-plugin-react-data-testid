#!/usr/bin/env bash
# Assembles the GitHub Pages site into _site/ from the docs that already exist in the
# repo. Nothing here is a copy checked into git: the files are gathered at build time,
# so the published site cannot drift from the repository the way a docs/ mirror would.
#
#   ./scripts/build-site.sh          # build into _site/
#
# The point of publishing at all is that llms.txt only means something at an HTTPS
# path — the convention is a file an agent fetches at a known URL, and raw.githubusercontent
# is not that. After deploy the docs live at:
#
#   https://oakblu.github.io/plugin-react-data-testid-generator/llms.txt
#   https://oakblu.github.io/plugin-react-data-testid-generator/AGENTS.md
#   https://oakblu.github.io/plugin-react-data-testid-generator/schema/plugin-options.schema.json

set -euo pipefail
cd "$(dirname "$0")/.."

OUT="_site"
BASE="https://oakblu.github.io/plugin-react-data-testid-generator"

# Every file the site serves. A rename in the repo breaks the build here rather than
# silently publishing a site with a dead link.
SOURCES=(
  llms.txt
  AGENTS.md
  readme.md
  CHANGELOG.md
  schema/plugin-options.schema.json
)

rm -rf "$OUT"
mkdir -p "$OUT"

for f in "${SOURCES[@]}"; do
  if [ ! -f "$f" ]; then
    echo "error: $f is listed in SOURCES but does not exist." >&2
    echo "       Update SOURCES in scripts/build-site.sh if the file was renamed." >&2
    exit 1
  fi
  mkdir -p "$OUT/$(dirname "$f")"
  cp "$f" "$OUT/$f"
done

VERSION=$(node -p "require('./package.json').version")

# The description is interpolated into two contexts with different escaping rules, and a
# stray quote in package.json would silently produce a truncated meta tag and invalid
# JSON-LD — broken structured data reports no error, it just stops being read.
# DESC_JSON carries its own surrounding quotes; DESC_HTML is escaped for markup.
DESC_JSON=$(node -p "JSON.stringify(require('./package.json').description)")
DESC_HTML=$(node -p "require('./package.json').description \
  .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/\"/g, '&quot;')")

# The landing page is deliberately thin. It exists to be a crawlable HTML entry point
# carrying a description and JSON-LD; the real content is the markdown beside it.
cat > "$OUT/index.html" <<HTML
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>swc-plugin-react-data-testid — automatic data-testid attributes for React</title>
<meta name="description" content="$DESC_HTML">
<link rel="canonical" href="$BASE/">
<script type="application/ld+json">
{
  "@context": "https://schema.org",
  "@type": "SoftwareSourceCode",
  "name": "swc-plugin-react-data-testid",
  "description": $DESC_JSON,
  "version": "$VERSION",
  "codeRepository": "https://github.com/Oakblu/plugin-react-data-testid-generator",
  "programmingLanguage": ["Rust", "JavaScript"],
  "runtimePlatform": "SWC (WebAssembly)",
  "license": "https://opensource.org/licenses/MIT",
  "keywords": "swc, swc-plugin, react, data-testid, testid, e2e testing, playwright, cypress, testing library, test automation",
  "applicationCategory": "DeveloperApplication",
  "installUrl": "https://www.npmjs.com/package/swc-plugin-react-data-testid"
}
</script>
<style>
  :root { color-scheme: light dark; }
  body { max-width: 44rem; margin: 3rem auto; padding: 0 1.25rem;
         font: 16px/1.6 ui-sans-serif, system-ui, -apple-system, sans-serif; }
  code, pre { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: .9em; }
  pre { padding: .85rem 1rem; overflow-x: auto; border-radius: .5rem;
        background: rgba(127,127,127,.14); }
  h1 { margin-bottom: .25rem; } h2 { margin-top: 2.25rem; }
  .sub { opacity: .75; margin-top: 0; }
  ul { padding-left: 1.1rem; } li { margin: .4rem 0; }
</style>
</head>
<body>
<h1>swc-plugin-react-data-testid</h1>
<p class="sub">$DESC_HTML</p>

<pre><code>npm install --save-dev swc-plugin-react-data-testid</code></pre>
<pre><code>// .swcrc
{ "jsc": { "experimental": { "plugins": [["swc-plugin-react-data-testid", {}]] } } }</code></pre>

<h2>Documentation</h2>
<ul>
  <li><a href="llms.txt">llms.txt</a> — index of these docs for LLMs and agents</li>
  <li><a href="AGENTS.md">AGENTS.md</a> — full specification of the id-naming algorithm,
      precise enough to predict every generated <code>data-testid</code> without a build</li>
  <li><a href="readme.md">README</a> — installation, per-framework setup, limitations</li>
  <li><a href="schema/plugin-options.schema.json">Options JSON Schema</a> — every option, typed</li>
  <li><a href="CHANGELOG.md">Changelog</a></li>
</ul>

<h2>Links</h2>
<ul>
  <li><a href="https://www.npmjs.com/package/swc-plugin-react-data-testid">npm</a></li>
  <li><a href="https://github.com/Oakblu/plugin-react-data-testid-generator">GitHub</a></li>
</ul>
</body>
</html>
HTML

# Validate what was just written. Invalid JSON-LD is inert rather than loud — a consumer
# silently stops reading it — so the failure has to be raised here, in CI, instead.
node -e "
  const fs = require('fs');
  const html = fs.readFileSync('$OUT/index.html', 'utf8');
  const m = html.match(/<script type=\"application\/ld\+json\">([\s\S]*?)<\/script>/);
  if (!m) { console.error('error: no JSON-LD block in index.html'); process.exit(1); }
  try { JSON.parse(m[1]); } catch (e) {
    console.error('error: JSON-LD is not valid JSON — ' + e.message);
    console.error(m[1]);
    process.exit(1);
  }
"

echo "Built $OUT/ (version $VERSION):"
find "$OUT" -type f | sort | sed 's/^/  /'
