# AGENTS.md

## Core principle

Before making non-trivial code changes, understand the existing code structure first.

For repository understanding, architecture analysis, bug localization, call-flow tracing, impact analysis, and large refactors, use CodeGraph before falling back to repeated `rg`, grep, or manual file-reading loops.

Prefer small, focused changes. Do not rewrite unrelated code.

## CodeGraph usage

When CodeGraph MCP tools are available, use them first for structural code understanding.

Prefer the highest-level CodeGraph exploration tool when available, such as `codegraph_explore`, before using lower-level tools.

Use CodeGraph first for:

* finding feature entry points
* understanding repository architecture
* tracing caller and callee relationships
* understanding request, data, and state flows
* locating relevant modules for bugs or refactors
* estimating the impact area before editing code
* checking what may be affected by a change
* understanding how a public API, route, command, component, service, or model is connected to the rest of the project

Use `rg`, grep, or direct file reads first only for:

* exact string search
* simple single-file edits
* documentation-only changes
* checking dependency versions
* searching config keys
* cases where CodeGraph is unavailable or returns insufficient results

## Required workflow for non-trivial changes

For any non-trivial code change, follow this workflow:

1. Use CodeGraph to explore the relevant feature, module, route, function, class, component, or service.
2. Identify the entry points, key files, important symbols, callers, callees, and likely impact area.
3. Summarize the current flow briefly before editing.
4. Make the smallest reasonable code change.
5. Avoid unrelated refactors.
6. Run the most relevant checks available in the repository.
7. Summarize what changed, which files were modified, and what checks were run.

## When analyzing bugs

When asked to fix or investigate a bug:

1. Use CodeGraph to locate the likely flow related to the bug.
2. Trace the path from the entry point to the failing behavior.
3. Identify the smallest safe fix.
4. Check whether the fix affects callers, callees, public APIs, data models, or tests.
5. Add or update tests when appropriate.

Do not guess the location of a bug only from filenames. Use CodeGraph to understand the actual relationships when possible.

## When refactoring

Before refactoring:

1. Use CodeGraph to identify all relevant callers and callees.
2. Estimate the blast radius of the change.
3. Prefer incremental refactors.
4. Preserve public behavior unless the user explicitly asks to change it.
5. Do not combine broad formatting changes with functional changes.

## Fallback behavior

If CodeGraph is unavailable, not initialized, or unable to answer the query:

1. State that CodeGraph was unavailable or insufficient.
2. Fall back to `rg`, grep, direct file reads, and normal repository inspection.
3. Continue the task using the best available project context.

Do not stop working only because CodeGraph is unavailable.

## Coding style

Follow the existing project style.

Prefer:

* simple and readable code
* existing project patterns
* existing utilities and abstractions
* minimal changes
* clear names
* explicit error handling where appropriate

Avoid:

* unrelated rewrites
* unnecessary new dependencies
* large architectural changes without user approval
* changing public APIs unless necessary
* modifying authentication, payment, permission, or data-deletion logic without extra care

## Package manager

Use the package manager already used by the repository.

Check lockfiles first:

* `pnpm-lock.yaml` means use `pnpm`
* `package-lock.json` means use `npm`
* `yarn.lock` means use `yarn`
* `bun.lock` or `bun.lockb` means use `bun`
* `uv.lock` means use `uv`
* `poetry.lock` means use `poetry`
* `Cargo.lock` means use `cargo`

Do not switch package managers unless the user explicitly asks.

## Verification

After changing code, run the most relevant checks available.

Prefer this order:

1. targeted tests for changed files or modules
2. type check
3. lint
4. full test suite
5. build

If a command fails, inspect the error and fix the root cause when possible.

If a check cannot be run, explain why and mention what should be run manually.

## Git behavior

Do not create commits unless the user explicitly asks.

Do not push changes unless the user explicitly asks.

Before finishing, summarize:

* what was changed
* why it was changed
* which files were modified
* what checks were run
* any remaining risks or follow-up work
