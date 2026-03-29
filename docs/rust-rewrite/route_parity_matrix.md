# Route Parity Matrix (Rails -> Axum)

This matrix freezes the first-pass contract mapping for the big-bang rewrite.

## Web Routes

| Rails Route | Axum Route | Status |
| --- | --- | --- |
| `/` | `/` | Implemented scaffold |
| `/home` | `/home` | Implemented scaffold |
| `/music` | `/music` | Implemented scaffold |
| `/library` | `/library` | Implemented scaffold |
| `/search` | `/search` | Implemented scaffold |
| `/stats` | `/stats` | Implemented scaffold |

## JWT API (`/api/v1`)

| Rails Route | Axum Route | Status |
| --- | --- | --- |
| `POST /api/v1/auth/token` | `POST /api/v1/auth/token` | Implemented |
| `GET /api/v1/native/credentials` | `GET /api/v1/native/credentials` | Implemented |
| authenticated API base | `GET /api/v1/me` | Implemented example |

## Internal Radio API (`/api/internal`)

| Rails Route | Axum Route | Status |
| --- | --- | --- |
| `GET /api/internal/radio_stations/active` | same | Implemented |
| `GET /api/internal/radio_stations/:id/next_track` | same | Implemented |
| `POST /api/internal/radio_stations/:id/notify` | same | Implemented |

## Subsonic API (`/rest`, `/api/rest`)

| Rails Route Group | Axum Route | Status |
| --- | --- | --- |
| Subsonic endpoints | `GET /rest/ping` | Implemented first endpoint |
| Subsonic endpoints | `GET /api/rest/ping` | Implemented first endpoint |

## Admin

| Rails Route Group | Axum Route | Status |
| --- | --- | --- |
| `/admin` Madmin | `/admin` | Implemented scaffold |
| Madmin users | `/admin/users` | Implemented scaffold |
| Mission Control jobs page analogue | `/admin/jobs` | Implemented scaffold |
