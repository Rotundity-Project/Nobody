# UI Redesign Guide (V3)

## 1. Scope
This guide defines the V3 frontend architecture and UI conventions for Nobody.  
Goals:
- Keep the story loop readable and stable over long sessions.
- Make system complexity visible on demand.
- Keep components composable, testable, and evolvable.

## 2. Component Map
### 2.1 Route Layer
- `src/views/GameViewPage.vue`: route container for game view.
- `src/router/index.ts`: route registration for menu/script/game.

### 2.2 Scene Layer
- `src/components/GameRuntimeView.vue`: runtime orchestration.
- `src/components/MainMenu.vue`: main entry.
- `src/components/ScriptSelector.vue`: script selection flow.

### 2.3 Story Layer
- `src/components/StoryViewport.vue`: story viewport and reading locator.
- `src/components/StoryScenePanel.vue`: chapter header, rhythm, body wrapper.
- `src/components/VirtualStoryList.vue`: paragraph list with virtualization.
- `src/components/ChapterRecapCard.vue`: recap card.
- `src/components/ScrollToBottomButton.vue`: quick scroll action.

### 2.4 Interaction Layer
- `src/components/GameInteractionPanel.vue`: unified interaction container.
- `src/components/InputModeTabs.vue`: mode switching.
- `src/components/OptionListPanel.vue`: options list.
- `src/components/FreeTextInputPanel.vue`: free text input.
- `src/components/ContinueActionPanel.vue`: continue action panel.
- `src/components/InputStatusNotice.vue`: status hints.
- `src/components/LoadingStatePanel.vue`: loading + interrupt action.

### 2.5 Info/System Layer
- `src/components/InfoTabsDialog.vue`: right-side world drawer.
- `src/components/SystemCenterMenu.vue`: unified system entry menu.
- `src/components/NotificationCenter.vue`: banner + toast center.
- `src/components/GameSystemDialogs.vue`: save/load/settings dialogs.

### 2.6 Shared UI
- `src/shared/ui/UiButton.vue`: semantic button variants and focus-visible ring.
- `src/shared/ui/UiPanel.vue`: panel surface wrapper.

## 3. State and Composables
- `src/stores/gameStore.ts`: single source of truth.
- `src/composables/useStoryFlow.ts`: action execution, auto-advance, interruption.
- `src/composables/useInputMode.ts`: input mode + validation binding.
- `src/composables/useUiPanels.ts`: panel open/close state.
- `src/composables/useGameHotkeys.ts`: keyboard listener lifecycle.

Rules:
- Keep domain state in store.
- Keep view-level orchestration in composables.
- Keep components mostly declarative and prop-driven.

## 4. Theme and Tokens
Defined in `src/styles.css`:
- Foundation tokens (`--ref-*`)
- Semantic tokens (`--color-*`)
- Legacy aliases for gradual migration (`--bg-*`, `--panel`, `--accent`, etc.)

Rules:
- New styles should use semantic tokens first.
- Avoid hardcoded color literals in business components.
- Keep focus-visible states accessible (`outline/ring`).

## 5. Accessibility Baseline
- Keyboard:
  - `Esc` closes overlays.
  - Hotkeys are disabled in input contexts and when overlays are blocking.
- Focus:
  - Shared focus-visible styles in `styles.css`.
  - `UiButton` provides visible ring by default.
- Readability:
  - Story text uses larger line-height and high-contrast text variants.

## 6. Testing Baseline
- Unit/component tests: `npm run test`
- Smoke flow tests: `npm run test:smoke`
- Visual snapshots:
  - `src/components/__tests__/VisualSnapshot.test.ts`
  - Snapshot baseline in `src/components/__tests__/__snapshots__/VisualSnapshot.test.ts.snap`

When changing UI structure:
- Update component tests for behavior.
- Update visual snapshots intentionally and review diff.

## 7. Extension Rules
1. Add new UI primitive in `src/shared/ui` before duplicating style patterns.
2. Add new runtime orchestration logic in composables, not directly in large view files.
3. Keep route pages thin (`src/views`), push complexity to domain components.
4. For new global feedback, integrate into `NotificationCenter` first.
5. For new settings entries, route through `SystemCenterMenu`.

## 8. Safe Change Checklist
- Build passes: `npm run build`
- Smoke passes: `npm run test:smoke`
- Affected component tests pass.
- Snapshot updates reviewed.
- No encoding regression in docs/spec files.
