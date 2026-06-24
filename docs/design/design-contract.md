# Model Settings Reference Contract

## Goal

Refactor the settings experience for Daytrace / 日报助手 so users can configure a vision model and a report model with an unambiguous save-and-live-test workflow.

**Artifact:** React/Tauri desktop settings and model configuration subpages.  
**Audience:** a technically capable person configuring their own API credentials.

## Evidence

| Evidence | Confidence | What it implies |
| --- | --- | --- |
| User requested a ccSwitch-like testing experience | provided | Fast verification and transparent connection state take priority. |
| Existing app has separate vision and text providers | observed | The UI needs role-specific configurations, not a generic provider switcher. |
| Existing flow has a key save, page save, and test action | observed | The refactor must collapse or clearly distinguish these operations. |
| External ccSwitch reference was not retrievable in this environment | observed | Visual details are inferred; do not copy a specific layout or brand asset. |

## Reference boundaries

| Keep | Change | Do not copy |
| --- | --- | --- |
| Clear configuration cards, immediate connection feedback, readable status | Apply the workflow to screenshot analysis and report generation | ccSwitch logos, visual assets, wording, exact screen layout, or provider-specific branding |

## Final stance

Use a calm operational dashboard with a dedicated subpage per model role. The central promise is evidence: each role card tells the user whether it has a saved credential and a successful live verification; a single “save and test” action binds configuration persistence to real model reachability.

## Risks and unknowns

- A key can be saved but later revoked; connection status should be treated as last verified, not permanent truth.
- The actual ccSwitch visual system was not inspected; this is an adaptation of its described interaction quality only.
- Existing dirty-worktree changes must remain intact.

## Quality gate

- [ ] A user can see vision and report model state without opening a form.
- [ ] Saving a credential and testing it is one understandable flow.
- [ ] Each test says whether it tested text generation or image analysis.
- [ ] Fields remain editable after a failed test.
- [ ] Keyboard focus, disabled states, and error text are usable.
