# Tambola Rules & Claim Validation Engine — Media Tambola

## 1. Ticket Generation Rules (90-Ball Tambola)
A standard 90-ball Tambola ticket conforms to the following strict mathematical structure:
- **Grid Size**: Exactly 3 rows and 9 columns (27 cells total).
- **Number Count**: Exactly 15 numbers per ticket (12 blank cells).
- **Row Distribution**: Exactly 5 numbers and 4 blank cells per row.
- **Column Ranges**:
  - Column 0: 1–9
  - Column 1: 10–19
  - Column 2: 20–29
  - Column 3: 30–39
  - Column 4: 40–49
  - Column 5: 50–59
  - Column 6: 60–69
  - Column 7: 70–79
  - Column 8: 80–90
- **Column Ordering**: Within any column, numbers must appear in strictly ascending order from top to bottom.
- **Column Occupancy**: Every column must have between 1 and 3 numbers, summing to exactly 15 across the 9 columns.
- **Uniqueness**: Every ticket must have a globally unique canonical layout hash (a deterministic SHA-256 representation of cell positions and numbers).

### 1.1 Generation Algorithm & Layout Hashing
- **Unbiased Partition Selection**: The 15 numbers are partitioned into 9 column counts $c_0..c_8 \in \{1, 2, 3\}$ by initializing each column to 1 and randomly incrementing columns (up to max 3) until the sum is 15. Every valid column count combination is reachable without bias.
- **Randomized Backtracking Row Assignment**: For each column $c$ with count $k$, candidate row subsets of size $k$ are shuffled uniformly at random using `RandomSource`. Backtracking recursively explores row assignments, pruning any branch where a row exceeds 5 or cannot reach 5 with remaining columns. This ensures that every column has positive probability of placing numbers in Row 0, Row 1, and Row 2 (specifically allowing singletons in Row 2).
- **Number Allocation**: For each column $c$, $k \in \{1, 2, 3\}$ numbers are drawn without replacement from the column range $min_c..=max_c$ using `RandomSource`. Numbers are sorted ascending and mapped top-to-bottom to the assigned rows.
- **Canonical Representation**: String representation formatted row-major with `|` column delimiters and newline row delimiters (e.g., `10|_|25|_|42|_|61|_|88\n...`).
- **Layout Hash**: Lowercase 64-character SHA-256 hex digest of the canonical string, enforced via SQLite `UNIQUE(layout_hash)`.
- **Statistical Invariants**: Over large sample sets (e.g. 20,000 tickets):
  - Every column count 1, 2, and 3 occurs for every column ($> 0$).
  - For every column, the singleton case ($count = 1$) located in Row 2 occurs ($> 0$).

## 2. Winning Schemes Architecture & Rules
Media Tambola provides 6 standard data-driven schemes implemented via pure, stateless evaluators (`SchemeEvaluator` trait in `src-tauri/src/domain/schemes/mod.rs`):

### 2.1 Scheme Types and Position Formulas
1. **EARLY_FIVE**:
   - **Condition**: Any 5 numbers on the ticket have been called.
   - **Position Formula**: All called numbers on the ticket are gathered with their draw positions in `call_position_by_number`. If fewer than 5 have been called, the ticket is not qualified. When 5 or more are called, their positions are sorted in ascending order; the 5th smallest position is the qualification position ($P_4$ in 0-indexed order).
   - **Full House Flag**: `false`.

2. **FOUR_CORNERS**:
   - **Condition**: The leftmost and rightmost occupied cells of Row 0 (top line) and Row 2 (bottom line).
   - **Convention**: In standard 90-ball Tambola tickets with 3 rows of 5 numbers across 9 columns, "corners" are defined as the first and fifth numbers in Row 0 (the leftmost and rightmost occupied cells of the top line) and the first and fifth numbers in Row 2 (the leftmost and rightmost occupied cells of the bottom line). This convention is isolated within `FourCornersEvaluator`.
   - **Position Formula**: The maximum call position of these 4 corner numbers:
     $$\text{position} = \max_{n \in \text{corners}} (\text{call\_position}[n])$$
   - **Full House Flag**: `false`.

