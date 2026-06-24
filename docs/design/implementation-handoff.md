# Implementation Handoff — Model Settings

- Replace the inline provider forms with a settings dashboard and a dedicated `ModelConfig` route for each role.
- Keep existing React, Zustand, TanStack Query, and CSS architecture; use the tokens in `DESIGN.md`.
- Read secrets only from process environment variables. Never accept or persist a secret in the app.
- Add persisted verification metadata to settings: last-tested result, time, and an error summary per role.
- The subpage primary action saves the non-secret config, saves an entered secret, then calls the role-appropriate live test. A secondary save option is allowed but must be visually subordinate.
- First implementation must prove: unsaved field edits can be tested, saved configurations can display last verification, and a vision test calls image analysis.
