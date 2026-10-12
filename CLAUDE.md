@AGENTS.md

## Claude Code

Everything above is [AGENTS.md](AGENTS.md), the manual shared with every contributor and every other agent. Claude Code does not read `AGENTS.md` on its own, so this file imports it. The import must stay at the top.

Below is only what is specific to Claude Code.

- **GitHub access.** In a local session, use the `gh` command with the maintainer's login. Cloud sessions have no `gh`; GitHub requests go through Claude Code's GitHub integration instead.
- **The `review-change` skill** in `.claude/skills/review-change/` is how Claude has a change reviewed by a separate reviewer agent, with the checklist that reviewer follows, and how it audits the whole book.
