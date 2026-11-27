# scripts/check_release_ci.py

- preview_job_status · function · L68-L93 — def preview_job_status(job, label)
- preview_run_status · function · L96-L100 — def preview_run_status(run, label): # Job evidence below must account for the failed workflow before authorizing.
- Rejected · class · L103-L104 — class Rejected(RuntimeError)
- WaitExpired · class · L107-L108 — class WaitExpired(RuntimeError)
- validate_inputs · function · L111-L118 — def validate_inputs(repository, sha, timeout, poll_interval)
- gh_api · function · L121-L131 — def gh_api(endpoint, paginated=False, request_timeout=60)
- objects · function · L134-L145 — def objects(api, endpoint, key)
- run_identity · function · L148-L157 — def run_identity(run, sha)
- status_reason · function · L160-L170 — def status_reason(item, label)
- check_once · function · L173-L236 — def check_once(repository, sha, api, preview=False)
- authorize · function · L239-L254 — def authorize(repository, sha, timeout=7200, poll_interval=30, api=gh_api, clock=time.monotonic, sleep=time.sleep, emit=print, preview=False)
- main · function · L257-L278 — def main(argv=None)
