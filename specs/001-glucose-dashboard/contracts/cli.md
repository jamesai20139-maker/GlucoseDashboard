# CLI Contract

## Audience and conventions

The CLI is for a non-developer local user. Commands use concise names, write normal
results to standard output, write actionable failures to standard error, and return a
non-zero exit code on failure. User-facing command messages are Traditional Chinese;
standard technical terms and command names remain unchanged.

## Commands

| Command | Purpose | Success behavior | Failure behavior |
|---|---|---|---|
| `glucose-dashboard` | Start the normal daily flow | Validates configuration, starts the local service, opens the browser | Reports setup, authentication, or service failure; non-zero exit |
| `glucose-dashboard start` | Explicit alias for normal start | Same as the default command | Same as the default command |
| `glucose-dashboard config` | Configure or replace the Google Sheet connection | Completes browser sign-in, validates Sheet access, saves non-sensitive settings | Does not save an incomplete configuration; non-zero exit |
| `glucose-dashboard doctor` | Run system checks | Reports login, Sheet, network, config, cache, and dashboard status | Reports failed checks and returns non-zero if required checks fail |
| `glucose-dashboard update` | Install a newer compatible release | Preserves configuration and reports the installed version | Leaves last usable installation intact; non-zero exit |
| `glucose-dashboard version` | Display installed version | Prints version and exits without opening the dashboard | Returns non-zero only if version metadata is unavailable |

## Exit behavior

- `0`: command completed successfully.
- Non-zero: command failed, was incomplete, or detected an unusable required dependency.
- `doctor` may report individual failed checks while still printing all check results.

## Security behavior

The CLI never prints OAuth secrets or writes them to standard output, standard error, or
plain-text configuration. Configuration output may show a redacted Sheet identifier and
the credential-store status.
