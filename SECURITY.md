# Security Policy

## Supported Versions

Only the latest release is supported. Since the app auto-updates its version on every
release, there is no older version receiving fixes — please update before reporting an
issue.

## Reporting a Vulnerability

Please **do not** open a public issue for a security vulnerability.

Instead, use GitHub's private reporting: go to the **Security** tab of this repository
→ **Report a vulnerability**. This opens a private advisory visible only to you and the
maintainers until it's resolved.

Include:

- A description of the vulnerability and its potential impact
- Steps to reproduce it
- The affected version/commit, if known

## Scope

Lexo talks to exactly one external service — the DeepL API, using the key you provide
in Settings. That key is stored locally in `settings.json` under your OS's app-config
directory and is never sent anywhere except the DeepL API itself. Reports related to
that key's handling, the app's Content-Security-Policy, or the selection/OCR flows
(which simulate keyboard input) are all in scope.
