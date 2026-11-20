# Backend Error Codes & Parameters Contract

Authoritative specification for backend error codes returned by Tauri IPC commands and persisted in media validation audits.

All backend errors are serialized to the frontend in the standard envelope:
```json
{
  "code": "STRING_CODE",
  "params": {
    "key": "value"
  },
  "message": "English fallback message"
}
```

---

## 1. Media Management & Validation

| Error Code | Parameters | English Fallback Description |
|---|---|---|
| `MEDIA_FILE_NOT_FOUND` | — | Local media file was not found on disk. |
| `MEDIA_EXTENSION_NOT_ALLOWED` | `extension` | File extension is not permitted for the selected media type. |
| `MEDIA_TYPE_MISMATCH` | `expected`, `actual` | File contents do not match the expected media type (e.g. video bytes for image). |
| `MEDIA_REMOTE_HTML` | — | URL points to an HTML web page instead of a direct media file. |
| `MEDIA_REMOTE_TIMEOUT` | — | Remote media request timed out after 10 seconds. |
| `MEDIA_REMOTE_HTTP_STATUS` | `status` | Remote server returned an error HTTP status code. |
| `MEDIA_REMOTE_CONNECTION_FAILED` | — | Failed to establish network connection to the remote media host. |
| `MEDIA_UNSUPPORTED_FORMAT` | — | Unsupported media file format or unrecognized magic header bytes. |
| `MEDIA_SLOT_OUT_OF_RANGE` | `number` | Media slot number must be between 1 and 90. |
| `MEDIA_EMPTY_TITLE` | — | Media title cannot be empty. |
| `MEDIA_EMPTY_SOURCE` | — | Media source path or URL cannot be empty. |

---

## 2. Game Session & Controller

| Error Code | Parameters | English Fallback Description |
|---|---|---|
| `GAME_ALREADY_ACTIVE` | — | Another game session is already active or paused. |
| `GAME_NOT_FOUND` | `gameId` | Game session was not found. |
| `GAME_NOT_ACTIVE` | — | There is no active game session to perform this action. |
| `GAME_MEDIA_INVALID` | `validCount` | Cannot start game: fewer than 90 media items are validated (only {validCount} valid). |
| `GAME_NO_ACTIVE_SCHEMES` | — | Cannot start game: no active prize schemes configured. |
| `GAME_ALREADY_COMPLETED` | — | All numbers have already been drawn for this game session. |
| `BATCH_FORMAT_MISMATCH` | `expectedFormat`, `actualFormat` | Selected ticket batch format or song count does not match active game format. |
| `FORMAT_LOCKED_GAME_ACTIVE` | — | Cannot change game format while a game is active or paused. |
| `SONG_COUNT_INVALID` | `songCount` | Song count is invalid for the selected game format. |

---

## 3. Ticket Batches & Generation

| Error Code | Parameters | English Fallback Description |
|---|---|---|
| `BATCH_IN_USE` | `batchId` | Cannot delete ticket batch because it is assigned to an existing game or claim. |
| `BATCH_NOT_FOUND` | `batchId` | Ticket batch was not found. |
| `BATCH_COUNT_INVALID` | `count` | Ticket batch count must be between 1 and 10,000. |

---

## 4. Prize Schemes

| Error Code | Parameters | English Fallback Description |
|---|---|---|
| `SCHEME_NAME_DUPLICATE` | `name` | A prize scheme with this name already exists. |
| `SCHEME_NOT_FOUND` | `schemeId` | Prize scheme was not found. |
| `SCHEME_INVALID_CAPACITY` | `maxWinners` | Max winners capacity must be between 1 and 100. |

---

## 5. Application Settings

| Error Code | Parameters | English Fallback Description |
|---|---|---|
| `SETTING_NOT_ALLOWLISTED` | `key` | Setting key is not recognized or not allowlisted. |
| `SETTING_INVALID_VALUE` | `key`, `value` | Setting value is invalid for the specified configuration key. |

---

## 6. Claim Desk Verification (Spec §11)

| Rejection Code | Description |
|---|---|
| `NO_ACTIVE_GAME` | No game is currently active. |
| `GAME_NOT_ACCEPTING_CLAIMS` | Current game status does not permit claim submissions. |
| `TICKET_NOT_FOUND` | Ticket code does not exist in the database. |
| `TICKET_NOT_IN_GAME` | Ticket does not belong to the batch assigned to this game. |
| `SCHEME_NOT_IN_GAME` | Claimed prize scheme is not enabled for this game session. |
| `NOT_COMPLETED` | Ticket numbers do not satisfy the scheme pattern requirement. |
| `LATE_CLAIM` | Claim submitted too late (numbers drawn after qualification). |
| `ALREADY_WON_SCHEME` | This ticket has already won this specific prize scheme. |
| `ALREADY_WON_FULL_HOUSE` | Ticket has already won Full House and is precluded from further wins. |
| `ONLY_FULL_HOUSE_ELIGIBLE` | Ticket has won a non-Full-House prize and is now eligible only for Full House. |
| `WINNER_LIMIT_REACHED` | Maximum number of winners for this prize scheme has been reached. |

---

## 7. System & Infrastructure

| Error Code | English Fallback Description |
|---|---|
| `LOCK_ERROR` | Failed to acquire database lock. |
| `DATABASE_ERROR` | An internal database error occurred while executing the transaction. |
| `UNKNOWN_ERROR` | An unexpected error occurred. |
