# Project skills (handoff)

Skills in this folder are meant for agents working in this repo. To install a skill so Cursor loads it:

```bash
# From repo root
mkdir -p .cursor/skills
cp -r docs/skills/landing-marketing-app .cursor/skills/
```

If `.cursor/` is in `.cursorignore`, Cursor may still load skills from `.cursor/skills/` (behavior is environment-dependent). The canonical copy lives here under `docs/skills/` for version control and handoff.

## Available skills

| Skill | Purpose |
|-------|---------|
| `landing-marketing-app` | How to work with the splash (5dlabs.ai) and marketing (CTO) apps: paths, deploy, agent tiles, conventions. Use when editing landing/marketing or when handed off to work on these apps. |
