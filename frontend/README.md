フロントエンド側のディレクトリです

## Developing

```bash
nodenv install
npm install
npm run dev
```

## Building

```bash
npm run build
```

## Previewing

```bash
npm run preview
```

## Vegetable relay preview

`/vegetable_production_relay` fetches a daikon aggregation for
2021–2023 from `${VITE_STORAGE_BASE_URL}/vegetable_production_relay/daikon.json`.
Generate the file in `data_preparation/output/vegetable_production_relay/` with
the Rust aggregation command. The local storage server serves that output directory.
The page labels tonnes for each year and distinguishes below-display-unit, no-fact,
and unknown/uncollected values. Its partial sums include numeric reports from the
nine top-level major consumer regions only. See
`data_preparation/inspections/vegetable_production_relay/SOURCE_INTERPRETATION.md`
for source meanings and the remaining cross-year scope question.
