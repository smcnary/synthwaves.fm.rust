# Admin Parity Notes

Current Axum admin scaffolding:

- `GET /admin`
- `GET /admin/users`
- `GET /admin/jobs`

## Intended parity targets

- User browsing and management from Rails Madmin.
- Track/album/playlist moderation pages.
- Job visibility and retries (replacement for Mission Control Jobs).
- API key management surface.

## Authorization baseline

- Require authenticated admin user on all `/admin/*` routes.
- Preserve existing `authenticated? && admin?` semantics from Rails.
