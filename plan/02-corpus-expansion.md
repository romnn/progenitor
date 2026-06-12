# 02 — Corpus expansion: 60 → 120 specs (30 new 3.0.x + 30 new 3.1.x)

Current corpus (today still `crates/wild-tests/manifest.toml`): exactly
30 × 3.0 and 30 × 3.1, all passing. Target: 60 × 3.0 and 60 × 3.1, each one a
`conformance/<name>/` crate (per 05), all green under
`cargo check --workspace`, with behavioral tests in each crate's lib.rs for
every novel edge case the new specs surface.

Every candidate below was **verified by downloading the document and reading
its exact `openapi:` version string** (2026-06-12). APIs.guru turned out to be
nearly useless for 3.1 (35 of 2529 indexed APIs, of which 31 are Adyen/Codat —
vendors we already cover), so the 3.1 list is built from direct vendor probes
(~210 URLs tried). Sizes are real downloaded KB.

## 1. New 3.1.x specs (30)

| name | ver | fmt | KB | why this one |
|---|---|---|---|---|
| telnyx | 3.1.0 | json | 4967 | slow; webhooks block, 36 discriminators, 79 oneOf, multipart, octet-stream; near-differential 3.0 pair exists (alternates) |
| posthog | 3.1.0 | yaml | 4884 | slow; drf-spectacular live endpoint; **925 patternProperties**, 3743 anyOf, 198 const, prefixItems |
| increase | 3.1.0 | json | 3470 | webhooks, 1557 anyOf, null-heavy; openapi key not in first 128 KB; Stainless hash-URL rotates |
| webflow | 3.1.0 | yaml | 3318 | webhooks block, official Data API v2 |
| opensearch | 3.1.0 | yaml | 2719 | 256 oneOf, 183 const; URL is a release-asset alias that 302-redirects |
| orb | 3.1.0 | json | 1709 | 2955 oneOf lines, 19 discriminators, octet-stream, null-heavy; Stainless rotating URL |
| workos | **3.1.1** | yaml | 1171 | rare 3.1.1 version string; 623 const |
| lithic | 3.1.0 | yaml | 1048 | card-issuing fintech; 99 oneOf, 86 const, patternProperties; Stainless rotating URL |
| logfire | 3.1.0 | json | 933 | FastAPI live; prefixItems, const, multipart, discriminator |
| langsmith | 3.1.0 | json | 893 | minified single-line FastAPI; webhooks, prefixItems, patternProperties |
| lago | 3.1.0 | yaml | 830 | webhooks block; OSS billing |
| openrouter | 3.1.0 | json | 741 | patternProperties, const, multipart, octet-stream |
| runway | 3.1.0 | json | 691 | **54 discriminators, 163 const** — discriminated-union showcase; Stainless rotating URL |
| airflow | 3.1.0 | yaml | 505 | Apache Airflow 3 FastAPI; 639 anyOf, prefixItems, 626 null-type lines |
| netbird | 3.1.0 | yaml | 384 | WireGuard mesh-VPN management API |
| scaleway-instance | 3.1.0 | yaml | 278 | EU cloud; ~30 sibling product schemas if more needed later |
| finch | 3.1.0 | json | 270 | HR/payroll; discriminators, null types; Stainless rotating URL |
| pokeapi | 3.1.0 | yaml | 267 | drf-spectacular; highly recognizable |
| deepinfra | 3.1.0 | json | 246 | live FastAPI; prefixItems, const, patternProperties |
| lambda | 3.1.0 | json | 223 | GPU cloud; const, anyOf, null types |
| xai | 3.1.0 | json | 203 | patternProperties, multipart, octet-stream |
| gladia | 3.1.0 | json | 121 | webhooks, multipart, discriminator; speech-to-text FastAPI |
| weather-gov | **3.1.2** | json | 117 | rare 3.1.2; **requires User-Agent header** (harness change, §4) |
| jina | 3.1.0 | json | 100 | minified live FastAPI |
| chroma | 3.1.0 | json | 79 | vector-DB cloud API |
| cratesio | 3.1.0 | json | 70 | **utoipa (Rust)-generated — dogfood value** |
| bfl | 3.1.0 | json | 58 | Black Forest Labs FLUX; HEAD 405, GET fine |
| unstructured | 3.1.0 | json | 14 | const, binary in a tiny file |
| frankfurter | **3.1.2** | json | 10 | rare 3.1.2; tiny smoke-test size |
| urlbox | 3.1.0 | yaml | 8 | `.well-known/open-api.yaml` path; smallest 3.1 |

