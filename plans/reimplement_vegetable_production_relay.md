# Vegetable Production Relay — Evidence-Driven Reimplementation Plan

**Status:** Proposed  
**Scope:** Rust data preparation, inspection, validation, publication, and eventual SvelteKit visualization  
**Proposed location:** `plans/reimplement_vegetable_production_relay.md`

## 1. Context and objective

The previous `vegetable_production_relay` implementation combined e-Stat discovery, data acquisition, CSV parsing, metadata interpretation, validation, aggregation, and publication in a large pipeline. Its complexity made it difficult to determine whether the published quantities were statistically correct. The implementation has been removed, and **this project starts from scratch**.

Build a small, observable pipeline whose values can be traced back to the original e-Stat responses and independently verified. **Do not use the old output as a correctness oracle.** It is useful for identifying requirements and possible edge cases, but any statistical assumption must be re-established from source evidence.

This plan incorporates the inspection-first principles of the former `plans/refactor_vegetable_relay_data_fetching.md`, adjusted from refactoring to greenfield reimplementation.

### Success criteria

For any published value, a developer can identify:

1. The source statistical table and its revision.
2. The raw source observations and classification codes that contributed.
3. The meaning and unit of those observations, supported by metadata/documentation.
4. The included and excluded markets, with reasons.
5. How missing, suppressed, zero, and below-precision values were handled.
6. The aggregation formula and completeness assessment.
7. The independent reference calculation that supports the result.

The pipeline must fail closed: ambiguous semantics or incomplete required data cannot silently produce a `complete` value.

## 2. Working principles

- **Evidence before assumptions.** Capture actual API responses and inspect them before committing to schemas or category mappings.
- **One small vertical slice first.** Start with `だいこん`, one observed table/year, and a few explicitly selected prefecture/month reference cases; expand only after verification.
- **Separation of concerns.** Network I/O, parsing, normalization, validation, aggregation, and publication must be independently testable.
- **Immutable evidence.** Save raw response bytes and checksums; derived artifacts must point to their inputs.
- **Replayable runs.** Given saved inputs and configuration, processing can run without network access.
- **Inspection does not change results.** Diagnostic recording must not influence transformations.
- **Explicit uncertainty.** Unknown classification, unit, duplicate, pagination gap, or exceptional symbol is an error or a documented unresolved condition—not a guessed conversion.
- **No premature framework.** Implement only the smallest inspection tooling needed for the current milestone.
- **No publication until verified.** A successful HTTP request, parser run, or JSON serialization is not evidence of statistical correctness.

## 3. Target processing boundaries

```text
e-Stat getStatsList
    -> discovered table candidates -> table selection report

e-Stat getMetaInfo
    -> captured metadata -> verified dimension/classification mapping

e-Stat getSimpleStatsData
    -> immutable CSV page captures -> complete acquisition report

raw CSV + verified metadata
    -> typed source observations
    -> normalized observations
    -> semantic/coverage validation
    -> monthly prefecture aggregation
    -> independent reference comparisons
    -> validated dataset candidate
    -> immutable published generation + manifest
    -> SvelteKit consumer
```

**Design constraint:** Do not combine these steps in a single large `pipeline.rs`. That file should orchestrate components rather than implement their business logic. Use typed records for boundaries, retaining source table IDs and source classification codes for traceability. Prefer explicit error types and structured validation findings over silent filtering or `unwrap()`.

### Suggested structure (evolve incrementally)

```text
data_preparation/
  src/modules/projects/vegetable_production_relay/
    mod.rs
    pipeline.rs                  # orchestration only
    models.rs                    # typed boundary contracts
    discovery.rs
    metadata.rs
    acquisition.rs
    parsing.rs
    normalization.rs
    validation.rs
    aggregation.rs
    inspection.rs
    publication.rs
  inspections/vegetable_production_relay/
    README.md
    findings/                   # reviewed, concise evidence-backed conclusions
    runs/<run-id>/              # generated; normally gitignored
      api/{stats_list,meta_info,simple_stats_data}/
      intermediate/
      final/
      reports/{summary.json,validation.json}
  tests/fixtures/vegetable_production_relay/  # small sanitized real fixtures
  output/vegetable_production_relay/
```

