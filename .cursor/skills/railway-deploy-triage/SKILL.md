---
name: railway-deploy-triage
description: Diagnose failed backend deploys in GitHub Actions and Railway for this repo, then apply fixes, push, and verify reruns. Use when CI/deploy fails, Railway build logs show errors, or user asks to poll Railway/GitHub deploy logs.
---

# Railway Deploy Triage

Use this workflow for `synthwaves.fm.rust` backend deploy failures.

## Inputs Needed

- GitHub CLI auth (`gh auth status` succeeds)
- Railway auth:
  - preferred: `RAILWAY_TOKEN` exported in shell, or
  - fallback: `railway login` interactive session
- Repo root: `synthwaves.fm.rust`

## Fast Triage Checklist

1. Check git state and avoid committing local DB changes.
2. Identify latest failed runs in GitHub Actions.
3. Read failed logs to find the *first* failing step.
4. Fix code/workflow/config at source.
5. Run local verification for the changed surface.
6. Commit only relevant files and push.
7. Poll rerun status; if deploy still fails, inspect failed step logs and repeat.
8. If Railway auth fails (`Unauthorized`), stop and ask user to rotate/fix token permissions.

## Commands

### 1) Inspect failures

```bash
git status --short
gh run list --limit 10
gh run view <run_id> --log-failed
```

If the failed run is a specific job:

```bash
gh run view <run_id> --job <job_id> --log-failed
```

### 2) Railway CLI checks

```bash
railway --version
railway whoami
```

If using explicit token for one command:

```bash
RAILWAY_TOKEN="$RAILWAY_TOKEN" railway whoami
```

### 3) Validate deploy config assumptions

- Railway must build Rust image from `Dockerfile.rust` via `railway.json`.
- Deploy workflow should run Railway commands from repo root when using root `railway.json`.
- Do not rely on legacy Rails `Dockerfile` for Axum deploys.

### 4) Re-run/poll after pushing fixes

```bash
gh run list --limit 10
gh run watch <run_id> --interval 5 --exit-status
gh run view <run_id> --log-failed
```

### 5) Optional Railway logs

```bash
railway logs --follow
```

If this returns unauthorized, treat as token/account issue (not deploy code issue).

## Repo-Specific Failure Patterns

- `cargo fmt --check` failure blocks both CI and deploy workflows before Railway step.
- `Unauthorized` in `railway link` means invalid token or missing access to project/service/environment.
- `Gemfile.lock not found` in Railway build means wrong Dockerfile was used (Rails image path).

## Commit Guardrails

- Exclude `rust/storage/development.sqlite3` from commits unless user explicitly requests.
- Commit only files related to the fix.
- Push to `main` to trigger the configured deploy workflow.
