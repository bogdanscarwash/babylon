# ADR094 open gates — owner-run keygen (D1) and hypergraph-rs adoption (D6)

Two ADR094 obligations are deliberately NOT executed by the implementation plan. They are
recorded here so their status is explicit and nobody fakes them.

## D1 — cache-signing keypair (OWNER-RUN, pending)

The ed25519 keypair that signs narinfo files is the integrity boundary (TLS is transport,
not trust). The SECRET half must exist only in CI; the PUBLIC half is baked into
`install.sh` as `CACHE_KEY`. Minting it is owner-run — the plan ships `install.sh` with a
placeholder that HARD-REFUSES to run until replaced.

Owner command (in babylon-infra, refuses to overwrite an existing key; writes the secret to
`$XDG_DATA_HOME/babylon-infra/keys/`; prints the public key and the `gh secret set`
next-step):

```sh
cloudflare/scripts/generate-cache-key.sh
```

After running it, the owner:

1. Replaces `CACHE_KEY="babylon-cache-1:REPLACE_WITH_PUBLIC_KEY"` in `install.sh` with the
   printed public key.
2. Registers the secret half in the game repo:
   `gh secret set NIX_CACHE_SIGNING_KEY` (consumed by `nix-release.yml`).

Until both steps are done, `nix-release.yml` builds and passes the regression gate but has
no key to sign with, and `install.sh` refuses to install.

## D6 — hypergraph-rs as a pinned flake input (BLOCKED on port parity)

`hypergraph-rs` is a gitignored local subrepo with its own git history and no flake of its
own — invisible to flake builds as a subtree of `self`. Adoption checklist (implement none
of this until the gate opens):

- Enter the game flake as a pinned INPUT with `inputs.nixpkgs.follows = "nixpkgs"` — never a
  subtree of `self`.
- A version bump is an input-hash change that rides the SAME monthly closure regression gate
  as nixpkgs (Task 4 / D5).
- **Adoption is gated on the PR #210 port-parity test suite passing.** As of 2026-07-20 that
  suite does not exist (PR #210 was docs-only). Do not add the input until the parity suite
  exists and is green.