Directory names are a proposed contract, not a reason to create empty modules. Keep the old `commodity-prices` and `prefectures-update` workflows functional. Changes to shared HTTP/error utilities require separate regression checks.

## 4. Inspection contract

Each `inspect` run gets a unique ID (for example, UTC timestamp plus a collision-resistant suffix) and records the code revision/configuration, relevant source table identifiers, and status of each stage.

**For each API call**, capture:

- Endpoint and **sanitized** query parameters (never `ESTAT_APP_ID` or other secrets).
- Request time, duration, response status, selected headers, byte count, checksum.
- Exact raw response bytes, including unsuccessful responses for diagnosis.
- Pagination request parameters and reported total/continuation information.

**For each transformation**, capture:

- Input artifact identifiers/checksums and output artifact checksum.
- Input, accepted, excluded, rejected, and emitted row counts.
- Mapping/exclusion rules and their evidence references.
- Validation findings with stable codes, sample offending source identifiers, and severity.
- A readable summary in addition to machine-readable details.

**Suggested modes:**

- `inspect`: fetch/process and record evidence **without publishing**.
- `replay`: process saved raw API evidence **without network access**.
- `refresh`: validate candidate results and publish only when all required gates pass.

The CLI shape is not fixed yet; implement these concepts before stabilizing flags or commands. Inspection runs are local generated artifacts and should normally be gitignored. Commit small, reviewed, sanitized fixtures and durable findings. Avoid credentials in filenames, logs, URLs, and error strings.

## 5. Milestones and go/no-go gates

### M1 — Minimal API capture and replay

**Implement:** A thin, bounded-timeout HTTP client; status checking; explicit, testable retry policy for transient errors; raw-byte capture; checksum and sanitized request metadata; offline replay. Do not implement statistics logic.

**Inspect:** A real successful response and a controlled failing response, including recorded status and bytes.

**Gate:** Exact response bytes and checksum match; secrets are absent; failed HTTP responses are not interpreted as successful data; replay is network-free and deterministic.

**Deliverable:** Small inspection utility, one successful capture, one failure-path test, documentation on replay.

### M2 — Discover and select tables (`getStatsList`)

**Investigate:** Starting with `だいこん`, verify search semantics, table titles, statistical code (previously `00500226`), survey periods, publication/revision dates, and pagination. Determine whether there are multiple candidate tables per vegetable/year.

**Implement:** Candidate table extraction and **explicit selection rules**. Preserve candidate and rejection reasons.

**Inspect:** A table-selection report listing all candidates, selected IDs, excluded IDs, revisions, and evidence for each decision.

**Gate:** Selection is unambiguous and complete for the inspected scope; duplicates, conflicting tables, and truncated pages fail clearly.

**Deliverable:** Discovery fixtures, selected-table report, tested selection logic.

### M3 — Establish metadata semantics (`getMetaInfo`)

**Investigate:** Actual dimension names/codes, prefecture/origin classification, destination markets, measures, period/month mapping, units, annual totals, and category hierarchy. The earlier pipeline expected 47 prefectures and market category blocks; **verify rather than inherit** these expectations.

**Implement:** Typed representation of observed metadata and explicit mappings keyed by source codes. Keep display names separate from identifiers.

**Inspect:** Full classification inventory, code-to-meaning mapping, market inclusion/exclusion candidates, time interpretation, and unresolved ambiguities.

**Gate:** Every classification used later has a supported interpretation; no reliance on positional ordering unless independently corroborated; uncertain units or time meaning block progress.

**Deliverable:** Metadata findings document, mapping tests and fixtures.

### M4 — Acquire complete CSV and parse source observations (`getSimpleStatsData`)

