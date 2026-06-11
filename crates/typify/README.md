# Vendored typify fork

Vendored from https://github.com/romnn/typify branch `feat/improve-support`
at commit aa4517e734406e942c894083da6d664db3850156, itself a fork of
https://github.com/oxidecomputer/typify (Apache-2.0, see LICENSE).

Vendored so progenitor owns its whole code-generation stack: fixes that
span the OpenAPI frontends, the internal model, and type generation land
as one commit gated by one test suite (including the wild-spec corpus in
`crates/wild-tests`). Upstream typify announced it will replace its
schema model entirely (oxidecomputer/typify#886), so tracking it as a git
dependency had a shrinking future anyway.
