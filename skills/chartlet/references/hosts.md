# Installing the skill and the MCP server

A skill is a folder with a `SKILL.md`; there is no packaging step. The MCP server is separate and
optional (see the table in `SKILL.md`). Install the skill, then the server if you want the tools.
Hosts change; the commands below follow the hosts' own documentation at the time of writing
(Claude Code: code.claude.com/docs/en/skills and /mcp; Codex: its skills and MCP pages).

Get the folder: copy `skills/chartlet/` from a clone of the repository
(<https://github.com/casoon/chartlet>).

## Claude Code

Skill:

```sh
# for you, in every project
mkdir -p ~/.claude/skills && cp -R skills/chartlet ~/.claude/skills/
# or for one repository (commit it to share it)
mkdir -p .claude/skills && cp -R skills/chartlet .claude/skills/
```

A skills directory that did not exist when the session started needs `/reload-skills`; changes to
existing ones are picked up in the session. Update: copy the folder again. Uninstall: delete the
folder.

MCP server (optional):

```sh
claude mcp add --env CHARTLET_MCP_ROOT="$PWD" --transport stdio chartlet -- npx -y @casoon/chartlet-mcp
claude mcp list            # chartlet should be listed
claude mcp remove chartlet # uninstall
```

`--scope project` writes `.mcp.json` for the team; Claude Code asks to approve it. Update: with
`npx -y` the current version is fetched; pin one as `@casoon/chartlet-mcp@0.21.0` if you need
reproducibility.

## Codex

Skill: Codex scans `$HOME/.agents/skills` (user) and `.agents/skills` in the repository (and its
parents):

```sh
mkdir -p ~/.agents/skills && cp -R skills/chartlet ~/.agents/skills/
```

Changes are detected automatically; restart Codex if the skill does not appear. Update: copy the
folder again. Uninstall: delete the folder, or keep it and disable it in `~/.codex/config.toml`:

```toml
[[skills.config]]
path = "/home/you/.agents/skills/chartlet/SKILL.md"
enabled = false
```

MCP server (optional), in `~/.codex/config.toml` or with `codex mcp add`:

```toml
[mcp_servers.chartlet]
command = "npx"
args = ["-y", "@casoon/chartlet-mcp"]

[mcp_servers.chartlet.env]
CHARTLET_MCP_ROOT = "/path/to/project"
```

```sh
codex mcp add chartlet --env CHARTLET_MCP_ROOT=/path/to/project -- npx -y @casoon/chartlet-mcp
codex mcp list
```

## Troubleshooting

| Symptom | Cause and fix |
| --- | --- |
| The skill is not offered | Wrong folder level: `SKILL.md` must be directly in `<skills dir>/chartlet/`. Reload or restart the host |
| `chartlet_*` tools are missing | The server is not configured or not approved; run the host's list command. Needs Node 22.12 or newer |
| `outputPath` is refused | `CHARTLET_MCP_ROOT` is not set to an existing directory |
| `npx` cannot download | No network or a registry mirror; install `@casoon/chartlet-mcp` once and point the command at it |
| No theme applied | `chartlet.theme.json` must be in the working directory (CLI) or `CHARTLET_MCP_ROOT` (MCP); the manifest names the theme that was applied |
| No tools at all | Use the CLI route in [cli.md](cli.md) |

## What has been verified

Verified: the CLI route (chart, diagram and a correction request rendered from a clean
directory with the commands in [cli.md](cli.md)) and every MCP tool in isolation by the package's
tests. The installation commands above are taken from the hosts' documentation; the skill has
not yet been run end to end inside Claude Code or Codex, and no host-specific manifest is
shipped. Treat cross-host behavior as unproven until a host run is recorded here.