3. **TOP_LINE**:
   - **Condition**: All 5 numbers located in Row 0 have been called.
   - **Position Formula**: $\text{position} = \max_{n \in \text{row}_0} (\text{call\_position}[n])$.
   - **Full House Flag**: `false`.

4. **MIDDLE_LINE**:
   - **Condition**: All 5 numbers located in Row 1 have been called.
   - **Position Formula**: $\text{position} = \max_{n \in \text{row}_1} (\text{call\_position}[n])$.
   - **Full House Flag**: `false`.

5. **BOTTOM_LINE**:
   - **Condition**: All 5 numbers located in Row 2 have been called.
   - **Position Formula**: $\text{position} = \max_{n \in \text{row}_2} (\text{call\_position}[n])$.
   - **Full House Flag**: `false`.

6. **FULL_HOUSE**:
   - **Condition**: All 15 numbers across the entire ticket have been called.
   - **Position Formula**: $\text{position} = \max_{n \in \text{all\_15\_numbers}} (\text{call\_position}[n])$.
   - **Full House Flag**: `true` (authoritatively determined by `scheme_type == SchemeType::FullHouse`, never by string name).

### 2.2 Chronological Draw Order Invariant
- Under no circumstances are number magnitudes compared or evaluated for qualification order.
- The 1-based chronological draw sequence recorded in `call_position_by_number` is authoritative. For instance, if corners are [12, 63, 55, 90] and numbers are called in order [12, 55, 90, 63], the qualification position is the call position of 63, regardless of the numerical values.

## 3. Claim Evaluation & Late Claim Rule
Qualification and validation are evaluated authoritatively by the Rust backend.

### 3.1 Draw Position Map & 1-Based Indexing
- Draw positions are strictly **1-based** integers (`position 1` = the first number called, `position 2` = second number called, up to `position 90` maximum).
- Each draw position holds exactly one called number.
- `callPositionByNumber: HashMap<u8, u32>` maps each drawn number to its 1-based sequential call position in the active game. Uncalled numbers are absent from this map.

### 3.2 Qualification Position Calculation
- **Lines / Corners / Full House**:
  $$qualification\_position = \max_{n \in required\_numbers} (callPositionByNumber[n])$$
- **Early Five**:
  Let $P = \{ callPositionByNumber[n] \mid n \in ticket\_numbers \text{ and } n \text{ has been drawn} \}$.
  If $|P| \ge 5$, $qualification\_position$ is the 5th smallest value in $P$.

### 3.3 Strict Late Claim Rule
A claim is submitted when the game is at `current_draw_position`.
- If $qualification\_position == current\_draw\_position$:
  The claim is **ON TIME** and eligible for acceptance.
- If $qualification\_position < current\_draw\_position$:
  The claim is **LATE** and MUST be rejected with:
  `"Late claim is not allowed."`
- If the ticket has not fulfilled the requirements:
  The claim MUST be rejected with:
  `"Ticket has not completed this winning scheme."`

### 3.4 Authoritative Claim Check Order (Spec §9, §10, §11, §12)
Claim validation is executed by a pure decision function (`src-tauri/src/domain/claims/claim_validation.rs`) following this strict priority sequence:

1. **Game Status Guard**: If no game session exists -> `NO_ACTIVE_GAME`. If game status is not `ACTIVE` or `PAUSED` (e.g. `COMPLETED` or `ENDED`) -> `GAME_NOT_ACCEPTING_CLAIMS`. (A `PAUSED` game still accepts claims).
2. **Ticket Identification Guard**: If ticket does not exist -> `TICKET_NOT_FOUND`. If ticket is not mapped to this game in `game_tickets` -> `TICKET_NOT_IN_GAME`.
3. **Scheme Snapshot Guard**: If scheme was not active in this game's snapshot -> `SCHEME_NOT_IN_GAME`.
4. **Qualification Evaluation**: Evaluated via `SchemeRegistry`. If condition is not met -> `NOT_COMPLETED` ("Ticket has not completed this winning scheme.").
5. **Late Claim Check**:
   - If $qualification\_position < current\_draw\_position$ -> `LATE_CLAIM` ("Late claim is not allowed."), returning `qualification_position`.
   - Accept only when $qualification\_position == current\_draw\_position$. (Future qualification is treated as not completed).
