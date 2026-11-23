# Repository refactor overview

Improve maintainability and consistency across data preparation, frontend workspaces, and dataset publishing. Release and verify data preparation changes one project at a time using the existing Google Cloud Storage (GCS) bucket (v2). Start the frontend workspace refactor after the planned project releases are verified in production.

## Stage 1 — Data preparation and project-by-project releases

### Remaining migration work

1. Define the migration scope and review each project's CLI, output layout, and promotion behavior against the [CPI pipeline](../../data_preparation/README.md). Check that the conventions also fit a dataset with different access patterns, such as vegetable relay with its summary and on-demand detail files.
2. Document the shared layout for acquisition records, raw inputs, and browser assets, including how browser assets are served and published. Align CLI behavior and the boundaries between acquisition, transformation, validation, and writing where those stages apply.
3. Apply the conventions where projects still differ. Preserve dataset-specific browser formats where access patterns differ. Where a format needs to change, evaluate request count and transfer size on the routes that use it.

### Project releases

Keep the frontend’s existing storage base URL. Use the following release steps for each project.

1. Inventory the project’s browser assets and the routes that consume them, including shared assets and files loaded on demand. Document the project’s commands, artifact locations, and browser paths and formats.
2. Run the Rust tests and regenerate and validate the project’s complete browser output, including files whose formats stay the same. Acquire source inputs where needed or replay verified saved inputs where supported. Check that the publishing allowlist includes all required browser assets and excludes source and audit files.
3. Publish only the current project to the existing GCS bucket. Use the existing prefix with the project-scoped publishing command when file-by-file updates are compatible with the deployed frontend and with a mix of old and new files. Retain a validated copy of the previous outputs for rollback. If paths or schemas change, or files must switch together, publish the complete output to a new versioned prefix in the existing GCS bucket and retain the previous prefix for rollback.
4. Before publishing in place, check the generated output with the current frontend. For a versioned release, run browser checks against a release build that reads directly from the new prefix before deploying the loader changes. After release, verify all affected routes in production, including interactions that fetch additional assets. If verification fails, republish the previous outputs for an in-place update or redeploy the previous frontend for a versioned release.

Record the published prefix, affected routes, and production verification result for each project. Stage 1 is complete when every project in the migration scope has passed production verification.

## Stage 2 — Frontend refactor

Start after the Stage 1 project releases are verified in production. Use the browser data contracts verified in those releases.

Share workspace components where routes have matching needs and rerun the browser baseline after changes.
