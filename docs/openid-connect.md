# Configurable OpenID Connect login

Add entries to the optional `oidc` object in `config.json`, then restart the
server. The auth page always offers **Sign in with OpenID Connect**, linking
to `/auth/oidc`. That chooser lists the configured providers; if there are none,
it shows an unavailable message and a link back to the other sign-in methods.
Each configuration entry adds a provider button automatically. Existing
FusionAuth, GitHub, and Google login routes continue to work without
configuration changes. No database migration is needed.

For example, add this object alongside the existing configuration fields:

```json
"oidc": {
  "microsoft": {
    "display_name": "Microsoft",
    "issuer": "https://login.microsoftonline.com/YOUR-TENANT-ID/v2.0",
    "client_id": "YOUR-APPLICATION-ID",
    "client_secret_env": "HEARTS_MICROSOFT_CLIENT_SECRET",
    "client_auth_method": "post",
    "scopes": ["profile"]
  },
  "company": {
    "display_name": "Company account",
    "issuer": "https://login.example.com/realms/hearts",
    "client_id": "turbo-hearts",
    "client_secret_env": "HEARTS_COMPANY_CLIENT_SECRET"
  }
}
```

Register a web application with each provider. Enable the authorization-code
flow and PKCE with S256, and register this **exact callback URL**, substituting
your public `external_uri` and provider key:

```text
https://play.example.com/auth/oidc/microsoft/callback
https://play.example.com/auth/oidc/company/callback
```

Set the referenced environment variables in the server's environment before
starting it. Inline `client_secret` is also supported; use only one secret field.
Omit both for a provider that permits public clients with PKCE. Missing or empty
configured secrets are startup errors. Do not commit real secrets.

| Field | Meaning |
| --- | --- |
| Provider key (`microsoft`, `company`) | Stable identifier using 1–64 lowercase letters, digits, hyphens, or underscores. Forms the login/callback URLs and account namespace. |
| `display_name` | Text displayed on the login button; HTML is escaped. |
| `issuer` | Exact issuer URL advertised by the provider, including any tenant/realm path and trailing slash. This is not an authorization endpoint URL. |
| `client_id` | Registered application/client ID. |
| `client_secret_env` | Name of an environment variable containing the client secret. |
| `client_secret` | Inline alternative to `client_secret_env`. |
| `client_auth_method` | `basic` (default) or `post`, according to the provider's token endpoint requirements. |
| `scopes` | Optional additional scopes, default `["profile"]`. `openid` is always included. Add `email` only if your provider requires it; email never identifies the local account. |

Providers must expose OpenID Connect discovery and a JWKS signing-key endpoint.
Typical issuer shapes include:

- Keycloak: `https://login.example.com/realms/hearts`
- Auth0: `https://YOUR-TENANT.auth0.com/` (or the configured custom domain)
- Okta: the issuer of your configured authorization server
- Google: `https://accounts.google.com`
- GitLab.com: `https://gitlab.com`
- Microsoft Entra: `https://login.microsoftonline.com/YOUR-TENANT-ID/v2.0`

Use the provider's documented issuer and client registration requirements.
Microsoft's `common` and `organizations` endpoints advertise a tenant-dependent
issuer template; this implementation deliberately requires an exact issuer and
does not support those multi-tenant shortcuts. Apple private-key client
assertions and `form_post` callbacks are not supported. GitHub and Discord's
ordinary OAuth APIs are not interchangeable with OpenID Connect discovery.

## Login behavior

The server discovers endpoints and keys when login starts and again at callback,
so provider outages do not prevent startup and key rotation needs no restart.
Back-channel requests have a timeout, a response-size limit, and do not follow
redirects. Production URLs require HTTPS. HTTP is accepted only for loopback
hosts (`localhost`, `127.0.0.1`, or `::1`) during local development.

Each login uses a random state, nonce, and PKCE verifier. The transaction is
bound to an HttpOnly, SameSite=Lax browser cookie, expires after ten minutes,
and can be consumed once. Production transaction cookies also use Secure.
The server validates ID-token signatures, issuer, audience, authorized party,
expiry, issue time (allowing 60 seconds of clock skew), nonce, and the access-token
hash when provided. If the ID token lacks a
display name, the optional UserInfo endpoint is queried and its subject must
match the ID token. Provider errors and invalid callbacks do not create sessions.

The user identity is the provider key, exact issuer, and verified `sub` claim.
Names, usernames, and email addresses are display data, not identity keys.
Keep provider keys and issuers stable: changing either creates a separate
account namespace. A Google account through a new OIDC entry is also separate
from an existing legacy Google login; automatic account linking is not included.

The resulting login uses the application's existing session cookies and user
storage. The existing frontend clears session cookies itself on a failed SSE
connection, so those session cookies remain browser-readable. Only the OIDC
transaction cookie is HttpOnly. Display names are cookie-encoded, including
Unicode and punctuation.

Pending transactions live in memory. A restart invalidates in-progress logins;
users can start again. Multiple server instances require sticky routing for
both the start and callback requests, or a future shared transaction store.
Only GET callbacks with authorization codes are supported. Token refresh and
provider logout are outside this login-only integration.

## Verification

```sh
cargo test --locked -p turbo-hearts auth_endpoints::oidc
cargo test --locked --workspace --all-targets
```

The tests use an in-process discovery/token/UserInfo provider with signed ID
tokens and an isolated SQLite database. They exercise successful login, account
reuse, PKCE, key rotation, client authentication, rejected tokens, state/browser
binding, replay, expiry, and denied consent. No real provider credentials are
needed. Before enabling a real provider for users, complete a login using its
registered client and deployed callback URL.
