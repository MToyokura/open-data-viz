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

`/vegetable_production_relay` fetches an inspection-only daikon aggregation for
2021–2023 from `${VITE_STORAGE_BASE_URL}/vegetable_production_relay/daikon.json`.
Generate the file in `data_preparation/output/vegetable_production_relay/` with
the Rust aggregation command. The local storage server serves that output directory.
The data remains marked review-pending and nonpublishable; the page calls out the
unresolved unit for 2021, partial sums, and outstanding semantic and independent
aggregation reviews.
