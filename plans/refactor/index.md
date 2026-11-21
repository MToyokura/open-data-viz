# Repository refactor overview

Improve maintainability and consistency across data preparation, frontend workspaces, and dataset publishing. Release and verify data preparation changes one project at a time using the existing Google Cloud Storage (GCS) bucket (v2). Start the frontend workspace refactor after the planned project releases are verified in production.

## Stage 1 — Establish the frontend regression baseline

Add and run a minimal browser smoke test for every frontend page. Verify that each page loads and renders its primary content. Use small, deterministic fixtures for pages that load data. Start Stage 2 once the full baseline passes.

## Stage 2 — Data preparation and project-by-project releases

Use the [existing Consumer Price Index (CPI) pipeline](../../data_preparation/README.md) as the reference for shared CLI behavior, offline transformation from saved raw inputs, validation and staged promotion, and the separation of acquisition records, raw inputs, and browser assets. Preserve dataset-specific browser formats where their access patterns differ.

Subplans for the generated output file structure and data preparation module refactor will be added later.

1. Generated output file structure. Define paths for raw inputs, acquisition records, and browser assets, and document how those assets are served and published.
2. Data preparation module refactor. Define consistent CLI behavior and clear boundaries for acquisition, transformation, validation, and writing where those stages apply.
3. Dataset pipeline migration. Apply the shared layout and pipeline conventions to the datasets being migrated. Where a browser format needs to change, evaluate request count and transfer size for the routes that use it.

Check that the shared conventions also fit another dataset with different access patterns. Keep the frontend’s existing storage base URL and use the following release steps for each project.

1. Inventory the project’s browser assets and the routes that consume them, including shared assets and files loaded on demand. Document the project’s commands, artifact locations, and browser paths and formats.
2. Run the Rust tests and regenerate and validate the project’s complete browser output, including files whose formats stay the same. Acquire source inputs where needed or replay verified saved inputs where supported. Check that the publishing allowlist includes all required browser assets and excludes source and audit files.
3. Publish only the current project to the existing GCS bucket. Use the existing prefix with the project-scoped publishing command when file-by-file updates are compatible with the deployed frontend and with a mix of old and new files. Retain a validated copy of the previous outputs for rollback. If paths or schemas change, or files must switch together, publish the complete output to a new versioned prefix in the existing GCS bucket and retain the previous prefix for rollback.
4. Before publishing in place, check the generated output with the current frontend. For a versioned release, run browser checks against a release build that reads directly from the new prefix before deploying the loader changes. After release, verify all affected routes in production, including interactions that fetch additional assets. If verification fails, republish the previous outputs for an in-place update or redeploy the previous frontend for a versioned release.

Stage 2 is complete when every project in the migration scope has passed production verification.

## Stage 3 — Frontend refactor

Start after the Stage 2 project releases are verified in production. Use the browser data contracts verified in those releases.

Share workspace components where routes have matching needs.