6. **Duplicate Scheme Win Guard**: If ticket has already won this specific scheme in this game -> `ALREADY_WON_SCHEME`.
7. **Full House Preclusion Guard**: If ticket has already won `FULL_HOUSE` in this game -> `ALREADY_WON_FULL_HOUSE`.
8. **Single Non-Full-House Prize Guard**: If the claimed scheme is not Full House and the ticket has already won any non-Full-House scheme -> `ONLY_FULL_HOUSE_ELIGIBLE` ("This ticket has already won a non-Full-House scheme and is only eligible for Full House.").
9. **Capacity Limit**: If accepted winner count for this scheme $\ge max\_winners$ -> `WINNER_LIMIT_REACHED`.
10. **Acceptance**: Otherwise `ACCEPTED` with $winner\_position = accepted\_count + 1$ and $qualification\_position$.

### 3.5 Ticket Eligibility Matrix
| Scenario | Previous Wins on Ticket | Claimed Scheme | Result | Reason Code |
|---|---|---|---|---|
| 1 | None | Early Five | ACCEPTED | Valid 1st non-FH win |
| 2 | Early Five | Top Line | REJECTED | `ONLY_FULL_HOUSE_ELIGIBLE` |
| 3 | Early Five | Full House | ACCEPTED | Full House is permitted after 1 non-FH win |
| 4 | Full House | Any Scheme | REJECTED | `ALREADY_WON_FULL_HOUSE` |
| 5 | Any Win | Same Scheme | REJECTED | `ALREADY_WON_SCHEME` |
| 6 | Past Draw Position | Any Scheme | REJECTED | `LATE_CLAIM` |
| 7 | Fresh Game Session | Any Scheme | Fresh Eligibility | Per-game scope; historical winners preserved |

## 4. Game Controller & Draw Engine Lifecycle
Game sessions operate as an authoritative finite state machine managed in SQLite transactions:

### 4.1 State Machine
- **States**: `READY`, `ACTIVE`, `PAUSED`, `COMPLETED`, `ENDED`.
- **Concurrency Guard**: A partial unique index (`idx_single_active_or_paused_game`) prevents starting a game if another game is `ACTIVE` or `PAUSED`.
- **Transitions**:
  - `create_game(batch_id)`: Generates crypto draw sequence (Fisher-Yates 1..=90), snapshots all 90 media assets and active schemes, draws position 1 immediately, and enters `ACTIVE`.
  - `draw_next(game_id)`: Advances position by 1 ($pos \le 90$). At position 90, status becomes `COMPLETED` with `ended_at`. Double-draw calls are prevented via transaction locks and unique constraints.
  - `pause_game(game_id)`: Transitions `ACTIVE` -> `PAUSED`. Draw calls while paused are rejected.
  - `resume_game(game_id)`: Transitions `PAUSED` -> `ACTIVE`.
  - `end_game(game_id)`: Transitions `ACTIVE` | `PAUSED` -> `ENDED` with `ended_at`.

### 4.2 Snapshot Immutability & Protocol Serving
- At game inception, all 90 media assets and active schemes are copied to `game_media_snapshots` and `game_schemes`.
- Local snapshot media is served via `http://media.localhost/game/<gameId>/<number>` and `/thumb`.
- Master table modifications never alter games in progress.

### 4.3 Deterministic Crash Recovery
- `get_active_game` recovers the exact session state from database records without regenerating or altering the draw sequence.
