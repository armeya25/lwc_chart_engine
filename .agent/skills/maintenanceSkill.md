# Changelog Maintenance Skill

Use this skill to automatically update the `docs/changelog.md` file after significant changes or at the end of a session.

## 📋 Instructions

1.  **Identify and Synchronize Version**: 
    - **Single Source of Truth**: Always use `pyproject.toml` (`version = "..."`) as the master version.
    - **Automated Sync**: Run `helpers/upload_to_git.sh` to automatically propagate the version from `pyproject.toml` to:
      - `src/src-tauri/Cargo.toml`
      - `helpers/create-wheels.sh`
      - `.github/workflows/build_wheels.yml` (Manual check recommended)
    - **Check**: Ensure `README.md` and `docs/api.md` mention the current version (e.g., v0.9.7).
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
5.  **Tooling Standards**:
    - **Always use `uv`** for all Python package installations and management. 
    - **Always use `.venv`** as the standard virtual environment directory within the project root.

## 📝 Template

```markdown
## [VERSION] - YYYY-MM-DD

### [Category Name]
- **Summary**: Brief description of the change.
- **Details**: Impact or technical implementation note.
```

---
*maintained by amit vaidya*
