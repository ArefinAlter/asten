# 0.0.1c development increment

27 October 2026. Internal version `0.0.1-beta.1`. Full-release packaging and acceptance are recorded in [ACCEPTANCE-0.0.1c.md](ACCEPTANCE-0.0.1c.md).

## Existing agent subscriptions

Providers includes Codex, Claude Code, Cursor and Antigravity installed-agent connections alongside API-key providers. Sign in using the owning CLI; the product does not extract IDE tokens or browser credentials. Claude and Codex subscription authentication is checked through their own status command. Cursor and Antigravity rely on their own cached CLI sign-in and model entitlement. User consent and offline controls still apply because a locally started agent may use a cloud model.

Requests use stdin and a fresh temporary working folder, bounded output and deadlines. Windows descendants are supervised with kill-on-close jobs. The adapter accepts final responses only and validates requested product tools against the declared table. Claude built-in tools are disabled; Codex shell and web tools are disabled in read-only mode. Cursor Ask runs with an app-created deny policy; Antigravity Plan is refused unless the owning CLI has toolPermission=strict. These are provider controls, not an OS isolation claim. CLI tools/configuration behavior for those two has protocol coverage but no live account acceptance on this machine.

Live acceptance: the common Windows transport completed a bounded Claude subscription request returning the expected constant, without project files. Codex is installed but the subscription sign-in check did not pass; no Codex inference was claimed. Grok has no verified direct consumer-account adapter. A Grok model may be selected through Cursor if that account exposes it. See [installed agents](INSTALLED-AGENTS.md).

Subscription prices are not API prices. Provider quotas, extra-usage billing and model availability remain controlled by the provider; app token accounting uses reported usage when available. No silent fallback from an installed-agent error to an API-key provider occurs.

## Asten

Four disabled Cloud provider presets use existing installed-agent sign-ins and require the same notice consent. Key entry is hidden for them. Offline mode blocks those cloud connections. Existing read-only tools, prose refusal and rule-based canon behavior remain. The pre-alpha runtime image also sets executable/read permissions explicitly so a build under restrictive umask can start.

Validation: assistant Clippy with warnings denied; 56 unit tests and 28 integration tests pass, with one ignored subprocess fixture exercised by its parent test. Frontend type checking, 49 tests and hosted production build pass. No live Cursor/Antigravity account call or new installer acceptance is claimed.
