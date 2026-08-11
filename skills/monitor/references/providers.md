# Monitor Providers

## Vercel

- Script: `scripts/bin/monitor-vercel-deploy`
- Shape: `monitor-vercel-deploy [--poll-seconds N] [--timeout-seconds N] PROJECT BRANCH`
- Default poll: 60 seconds
- Terminal stdout: `READY|ERROR|CANCELED<TAB>URL`
- Exit mapping: `READY` => `0`; `ERROR|CANCELED` => `1`
- Requirements: linked `vercel` CLI session

## GitHub Actions

- Script: `scripts/bin/monitor-gh-actions`
- Shape: `monitor-gh-actions [--poll-seconds N] [--timeout-seconds N] BRANCH OWNER/REPO`
- Default poll: 300 seconds
- Terminal stdout: `CONCLUSION<TAB>RUN_ID<TAB>URL`
- Exit mapping: `success|neutral|skipped` => `0`; failure-like conclusions => `1`
- Requirements: authenticated `gh` CLI

## Depot CI

- Script: `scripts/bin/monitor-depot-ci`
- Shape: `monitor-depot-ci [--poll-seconds N] [--timeout-seconds N] RUN_ID`
- Default poll: 300 seconds
- Terminal stdout: `STATUS<TAB>RUN_ID<TAB>URL`
- Exit mapping: success-like statuses => `0`; failure-like statuses => `1`
- Requirements: authenticated `depot` CLI
- Optional environment: `DEPOT_ORG_ID`, `DEPOT_API_TOKEN`

## Azure DevOps Pipeline

- Script: `scripts/bin/monitor-azure-pipeline`
- Shape: `monitor-azure-pipeline [--poll-seconds N] [--timeout-seconds N] ORG PROJECT [PIPELINE_ID]`
- Default poll: 300 seconds
- Terminal stdout: `STATE<TAB>RUN_ID<TAB>URL`
- Exit mapping: `succeeded` => `0`; `failed|canceled` => `1`
- Requirements: authenticated `az` CLI with Azure DevOps access

## Azure DevOps Classic Release

- Script: `scripts/bin/monitor-azure-release`
- Shape: `monitor-azure-release [--poll-seconds N] [--timeout-seconds N] ORG PROJECT DEFINITION_ID [ENVIRONMENT]`
- Default poll: 60 seconds
- Terminal stdout: `STATE<TAB>RELEASE_ID<TAB>URL`
- Exit mapping: `succeeded` => `0`; `failed|canceled|rejected|partiallysucceeded` => `1`
- Requirements: authenticated `az` CLI with VSRM API access