URLs (verbatim, verified):

```
telnyx            https://raw.githubusercontent.com/team-telnyx/openapi/master/openapi/spec3.json
posthog           https://app.posthog.com/api/schema/
increase          https://storage.googleapis.com/stainless-sdk-openapi-specs/increase/increase-beca94b958f84df1e88f5e26405a66965fbf81a0c0c33b5201f0ec8d06e13ff0.yml
webflow           https://raw.githubusercontent.com/webflow/openapi-spec/main/openapi/v2.yml
opensearch        https://github.com/opensearch-project/opensearch-api-specification/releases/download/main-latest/opensearch-openapi.yaml
orb               https://storage.googleapis.com/stainless-sdk-openapi-specs/orb%2Forb-c92fb451e13f157b3735f188acc8d57aa3adfbaac1683645e1ba4f432dd7a4f8.yml
workos            https://raw.githubusercontent.com/workos/openapi-spec/main/spec/open-api-spec.yaml
lithic            https://storage.googleapis.com/stainless-sdk-openapi-specs/lithic/lithic-efe780032e44b3cf0f6914407e43bce6aa7176fa50aa6ec018f93c1f28af8490.yml
logfire           https://logfire-api.pydantic.dev/openapi.json
langsmith         https://api.smith.langchain.com/openapi.json
lago              https://raw.githubusercontent.com/getlago/lago-openapi/main/openapi.yaml
openrouter        https://openrouter.ai/docs/openapi.json
runway            https://storage.googleapis.com/stainless-sdk-openapi-specs/runwayml/runwayml-e4103e353904891c1dfe59c4cfaefb536c0dc20ae0316f2c18bae77aa4e8d8c7.yml
airflow           https://raw.githubusercontent.com/apache/airflow/main/airflow-core/src/airflow/api_fastapi/core_api/openapi/v2-rest-api-generated.yaml
netbird           https://raw.githubusercontent.com/netbirdio/netbird/main/shared/management/http/api/openapi.yml
scaleway-instance https://www.scaleway.com/en/developers/api/instance/v1/schema.yml
finch             https://storage.googleapis.com/stainless-sdk-openapi-specs/finch%2Ffinch-46f433f34d440aa1dfcc48cc8d822c598571b68be2f723ec99e1b4fba6c13b1e.yml
pokeapi           https://raw.githubusercontent.com/PokeAPI/pokeapi/master/openapi.yml
deepinfra         https://api.deepinfra.com/openapi.json
lambda            https://cloud.lambdalabs.com/api/v1/openapi.json
xai               https://docs.x.ai/openapi.json
gladia            https://api.gladia.io/openapi.json
weather-gov       https://api.weather.gov/openapi.json
jina              https://api.jina.ai/openapi.json
chroma            https://api.trychroma.com/openapi.json
cratesio          https://crates.io/api/openapi.json
bfl               https://api.bfl.ai/openapi.json
unstructured      https://api.unstructuredapp.io/general/openapi.json
frankfurter       https://api.frankfurter.dev/v1/openapi.json
urlbox            https://urlbox.com/.well-known/open-api.yaml
```

## 2. New 3.0.x specs (30)

