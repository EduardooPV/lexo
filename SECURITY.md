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

Lexo sends the API key you provide in Settings only to the DeepL API. It stores
the key in Windows Credential Manager, macOS Keychain, or a Linux Secret Service
credential store. When the credential store is unavailable, Lexo can use
`settings.json` in your app-config directory and shows a warning when saving the
key there.

Issues with API key handling, the app's Content Security Policy, or the
selection and OCR flows that simulate keyboard input are in scope.
