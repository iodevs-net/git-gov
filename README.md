# Git-Gov: Decentralized Code Governance Protocol

> **Status:** 🚧 Pre-Alpha / Request for Comments (RFC)

## The Problem: The Entropy Crisis
Open Source is facing a "Denial of Service" on maintainer attention. The rise of AI-generated code has lowered the barrier to contribution to zero, flooding repositories with syntactically correct but contextually empty code (low-entropy contributions).

Maintainers cannot distinguish between:
1.  **Human Work:** A thoughtful, iterative process of trial and error (High Entropy).
2.  **AI Generation:** Instantaneous, probabilistic token generation (Low Entropy).

## The Solution: Proof of Human Work (PoHW)
`git-gov` is a CLI tool and protocol designed to act as a **"Passport Scanner"** for your commits.

It runs locally on the developer's machine, observing the *physics* of code creation (burstiness, editing rhythm, time-to-commit) without recording the content (privacy-preserving). It then signs the commit with a cryptographic "Proof of Human Work".

### How it works
1.  **Watch:** The daemon monitors file edit patterns (not content).
2.  **Measure:** Calculates a "Humanity Score" based on edit entropy.
3.  **Sign:** Injects a `Git-Trailer` metadata into your commit.

### Vision
We are not building a wall. We are building a **Fast Track**.
Contributors with `git-gov` verification provide maintainers with instant trust, allowing their PRs to be prioritized.

## Roadmap
- [ ] **Sentinel (Rust):** CLI to monitor filesystem events.
- [ ] **Protocol:** Define the `.provenance` JSON schema.
- [ ] **Integrations:** GitHub Actions to verify the proof.

---
*License: MIT*
