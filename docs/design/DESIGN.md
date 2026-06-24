# Model Settings Design

## 1. Visual Theme & Atmosphere

Calm operational console: an assured, low-noise configuration surface that makes it obvious which model setup is active, whether its credential is present, and whether it passed a live test. The direction borrows the user-requested ccSwitch *workflow clarity*, not its exact visuals.

## 2. Color

- Canvas: warm near-white `#f7f7f4`
- Ink: `#18201c`
- Muted text: `#68736c`
- Lines: `#d8ddd7`
- Active accent: forest `#236343`
- Success: `#16794c`; warning/error: `#b44532`
- Selected configuration card: pale green `#edf6ef`

## 3. Typography

- Use the existing system sans stack for application consistency.
- Page title: 26px / 700; section title: 16px / 700; config name: 15px / 650.
- Metadata is compact (12–13px) and never competes with a connection result.

## 4. Spacing & Grid

- Desktop settings shell: content max-width 1040px; 24px vertical rhythm.
- Dashboard cards use a two-column grid above 880px and one column below it.
- Controls have a minimum 40px target; forms use 12px internal gaps.

## 5. Layout & Composition

- Main settings page is a dashboard: a single active-configuration status panel, two role cards (vision/report), then capture and safety controls.
- Each role card links to one dedicated configuration subpage rather than exposing a long, ambiguous form inline.
- The subpage has a clear back affordance, credential state, endpoint/model fields, one primary "save & test" action, and an explicit live result panel.

## 6. Components

- `ConnectionBadge`: unconfigured / saved / testing / connected / failed.
- `ModelRoleCard`: model, provider, endpoint host, last verification result, edit action.
- `EnvironmentCredentialNote`: states the required process environment variable; the app never accepts or persists a secret.
- `TestResult`: timestamped, role-specific result with a compact response preview.
- Primary action is "保存并测试"; the secondary action is "仅保存".

## 7. Motion & Interaction

- Buttons transition opacity and background in 140–180ms.
- Testing state reserves its layout and uses a small inline spinner; no jumping cards.
- Successful save and test promotes the card to connected immediately; failures preserve entered non-secret fields for correction.

## 8. Voice & Brand

Direct and factual Chinese: “已验证，可用于截图分析”, “尚未配置密钥”, “测试失败：…”. Avoid generic success toasts that do not say what was verified.

## 9. Anti-patterns

- No duplicate, unlabeled save buttons for one configuration.
- No claim that a key is usable merely because it exists in the keychain.
- No provider logo wall or provider-switching as the visual centerpiece.
- No gradients, glassmorphism, or oversized rounded cards.
- No hidden requirement to save the whole page before testing one model.
