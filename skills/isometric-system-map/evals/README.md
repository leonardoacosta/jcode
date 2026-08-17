# Evaluation bindings

`evals.json` keeps repository paths symbolic so private source locations and snapshots are not
committed to this public skill package.

For a local run, bind:

- `$BROWN_WHOLESALE_REPO` to a read-only Brown wholesale repository snapshot and record its exact
  immutable ref in the run metadata.
- `$DECUS_SHARED_REPO` to a read-only decus-shared repository snapshot at an immutable main ref.

Save full generated artifacts and citation-level grading evidence outside the public repository.
Only sanitized aggregate scores belong in committed reports.
