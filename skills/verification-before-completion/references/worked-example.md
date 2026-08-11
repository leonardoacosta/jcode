
# Worked Example: A Real Claim, Walked Through

Claim to verify: **"the API endpoint now returns 404 for missing records."**

**Hollow claim (banned shape):**
```
I updated the handler to check if the record exists and return a 404 if not.
I read through the code and the logic looks correct, so the endpoint now
returns 404 for missing records.
```
This is source-reading dressed up as verification. It proves the file contains
an `if (!record) return 404` branch — it proves nothing about what the running
server actually sends back.

**Actual verification chain for this claim:**
1. **Identify what command proves it** — an HTTP request against the real route
   for an ID that does not exist, checked for status code.
2. **Run it fresh, in this message:**
   ```
   $ curl -i http://localhost:3000/api/records/does-not-exist
   HTTP/1.1 404 Not Found
   content-type: application/json

   {"error":"Record not found"}
   ```
3. **Read the full output** — status line is `404`, body is the expected shape.
   Not "it printed something," the actual status code and body.
4. **State the claim WITH the evidence:**
   ```
   Verified: GET /api/records/does-not-exist returns 404 (curl output above).
   ```

If the app isn't running yet, standing it up (or writing a test that hits the
handler directly) is part of the verification step, not an excuse to skip it —
"I'd need to start the server to check" is a red flag phrase, not a reason to
fall back to reading the branch and calling it done.

The same chain shape applies outside HTTP: "build succeeds" needs a pasted
`exit 0` from the actual build command; "test passes" needs the pasted
pass-count line from the actual test run. The tool changes (curl, `pnpm
build`, `pnpm test`); the chain — identify, run fresh, read full output, THEN
claim — does not.
