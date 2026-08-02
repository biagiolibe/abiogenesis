# Task [ID] — [Task Title]

> **ID**: `[NNN]`
> **Category**: [Architecture / Feature / Bugfix / Refactor / etc.]
> **Priority**: [🔴 P1 / 🟡 P2 / 🟢 P3]
> **Estimate**: [~1h / ~2h / etc.]
> **Assigned to**: [Claude CLI / unassigned]
> **Session**: [Conversation ID or time reference]

---

## 🎯 Objective

[What needs to be done?]
[Why is it necessary?]

---

## 📋 Acceptance Criteria

[A task is considered complete when:]
- [ ] The code compiles without errors.
- [ ] Feature X works as described.
- [ ] [Add specific criteria...]

---

## 📁 Relevant Files

| File | Role |
|------|------|
| `src/module/file.ts` | Description of the role. |

---

## 🧩 Technical Context

[Paste type/interface definitions here, or describe the current state of the code.]

- **Current behavior**: [What happens now?]
- **Desired behavior**: [What should happen after?]

---

## 🔨 Suggested Implementation

[Recommended steps for the AI agent]

1. [Step 1]
2. [Step 2]

```
// Optional example snippet
```

---

## ⚠️ Constraints and Caveats

- **Style**: Follow the conventions defined in `TECH_DESIGN.md`.
- **Performance**: [Any specific constraints]

---

## 🔗 Dependencies

- **Depends on**: [previous task ID or none]
- **Blocks**: [next task ID or none]

---

## 🤖 How to delegate this task to Claude CLI

```bash
claude "$(cat tasks/[NNN]-name.md)"$'\n\nExecute this task in the current project.'
```
