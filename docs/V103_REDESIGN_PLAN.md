# v1.0.3 UI redesign plan (Claude, senior UI/UX game designer)

Status: PLAN, awaiting owner go-ahead on the three decisions at the end. No redesign code is written yet.

## 1. Direction
"Game-show control room": dark stage-lit surfaces, one strong accent, gold reserved for winners and prizes. The host screen is a cockpit that never scrolls; the projector is a stage that looks good from the back of a hall.

## 2. Design system first (one source of truth)
- Tokens in `src/index.css` (`@theme`): colour roles (surface 0-3, border, text 1-3, accent, accent-soft, success, warning, danger, gold), type scale (12/14/16/20/28/40/72), spacing on a 4 px grid, radii 8/12/20, elevation (3 levels + accent glow), motion (120/200/320 ms, one easing).
- Dark and light themes from the same tokens (the Theme setting already exists); contrast AA checked for both.
- Fonts: keep Noto Sans Gujarati; add a display weight for numbers. Gujarati strings are tested at their longest, not only English.
- Components rebuilt on the tokens, one file each: Button (4 variants, 3 sizes), IconButton, Card, Stat, Badge/Chip, Input, Select, Switch, Segmented control, Tabs, Dialog, Toast, Table, EmptyState, Skeleton, ProgressBar, Kbd.
- Rule: no raw colour or pixel value in feature code; every feature uses components.

## 3. App shell
Collapsible icon rail (labels on hover/expand), grouped as Setup (Media, Tickets, Schemes) / Play (Game, Claims, History) / System (Settings). Slim top bar: event name, active game chip, projector chip, language switch. Page header pattern: title, one-line help, primary action on the right.

## 4. Screens, in this order
1. Game Screen (host cockpit, no scroll at 1366x768 and 1920x1080):
   - Left, 60%: Audience Preview as a 16:9 stage that fills the available height, with ONE transport bar (play/pause/seek/volume). The projector video is slaved to it: pause here, pause there.
   - Under the stage: big current ball, Show/Hide answer, Show/Hide number, previous-songs rail.
   - Right, 40%: tabs Claim | Forecast | Projector | Lucky draw. Next, Pause, End and the countdown live in a fixed action bar that is always visible.
2. Projector: full-bleed media, number ball top-right, answer lower-third, winner announcement with the pattern animation and confetti, reduced-motion respected.
3. Tickets: generator wizard with live preview ticket, batch table with simple ticket numbers, print preview with real template.
4. Media Library: grid view with posters, status chips, bulk actions, ffmpeg card collapsed into a toolbar.
5. Settings: grouped cards with a left section list; new Danger zone (reset settings / delete all data).
6. Dashboard and Claim Desk, History last.

## 5. How I work, step by step
1. Static HTML mockups of the host cockpit, projector and one ticket screen (dark + light, English + Gujarati). You approve or change them. Nothing is coded before this.
2. Tokens and components, with a living component page and tests.
3. Screen by screen, each in its own commit batch. Per screen, acceptance is: screenshots at 1366x768 and 1920x1080 in English and Gujarati from the REAL built app, no scroll or overflow where promised, keyboard path works, axe-style checks pass.
4. Final pass: motion, empty and error states, installer, real-app run with the 67-video data.

## 6. Decisions for the owner (defaults in brackets)
- Approve mockups before any code? [yes]
- Keep the dark theme as the default and ship light as an option? [yes]
- Ticket numbers: plain 1, 2, 3 per batch, shown everywhere and typed in Quick Claim? [yes]
