# Changelog Maintenance Skill

Use this skill to automatically update the `docs/changelog.md` file after significant changes or at the end of a session.

## 📋 Instructions

1.  **Identify and Synchronize Version**: 
    - Check the `VERSION` variable in `helpers/upload_to_git.sh`.
    - **Crucial**: Ensure this version matches exactly in:
      - `pyproject.toml` (`version = "..."`)
      - `src/src-tauri/Cargo.toml` (`version = "..."`)
      - `README.md` (Installation and example snippets)
    - If code changes are significant, increment the version (e.g., 0.2.7 -> 0.2.8).
2.  **Summarize Recent Changes**:
    - Analyze the conversation history and the diffs of modified files.
    - Group changes logically into these categories:
      - `🚀 Core Improvements / Features`
      - `🛠 Build & Workflow Optimizations`
      - `🎯 UI & API Enhancements`
      - `⚙ Internal Refactoring`
3.  **Format the Entry**:
    - Use the established Markdown style: `## [VERSION] - YYYY-MM-DD`.
    - Use bullet points for individual changes.
    - Be concise but descriptive about the "Why" and "What".
4.  **Update the File**:
    - Prepend the new entry to the top of the `docs/changelog.md` file, below the main header.
    - Ensure the signature at the bottom remains: `*maintained by amit vaidya*`.

## 📝 Template

```markdown
## [VERSION] - YYYY-MM-DD

### [Category Name]
- **Summary**: Brief description of the change.
- **Details**: Impact or technical implementation note.
```

---
*maintained by amit vaidya*
