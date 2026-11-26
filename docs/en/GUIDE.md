# Ivily Envora — User guide (English)

Ivily Envora is a Windows desktop app for inspecting and safely changing environment variables, PATH,
profiles and project environments. The interface is available in Turkish (default) and English
(**Settings → General → Language**). The detailed Turkish guides are in [docs/rehber](../rehber/).

## Install

Download `Ivily Envora_<version>_x64-setup.exe` from
[Releases](https://github.com/efecrtgl/ivily-envora/releases) and run it. No administrator rights are needed;
the app installs for your account only (`%LOCALAPPDATA%\Ivily Envora`). Your data lives in
`%LOCALAPPDATA%\IvilyEnvora` and is kept when you uninstall. The installer is not code-signed yet, so
SmartScreen may warn; verify the file with `SHA256SUMS.txt` from the same release. In-app updates are signed
and are never installed without signature verification and your approval.

## Scopes

| Scope | Where | Persistent | Writing |
|-------|-------|-----------|---------|
| Process | A program's memory | No | Given only to processes launched from a project |
| User | `HKCU\Environment` | Yes | Standard rights |
| System | `HKLM\...\Session Manager\Environment` | Yes | Administrator (UAC, only for that change) |

Running programs keep their old environment after a persistent change — **restart them** to see new values.
If a name exists in both user and system scope, the user value wins; PATH is combined (system first).

## Every persistent change is safe by default

1. **Preview** — the affected scope, before/after values (sensitive values masked) and warnings.
2. **Confirm** — UAC is requested only here, and only for system-scope changes.
3. **Backup** — the previous state of the affected variables is saved as an automatic snapshot.
4. **Apply** — if a step fails, earlier steps are rolled back.
5. **Notify** — Windows is told the environment changed; a value-free activity record is written.

Any automatic backup can be restored from **Snapshots**.

## Secrets

Variables whose names or values look like tokens, passwords or keys are masked. Values are fetched only when
you click **Reveal** and are hidden again after a timeout. **Copy** writes the value to the clipboard from
the backend (excluded from clipboard history, cleared after a timeout). Secrets stored by the app (profiles,
projects, snapshots) are encrypted with Windows DPAPI. Exports exclude secrets unless you explicitly opt in.
DPAPI does not protect against malware running as your user, and Windows itself stores persistent
environment variables in plain text — see [GUVENLIK.md](../GUVENLIK.md) for the full threat model.

## Screens

- **Dashboard** — overview, PATH health, latest analysis, security, profiles, projects, tools, activity.
- **Variables** — search/filter all scopes; create, edit, duplicate, rename, delete; sensitivity overrides.
- **PATH** — flags missing/duplicate/relative/unresolved entries; draft-based editing; command conflicts.
- **Profiles** — reusable variable + PATH sets with version history, validation, apply and revert.
- **Projects** — link a directory to a profile and project variables; open a terminal or your own `.exe`
  with that environment, only after confirmation. Nothing in the project directory is ever executed.
- **Analysis** — 23 configurable rules explaining problems; nothing is fixed automatically.
- **Tools** — installed developer tools, versions and locations (Store aliases are never executed).
- **Templates** — secret-free starting points for profiles and projects.
- **Compare** — scopes, profiles, projects, snapshots or an imported file, side by side.
- **Snapshots** — manual and automatic backups; selective, previewed restore.
- **Import / export** — JSON, CSV, `.env`, PATH lists. Imported content is never executed or expanded.
- **Activity** — local, value-free log of operations.
- **Settings** — language, theme, dashboard, security, backup, activity retention, updates, Ivily ecosystem
  (IEP, off by default), database backup/restore, diagnostics report.

## Troubleshooting

See [SORUN-GIDERME.md](../SORUN-GIDERME.md). For bug reports, attach the **diagnostics report**
(Settings → About) — it contains no variable names or values.
