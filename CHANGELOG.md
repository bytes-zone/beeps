# Changelog

## 0.5.0 (Unreleased)

- Shortcut listing is now available in the TUI (`?`)

## 0.4.3 (2025-01-05)

Log some helpful information on `beeps-server` startup to make operations easier.

This release is available as a container image at `ghcr.io/bytes-zone/beeps:v0.4.3`.

## 0.4.2 (2025-01-04)

Add a `/health` endpoint to the server for healthchecks in environments like Kubernetes.

This release is available as a container image at `ghcr.io/bytes-zone/beeps:v0.4.2`.

## 0.4.1 (2025-01-04)

Version updates for all packages. The server is now available as a container image now, too! You can find it at `ghcr.io/bytes-zone/beeps:v0.4.1`.

## 0.4.0 (2024-12-29)

`beeps-server` is now available. You will need to provide it with a Postgres database and some secret material for signing JWTs. Right now only email/password login is implemented (whose registration is gated by an `--allow-registration` flag), plus a "whoami" endpoint for debugging. The server is not yet integrated with the client, but that will be coming soon.

## 0.3.0 (2024-12-24)

- You can now copy/paste tags with the `c`/`v` keys.

## 0.2.0 (2024-12-21)

- You can now navigate through the table with the arrow keys.
- You can now delete tags with backspace or delete when a row is selected.
- New tags will now trigger a system notification.

## 0.1.0 (2024-12-20)

The first version of beeps. The main binary for now is `beeps`, which is a TUI that lets you know when you have a new ping and tag it. Everything is stored locally using CRDTs, and a sync server will be coming soon (as well as quality-of-life features like notifications.)