**Investigate:** CSV response format, header/schema, encoding, page limits, continuation keys, reported totals, status/error bodies, value column, and units.

**Implement:** Acquisition of unmodified page bytes; a pagination completeness check; an offline parser into typed source observations retaining table ID, classification codes, raw quantity string, and unit.

**Inspect:** Raw page files, pagination ledger, parsed record count, sample source-to-typed-record comparisons.

**Gate:** No missing/duplicated pages or source positions; observed source row totals reconcile with API metadata; all fields needed for subsequent validation survive parsing.

**Deliverable:** Complete captured table, pagination report, parser fixture tests.

### M5 — Normalize and validate semantics

**Investigate:** Which rows represent quantities versus prices; which categories are totals versus contributors; which market categories overlap; what absent category combinations mean; authoritative meaning of `-`, `0`, `…`, `****`, and any other tokens. The old exclusion candidates were `東北_仙台・盛岡`, `関東_京浜`, and `九州_北九州`; validate the exclusion basis.

**Implement:** Explicit normalization, row selection, coverage checks, and value-status types (at least numeric, explicit zero, below precision, missing, suppressed). Never coerce missing or suppressed into zero. Reject unknown units/categories/symbols unless a documented rule permits them.

**Inspect:** Row ledger showing inclusion/exclusion reasons; coverage matrix for relevant months × origin prefectures × markets; counts by value status; unit diagnostics.

**Gate:** Every relevant raw row is accounted for; expected coverage is justified by source semantics; all exclusions have documented evidence; invalid/incomplete groups are identified rather than silently aggregated.

**Deliverable:** Validation report and stable rules covered by tests.

### M6 — Aggregate and independently verify reference values

**Proposed quantity:** For vegetable `v`, month `m`, and origin prefecture `p`, sum reported quantities across **verified non-overlapping contributing destination markets** `K`:

`Q(v,m,p) = Σ[k in K] q(v,m,p,k)`

This formula is **provisional** until M3–M5 establish units and market meaning. Annual totals must not be added to monthly observations; regional summary categories must not be double-counted.

**Implement:** Pure aggregation over validated observations. Carry completeness/status and source lineage forward. Do not label partly observed sums `complete`.

**Inspect:** Per-result contribution ledger (table ID, period, prefecture, market/code, raw value, normalized value, inclusion rule), status, and total.

**Independent check:** Manually or via a small independently written reference script, recompute selected prefecture-month totals directly from captured source CSV, using explicitly reviewed rules. Include high, low, zero, and missing/suppressed cases when available; avoid copying production aggregation logic into the verifier.

**Gate:** Reference totals reconcile at documented unit/precision tolerance; discrepancies halt the milestone; complete results have complete eligible coverage.

**Deliverable:** Verified one-vegetable result and checked-in reference cases.

### M7 — Safe publication and refresh

**Implement:** Candidate-generation staging, complete validation before publication, immutable generation files, and manifest-last publication. Recheck source revision IDs/checksums on refresh. Reuse cached raw data only when its identity and validity are established. Preserve previously published generations needed by old manifest snapshots.

**Inspect:** Candidate manifest, validation summary, asset checksums, source revision provenance, and simulated interrupted/failing runs.

**Gate:** Invalid/incomplete candidate cannot replace the current manifest; interrupted updates leave the previously published generation accessible; repeated replay produces equivalent logical output; changed source revisions trigger appropriate reprocessing.

**Deliverable:** Tested publisher and refresh mechanism.

### M8 — Expand vegetables and restore frontend

**Implement:** Add vegetables incrementally, choosing cases that stress different observed classifications and exceptional statuses. Only then restore the SvelteKit route/map and dataset consumption. Display missing, suppressed, and incomplete values distinctly from zero.

**Inspect:** Per-vegetable quality dashboard/report; frontend examples linked to verified published data.

