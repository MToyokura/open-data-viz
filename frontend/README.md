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

`/vegetable_production_relay` fetches the generated product index from
`${VITE_STORAGE_BASE_URL}/vegetable_production_relay/products.json`, then loads
the selected product's aggregation JSON. Generate verified product data and its
index in `data_preparation/output/vegetable_production_relay/` with the commands
in `data_preparation/README.md`. The local storage server serves that output
directory. Only products with a generated, validated JSON file appear in the
selector; the そらまめ equivalent-block note is shown with its data.
The page labels tonnes for each year and distinguishes below-display-unit, no-fact,
and unknown/uncollected values. Its partial sums include numeric reports from the
nine top-level major consumer regions only. See
`data_preparation/inspections/vegetable_production_relay/SOURCE_INTERPRETATION.md`
for source meanings and the remaining cross-year scope question.
