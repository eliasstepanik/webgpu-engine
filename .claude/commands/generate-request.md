**Command Name:** `/generate-request`

---

### Purpose

Create a fully-populated request file in `.claude/requests/` by **researching** the argument text (feature idea, bug description, etc.) without asking the user follow-up questions.

---

### Usage

```bash
/generate-request <short-description>
```

*Example:*

```bash
/generate-request fix this error
```

---

### What the Command Must Do

1. **Resolve file path**

   * Create a concise, meaningful slug from the **key nouns/verbs** in the argument instead of the full phrase.
   * Append a uniqueness suffix – either the current `YYYYMMDD‑HHMM` timestamp **or** a 4‑character hash – to avoid collisions.
   * Save the file to `.claude/requests/<slug>.md`.

2. **Load template**

   * Start from `.claude/requests/templates/INITIAL.md`.

3. **Automatic research**
   **3.1 Codebase scan**

   * Search for the argument text and related symbols.
   * Collect:

      * file paths
      * code snippets (≤10 lines each) that show context.
        **3.2 External search**
   * Query web docs, GitHub, Stack Overflow.
   * Keep only authoritative URLs.
   * Note library versions and pitfalls.

4. **Fill template sections**

   * **FEATURE:** one-sentence summary built from the argument.
   * **EXAMPLES:**

      * List example files from `.claude/examples/` that illustrate the issue or pattern.
      * Add a one-line explanation per file.
   * **DOCUMENTATION:** bullet list of researched URLs.
   * **OTHER CONSIDERATIONS:**

      * Gotchas found during research.
      * Constraints (performance, security, style guides, etc.).

5. **Save file**

   * Write completed markdown to the resolved path.
   * Create `.claude/requests/done/` if missing and move any prior raw input file there (if applicable).

6. **Echo confirmation**

   * Output the final file path.

---

### Expected Output File Structure

```markdown
## FEATURE:
Fix error when uploading large files times out

## EXAMPLES:
.claude/examples/upload-large-file.py – reproduces timeout
.claude/examples/retry-wrapper.js – shows retry pattern

## DOCUMENTATION:
https://docs.aws.amazon.com/s3/…#multipart-upload
https://github.com/someorg/repo/pull/123

## OTHER CONSIDERATIONS:
- Library X v2.4 has a 15 MB buffer limit – must upgrade to ≥2.6
- Existing retry decorator in utils/retry.py handles only idempotent ops
```

---

### Quality Gate Checklist (self-audit)

* [ ] Feature sentence clear
* [ ] At least two example references
* [ ] All URLs valid and relevant
* [ ] Gotchas and constraints listed
* [ ] File saved in correct folder

---

### Notes for Implementers

* Keep sections terse; remove boilerplate comments.
* Clip code snippets to essentials; avoid giant blocks.
* Reject low-quality web sources; prefer official docs.