**Gate:** Every published vegetable passes the same semantic/coverage/aggregation validation; representative map values can be traced to their captured source observations; no hidden partial results.

**Deliverable:** Validated multi-vegetable dataset and working visualization.

## 6. Validation taxonomy and failure policy

| Layer | What it establishes | Examples of blocking failures |
| --- | --- | --- |
| Transport/acquisition | Response was received and entirely captured | Failed HTTP status, truncated pagination, missing page |
| Structural | CSV and metadata can be parsed consistently | Unexpected columns, duplicated keys, invalid record shape |
| Semantic | Source classifications mean what we think | Unknown unit, ambiguous month, overlapping market categories |
| Statistical | Calculation is complete and correct | Missing contributing observations, wrong exclusion, reference mismatch |
| Publication | Readers see a coherent validated snapshot | Missing generation asset, manifest updated early |

Warnings may describe non-blocking observations; **every warning category must have a documented effect on publication**. No broad `catch-and-continue` for unexpected source shapes. For data known to be incomplete, preserve diagnostics while withholding or clearly marking incomplete aggregates according to the agreed output schema.

## 7. Test strategy

- **Fixture tests:** Real, sanitized API responses (including error and single-object/array variants where observed) to pin parsing and selection semantics.
- **Unit tests:** Pure normalization, status classification, coverage, exclusion, and aggregation logic.
- **Reference tests:** Independently calculated known cases from captured CSV; these are correctness checks, not comparisons against old pipeline output.
- **Replay integration tests:** End-to-end processing from saved source evidence; deterministic without credentials or network.
- **Fault injection:** Non-2xx responses, rate limiting, truncation, duplicate pages/rows, changed schema, missing categories, bad units, unknown symbols, publication interruption.
- **Optional live smoke tests:** Run explicitly with credentials and a bounded request budget; not required for ordinary offline CI.

Each bug discovered during inspection should generate (1) an evidence-backed finding and (2) a regression fixture/test before fixing the implementation.

## 8. PR workflow and review checklist

Work in small PRs aligned with milestones, optionally splitting any milestone if evidence reveals hidden complexity. Each PR should contain:

1. The question/hypothesis being tested.
2. Exactly which API captures or fixtures support the implementation.
3. A concise inspection finding and any unresolved ambiguity.
4. New or updated automated tests.
5. The validation report and a statement of whether the milestone gate passed.
6. Any behavior or source assumptions introduced, with evidence links.

**Do not merge a downstream stage if its upstream semantic gate is unresolved.** If source ambiguity cannot be resolved, document it and narrow scope rather than guessing.

## 9. Immediate first PR

**Title:** `Add vegetable relay API capture and replay infrastructure`

**In scope:** inspection directories/README; a minimal capture interface; redacted request metadata; exact response bytes, SHA-256 checksums, timeout/status/error behavior; offline replay and tests; `.gitignore` for generated runs.

**Out of scope:** table selection, metadata interpretation, CSV aggregation, all-vegetable refresh, publication, and frontend work.

**Acceptance demonstration:** Capture one `getStatsList` response, inspect its saved request/response/metadata, replay it without network access, and demonstrate that a failing request remains a recorded failure and does not expose `ESTAT_APP_ID`.

**Next decision:** Review the captured discovery response and write the evidence-backed questions for M2 before implementing discovery logic.

## 10. Non-goals until correctness is established

- Reproducing the old pipeline's outputs byte-for-byte.
- Supporting all vegetables or historical years from the first iteration.
- Optimizing requests/parallelism before acquisition and data semantics are verified.
- Building a generic ETL/inspection framework unrelated to the relay.
- Treating a rendered map as proof that underlying data is correct.

## 11. Final definition of done

The feature is complete when the team can choose a displayed vegetable/prefecture/month quantity, follow its lineage to immutable raw e-Stat observations, justify every inclusion/exclusion and exceptional-value decision, reproduce the computation offline, compare it against an independent reference calculation, and confirm that only validated generations can become publicly visible.
