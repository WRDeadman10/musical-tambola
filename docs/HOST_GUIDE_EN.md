# Host Control Panel Guide (English)

This guide documents every button, control, and display element on the primary Host Control Panel (`/game`).

---

## 1. Top Status Bar & Controls

- **Game Title & Code**: Displays the active session's identifier (e.g. `G-2026-001`) and format badge (`Classic 90` or `Song Grid`).
- **Draw Progression Counter**: Displays the current position and total numbers in the pool (e.g. `12 / 90` or `7 / 45`).
- **Status Indicator**:
  - `ACTIVE` (Emerald chip): Game is live and ready to draw or receive claims.
  - `PAUSED` (Amber chip): Game is paused. Claims may still be evaluated, but number draws are locked until resumed.
  - `COMPLETED` (Blue chip): All numbers in the pool have been drawn.
- **Present Toggle (`P` / Button)**:
  - Toggles in-app presentation preview mode or launches/focuses the secondary presentation window.

---

## 2. Left Column: Media Stage & Answer Controls

- **Media Player Display**:
  - Plays the video or audio asset associated with the currently drawn number.
  - Automatically loads poster image or custom visual when audio is playing.
- **Show / Hide Answer Button (`A`)**:
  - Reveals or conceals the song title / artist name on the audience presentation screen.
  - Song titles always start **hidden** on each new number draw by default to facilitate guessing.
- **Show / Hide Number Button (`N`)**:
  - Toggles visibility of the numeric callout badge on the presentation screen.
- **Volume Slider**:
  - Adjusts media playback volume for the active session.

---

## 3. Middle Column: Flow Control & Draw Sequence

- **NEXT Button (Large Primary Button / `Space` / `→`)**:
  - Draws the next random number from the pre-generated Fisher-Yates draw sequence.
  - Advances draw position by 1, automatically resets song title to hidden, and triggers audio/video playback.
  - Safe against double-clicks (backend verifies `expected_position`).
- **Pause / Resume Button (`P`)**:
  - Atomically toggles between `ACTIVE` and `PAUSED` states.
- **End Game Button**:
  - Prompts for confirmation to terminate the active game session early.
- **Countdown Ticker**:
  - 10-second visual timer overlay for suspense between draws.
- **Compact Song Strip / Master Grid**:
  - 90-number (or 45-number) board showing all drawn numbers highlighted in call order.

---

## 4. Right Column: Verification & Side Activities

- **Quick Claim Desk (`C`)**:
  - **Scheme Selector**: Dropdown to select the winning scheme being claimed (e.g. Top Line, Early 5, Full House). Shows current winner count vs max capacity (e.g. `1/1` or `0/3`).
  - **Ticket Code Input**: Accepts full codes (e.g. `T-000042`) or numeric shorthand (e.g. `42`). Automatically converted to uppercase.
  - **Validate Claim Button**: Atomically checks qualification against the current draw position.
    - **ACCEPTED**: Awards winner slot, logs winning ticket, and announces prize on projector.
    - **REJECTED**: Displays specific reason (e.g. `LATE_CLAIM` with qualification draw position, `NOT_COMPLETED`, or `ALREADY_WON`).
- **Projector Control Card**:
  - Shows connected monitors and displays secondary screen open/closed status.
  - Buttons: **Open Projector**, **Close Projector**, and **Test Sound**.
- **Lucky Draw Launcher (`L`)**:
  - Quick access to autonomous lucky draw rounds for attendee names or ticket batches without interrupting the main draw.
- **Help Button (`?`)**:
  - Opens modal displaying all keyboard shortcuts.
