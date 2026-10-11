# Historical year expansion: source inventory

**Checked:** 2026-10-11  
**Purpose:** Set the next engineering scope and identify source families before stabilizing a common application data model. This is a discovery note; it does not approve semantic mappings or publication.

## Findings

| Years/source | Evidence currently available | Coverage judgment |
| --- | --- | --- |
| 2012–2019 | e-Stat's annual producer-to-major-consumer-region series is listed for these years. The 2012 daikon listing has a `計` table and separate month-labeled files. The catalogue groups and table formats vary across years. | Candidate historical flow data. Table IDs, dimension codes, cell coverage, revisions, units, and year-to-year comparability must be discovered and checked per release. |
| 2020 | Repository investigation found separate month-titled and annual daikon tables (for example, `0001910468` and `0001910467`) without an explicit time dimension. | Strong integration candidate, but table selection, observation coverage, and annual-total behavior need a replayable verification. |
| 2021–2023 | One consolidated table per year uses 13 repeated `cat03` blocks. Dated official workbooks uniquely matched all 13 blocks in the current evidence. | Monthly flow observations can be adapted from the block schema. Preserve original `cat03`; keep the mapping evidence and review status attached. |
| 2024 | One table has explicit aggregate and monthly time codes. The inspection codebook records its axis and market roles. | Modern schema candidate. Its official workbook reconciliation and independent semantic review are still outstanding. |
| 2025 | MAFF announced the 2025 annual/monthly results on 2026-04-21 and the final report on 2026-09-30. e-Stat has a 2025 annual source listing. | Immediate discovery target. Do not claim 2025 daikon flow support until the exact origin-by-consumer-region table, metadata, units, populated periods, and cell coverage are verified. |
| 2026 | The current annual cycle is incomplete. | If monthly observations are available, represent them as a separate partial year with an explicit as-of date and release status. Never present them as a complete year. |
| 1985 onward long-term series | MAFF/e-Stat publish a daikon long-term series covering 1985–2025. It is a city-class annual quantity/value/price series, not the origin-prefecture × consumer-region monthly flow. The source notes methodology changes, including changes after 2022. | Useful as a separate historical series or context, not a compatible substitute for flow data. Keep it under a distinct coverage type. |

## Architecture implications

1. Catalogue discovery is already paginated and keyed by the statistics code. Keep table IDs as discovered evidence, not year constants in code. Select candidates from catalogues using explicit reviewed criteria and `getMetaInfo`, then bind captures to the discovered table ID and revision.
2. Build adapters around observed source shapes, not annual boundaries: (a) separate table per period, (b) repeated temporal blocks in a classification axis, and (c) explicit time axis. Add another adapter only when a verified release requires it.
3. Keep source observations and normalized observations distinct. The normalized record must retain the source table/revision, source codes and raw value, interpreted period, unit, and mapping provenance. A common application record must not erase original evidence.
4. A generated availability manifest must be based on built artifacts and their coverage checks, not on a configured year list. Each year/series states its grain, completeness, source status, and whether it supports origin-to-region flow. A candidate or annual city-class series must not appear as a monthly flow.
5. Treat 1985–2025 long-term city-class totals as a different series family. Do not force its geography or annual grain into the flow record schema.

## Next investigation sequence

1. Refresh the 2025 e-Stat catalogue and identify the exact daikon origin-by-consumer-region candidate; inspect metadata and a complete capture before assigning a supported status.
2. Replay and verify the previously investigated 2020 table family, including annual versus monthly table selection and all 12 reporting months.
3. Inventory the 2012–2019 daikon flow files and group years by metadata/schema fingerprint. Start with 2012 and one year at each apparent format transition.
4. Define the provisional common record only after those representatives have been inspected; record unresolved differences as explicit optional fields or separate coverage profiles.
5. Add 2026 only when a source release is discoverable; emit it as partial with an as-of period, separate from complete annual releases.

## Official references

- [MAFF: Vegetable wholesale market survey](https://www.maff.go.jp/j/tokei/kouhyou/seika_orosi/): current releases, 2025 first report and final-report notice, and long-term series entry point.
- [e-Stat: 2012 daikon origin-by-consumer-region tables](https://www.e-stat.go.jp/stat-search/files?layout=datalist&lid=000001128209): separate `計` and January–December daikon files in the 2012 release.
- [e-Stat: 2012 source catalogue](https://www.e-stat.go.jp/stat-search/files?cycle=7&layout=dataset&page=1&tclass1=000001020455&tclass2=000001067227&tclass3val=0&toukei=00500226&tstat=000001015623): annual release structure and table collection.
- [e-Stat: 2025 daikon long-term series](https://www.e-stat.go.jp/stat-search/files?layout=datalist&lid=000001491111&page=1): long-term daikon series updated through 2025.

