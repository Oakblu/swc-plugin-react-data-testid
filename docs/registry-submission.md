# SWC plugin registry submission

Prepared entry for [plugins.swc.rs](https://plugins.swc.rs/), plus the facts a submission
needs. Nothing here is applied automatically — submitting is a manual step.

## What the registry actually is

It is a **compatibility** registry, not a plugin directory. It maps a plugin's published
versions to the `swc_core` version each was built against, so a user on a given host
(Next.js, Rspack, `@swc/core`) can see which plugin version will load. Per the
[announcement](https://blog.swc.rs/2024-8-27-swc-plugin-registry), entries are **not**
auto-discovered from npm: the maintainers crawl repositories listed in a YAML file, find
deployment commits via `package.json`, and read `swc_core` versions out of `Cargo.lock`.

Being listed is therefore worth doing for a reason beyond discovery: it is what tells a
user — or an agent resolving a version conflict — that this plugin is compatible with
their toolchain at all.

## Entry

```yaml
- repository: https://github.com/Oakblu/plugin-react-data-testid-generator
  packages:
    - swc-plugin-react-data-testid
```

## Supporting facts

| Field                | Value                                                        |
| -------------------- | ------------------------------------------------------------ |
| npm package          | `swc-plugin-react-data-testid`                                |
| Repository           | https://github.com/Oakblu/plugin-react-data-testid-generator  |
| `swc_core`           | 79.x (see `Cargo.toml`; `Cargo.lock` is committed)            |
| Verified against     | `@swc/core` 1.16.18, Next.js 16.4.0                           |
| Wasm target          | `wasm32-wasip1`                                               |
| Built artifact       | `plugin.wasm` (npm `main`)                                    |

## Where to send it

**Unconfirmed — verify before submitting.** The announcement post states the registry is
driven by a YAML file of repository URLs and npm package names and that the source is
public, but does not name the repository or document a submission process, and no repo
matching "registry" is visible in the `swc-project` org. Options, best first:

1. Open an issue or PR against the registry's source repo once located — check the
   footer/source link on plugins.swc.rs itself.
2. Post in [swc-project/swc discussion #4818, "Plugin encyclopedia: tell us your SWC
   plugins"](https://github.com/swc-project/swc/discussions/4818) — the known
   author-facing channel for announcing a plugin, with the entry above.

## Re-submitting

The registry reads `swc_core` from the repo's `Cargo.lock` at a deployment commit, so a
version bump needs no new submission — only a `swc_core` upgrade is worth flagging, since
that is what changes the compatibility range users are matched against.
