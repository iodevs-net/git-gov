# Edge Cases & Advanced Workflows 🛡️⚠️

Cliff-Watch is designed for sovereignty, but reality is messy. Here is how to handle common edge cases without losing your human score.

## 1. Pair Programming (2 Humans, 1 Keyboard)
**Scenario**: You are driving, your partner is navigating.
**Issue**: The "navigator" might dictate code that you type "too fast" or with different burstiness.
**Solution**:
- **Audit Mode**: Enable `audit_mode = true` during the session. The system will warn but not block.
- **High Focus**: Ensure the session is long. The "Focus Battery" aggregates energy over time. Even if typing is sporadic, the IDE presence counts.

## 2. Massive Refactors & Merge Conflicts
**Scenario**: You resolve a 50-file merge conflict or use `sed` to rename a variable globally.
**Issue**: High code entropy ($H_{code}$) with near zero motor cost ($E_{motor}$) -> Low Coupling Score.
**Solution**:
- **Accumulate Battery**: Spend 5-10 minutes reviewing the diffs *before* committing. Scrolling through the changes charges your "Reading/Navigation" battery.
- **Split Commits**: Separate the "Mechanical" changes (renames) from "Logic" changes. Automated refactors should be flagged with `[skip-cliff]` if your team policy permits, or committed by a CI bot.

## 3. AI Assistants (Copilot / ChatGPT)
**Scenario**: You paste a block of generated code.
**Issue**: High NCD (novelty) but zero burstiness (instant paste).
**Analysis**:
- Cliff-Watch *will* detect this as low motor entropy.
- To validate it, you must "pay" the cognitive cost. Review the code, rename variables, restructure functions.
- **The "Curation Tax"**: You cannot just paste and push. You must interact with the code enough to prove you understand it.

## 4. Accessibility & Alternative Inputs
**Scenario**: Voice coding, Eye tracking, or specialized keyboards (Vim-like).
**Issue**: Motor patterns differ significantly from standard typing.
**Solution**:
- Calibrate `pareto_alpha_min` in `cliff-watch.toml`. Voice coding often has higher burstiness (long pauses, fast bursts).
- Use `difficulty = "Easy"` to relax strict thermodynamic checks while maintaining focus tracking.

---
*Sovereignty requires intent. These workflows ensure that efficiency does not compromise intent.*