| name | ver | fmt | KB | why this one |
|---|---|---|---|---|
| zoom | 3.0.0 | json | 5980 | slow; 53 oneOf, heavy multipart; APIs.guru permalink (vendor spec is login-gated) |
| sentry | 3.0.3 | json | 3452 | official derefed artifact, auto-updated; 89 anyOf, patternProperties |
| okta | 3.0.3 | yaml | 3392 | **56 discriminators** — discriminator-heavy big YAML |
| launchdarkly | 3.0.3 | json | 2772 | live vendor endpoint (APIs.guru copy is stale Swagger 2.0) |
| klaviyo | 3.0.2 | json | 2719 | 180 oneOf; official stable.json |
| stytch | 3.0.3 | yaml | 2466 | big plain YAML — bulk stress with few composition keywords |
| jellyfin | 3.0.1 | json | 2035 | **77 `format: binary` + octet-stream — best binary-body exerciser found**; .NET-generated |
| aiven | 3.0.1 | json | 1761 | openapi key NOT in first 128 KB (parser-order stress) |
| vapi | 3.0.0 | json | 1757 | NestJS minified live endpoint, extensionless URL |
| zendesk | 3.0.3 | yaml | 1584 | 34 oneOf, octet-stream, multipart |
| novu | 3.0.0 | json | 1381 | NestJS minified live |
| superset | 3.0.2 | json | 1243 | flask-appbuilder; openapi key not in first 128 KB |
| polygon | 3.0.3 | json | 1132 | extensionless live URL; HEAD 405 |
| miro | 3.0.1 | json | 1041 | patternProperties, multipart, discriminator |
| binance | 3.0.2 | yaml | 976 | quoted version string `'3.0.2'`; huge enum/array responses |
| oxide | 3.0.3 | json | 923 | **progenitor's flagship consumer spec (Oxide Rack) — regression-protection value**; 89 oneOf |
| twitter | 3.0.0 | json | 788 | X API v2; unusual `"openapi" : "3.0.0"` spacing; HEAD 405 |
| intercom | 3.0.1 | yaml | 842 | versioned descriptions dirs; multipart |
| exoscale | 3.0.0 | yaml | 712 | chunked transfer (no content-length) |
| airbyte | 3.0.0 | yaml | 680 | config API; discriminator, binary |
| neon | 3.0.3 | json | 674 | vendor release alias (neon.tech 301s to neon.com) |
| circleci | 3.0.3 | json | 631 | **HEAD 404, GET 200** |
| influxdb | 3.0.0 | yaml | 626 | openapi key deep in YAML; multipart, binary |
| clerk | 3.0.3 | yaml | 607 | dated-filename spec; discriminators, patternProperties |
| dub | 3.0.3 | json | 523 | Speakeasy registry permalink; x-codeSamples-laden |
| spotify | 3.0.3 | yaml | 279 | community-fixed official spec (sonallux) |
| deepl | 3.0.3 | yaml | 260 | official; multipart, patternProperties |
| nomad | 3.0.3 | yaml | 252 | starts with a license comment block (key not near byte 0) |
| pinecone | 3.0.3 | yaml | 117 | official date-stamped spec repo |
| hubspot-contacts | 3.0.1 | json | 64 | unusual `"openapi" : "3.0.1"` spacing; release-pinned URL |

URLs (verbatim, verified):

```
zoom              https://api.apis.guru/v2/specs/zoom.us/2.0.0/openapi.json
sentry            https://raw.githubusercontent.com/getsentry/sentry-api-schema/main/openapi-derefed.json
okta              https://raw.githubusercontent.com/okta/okta-management-openapi-spec/master/dist/current/management-minimal.yaml
launchdarkly      https://app.launchdarkly.com/api/v2/openapi.json
klaviyo           https://raw.githubusercontent.com/klaviyo/openapi/main/openapi/stable.json
stytch            https://raw.githubusercontent.com/stytchauth/stytch-openapi/main/openapi.yml
jellyfin          https://api.jellyfin.org/openapi/jellyfin-openapi-stable.json
aiven             https://api.aiven.io/doc/openapi.json
vapi              https://api.vapi.ai/api-json
zendesk           https://developer.zendesk.com/zendesk/oas.yaml
novu              https://api.novu.co/openapi.json
superset          https://raw.githubusercontent.com/apache/superset/master/docs/static/resources/openapi.json
polygon           https://api.polygon.io/openapi
miro              https://raw.githubusercontent.com/miroapp/api-clients/main/packages/generator/spec.json
binance           https://raw.githubusercontent.com/binance/binance-api-swagger/master/spot_api.yaml
oxide             https://raw.githubusercontent.com/oxidecomputer/oxide.rs/main/oxide.json
twitter           https://api.twitter.com/2/openapi.json
intercom          https://raw.githubusercontent.com/intercom/Intercom-OpenAPI/main/descriptions/2.14/api.intercom.io.yaml
exoscale          https://openapi-v2.exoscale.com/source.yaml
airbyte           https://raw.githubusercontent.com/airbytehq/airbyte-platform/main/airbyte-api/server-api/src/main/openapi/config.yaml
neon              https://neon.com/api_spec/release/v2.json
circleci          https://circleci.com/api/v2/openapi.json
influxdb          https://raw.githubusercontent.com/influxdata/openapi/master/contracts/ref/cloud.yml
clerk             https://raw.githubusercontent.com/clerk/openapi-specs/main/bapi/2024-10-01.yml
dub               https://spec.speakeasy.com/dub/dub/dub-with-code-samples
spotify           https://raw.githubusercontent.com/sonallux/spotify-web-api/main/fixed-spotify-open-api.yml
deepl             https://raw.githubusercontent.com/DeepLcom/openapi/main/openapi.yaml
nomad             https://raw.githubusercontent.com/hashicorp/nomad-openapi/main/v1/openapi.yaml
pinecone          https://raw.githubusercontent.com/pinecone-io/pinecone-api/main/2026-04/db_control_2026-04.oas.yaml
hubspot-contacts  https://api.hubspot.com/public/api/spec/v2/specs/release/22150/version/3
```

