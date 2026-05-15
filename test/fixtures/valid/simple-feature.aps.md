# Dark Mode Feature

| ID   | Owner | Status |
| ---- | ----- | ------ |
| DARK | @test | Ready  |

## Purpose

Add dark mode support to the application for improved user experience in low-light conditions.

## Success Criteria

- [ ] Users can toggle between light and dark modes
- [ ] Preference persists across sessions

## Work Items

### DARK-001: Add theme toggle

- **Intent:** Allow users to switch between themes
- **Expected Outcome:** Toggle button in header switches theme
- **Validation:** `npm test -- theme.test.ts`

### DARK-002: Persist preference

- **Intent:** Remember user's theme choice
- **Expected Outcome:** Theme persists after page reload
- **Validation:** `npm test -- theme-persistence.test.ts`
- **Dependencies:** DARK-001
