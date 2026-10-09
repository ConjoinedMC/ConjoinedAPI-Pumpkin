# Protocol v1

HTTPS requests use `Authorization: Bearer <token>` and binary bodies.
The token is authorized for exactly one namespace. Requests do not depend on connected players.

`/v1/{namespace}/{owner}/{key}` supports GET, PUT, and DELETE.
`owner` is `global` for non-player data or a canonical lowercase player UUID.
Namespaces use `[a-z0-9_-]{1,64}`; keys use `[a-zA-Z0-9_.-]{1,128}`.
Values may contain at most 65,536 bytes, including empty values.

| Operation | Success | Missing key |
| --- | --- | --- |
| GET | 200 with bytes | 404 |
| PUT | 204 after durable upsert | Creates key |
| DELETE | 204 | 204 |

Other responses: 400 invalid input, 401 unauthorized, 405 unsupported method,
413 oversized body, 500 internal failure. Error responses contain no sensitive details.
Clients verify TLS certificates and do not follow redirects. Generate tokens from at least
32 random bytes encoded as unpadded base64url. Never reuse a token across namespaces.

Writes replace entire values. There is no compare-and-swap, multi-key transaction, listing,
automatic retry, or per-player access control. A namespace credential can access all players
in that namespace. Do not implement balances using a read followed by a write when concurrent
updates are possible. A timed-out write has an unknown outcome; reconcile before retrying.