## 3. Ranked alternates (if a main rots or proves unusable)

3.1: wolfram (APIs.guru permalink), hf-dataset-viewer, codat-banking,
codat-commerce, adyen-legalentity, adyen-transfers, sendgrid-email-activity,
github-ghes-317, adyen-balanceplatform (index-verified only), petstore31
(official toy, last resort). Sibling-of-existing-vendor specs were
deliberately demoted to alternates (manifest precedent: codat-accounting's
note sanctions siblings).

3.0: equinix-metal, braintrust, openfec, osf, telnyx-30-snapshot (pairs with
telnyx 3.1 like github-30/31), calcom, appwrite, twitch-community, langfuse,
ory-kratos, render, typesense, microsoft-graph (37 MB monster — only if we
want a new worst case), ebay-sell-fulfillment (APIs.guru permalink).

Rejected with reasons worth remembering: Slack/Netlify/PlanetScale/Weaviate/
Checkly/DocuSign are still Swagger 2.0; Mux/Fastly/Vultr/BigCommerce/Linear/
Airtable/Wise/Mailgun/Postmark have no stable raw spec URL; OpenProject is
3.1.2 but multi-file (74 relative file $refs — unusable for a single-file
harness).

## 4. Support-crate fetch prep (small changes before onboarding)

The download/cache logic lives in `conformance/support` after 05; these land
there:

- **Follow redirects**: opensearch's URL 302-redirects to
  release-assets.githubusercontent.com; verify ureq 3.1's redirect policy
  covers it (it should by default; assert with a unit test).
- **Send a User-Agent**: api.weather.gov rejects UA-less requests. Set an
  honest UA (`progenitor-conformance/<version> (+repo URL)`) on all
  downloads; it is also just polite at 120 specs.
- **GET-only is already correct** (several endpoints 404/405 on HEAD; the
  fetcher never HEADs — keep it that way).
- Stainless bucket URLs (increase, orb, lithic, runway, finch — same class as
  existing groq/anthropic) are hash-pinned and rotate; encode the
  re-resolution recipe in each crate's `spec.toml` notes (`{vendor}-python`
  repo, `.stats.yml`, `openapi_spec_url`) as the anthropic manifest note does
  today.
- Cache-extension quirk: the cache filename keys its extension off the URL
  suffix, so extensionless URLs (vapi, polygon, dub, posthog) will cache as
  `.json` even when YAML — harmless (parsing sniffs content, tailscale
  already does this) but worth a comment when the logic moves.

## 5. Onboarding process (batches of ~10, alternating 3.0/3.1)

1. `cargo run -p conformance-support --bin new-spec -- <name> <url>` per
   candidate; quirks from the tables above go into each `spec.toml` notes
   field. New crates start in staging (scaffolded but not yet workspace
   members, 05 §1) so the workspace stays green throughout.
2. `cargo check -p conformance-<name>`: a build.rs failure is the full
   generation diagnosis, a lib failure is rustc on the generated code.
   Triage: generator bug (fix it — that is the point of the corpus) vs.
   genuinely broken spec (leave in staging with a tracking note; the goal
   state remains all members, all green).
3. Promote green crates to workspace members; run the conformance
   `cargo check --workspace` to confirm the batch.
4. For every *novel* failure class fixed (expected from the feature census:
   patternProperties at posthog scale, webhooks blocks, prefixItems/const
   combinations, 3.1.1/3.1.2 version strings, binary-heavy jellyfin), write a
   behavioral test in the crate's lib.rs that deserializes a realistic
   payload through the affected types — the discord lesson: generate+compile
   green does not prove wire correctness.
5. Keep batches small so failures stay bisectable to a batch.

## 6. Acceptance criteria

- conformance/ has 120 spec crates as members (60 × 3.0.x, 60 × 3.1.x),
  staging empty.
- `generate_all` smoke, `cargo check --workspace`, and
  `cargo test --workspace` green at 120/120.
- Every new generator fix has a regression home: a typify/progenitor unit
  test or a crate-level behavioral test.
- PR-CI smoke stays under ~3 min at 120 specs (bounded thread pool).
- Decision D4 (per-crate `sha256` in spec.toml for cache reproducibility)
  revisited after the first URL rotation breaks a fresh checkout.
