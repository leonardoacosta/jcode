## 1. Preserve Link Destinations Through Rendering

- [x] 1.1 Extend Markdown render output to associate each link-label cell with its exact destination, and verify label/href associations in renderer unit tests
- [x] 1.2 Propagate link hit ranges through wrapping, message layout, and viewport scrolling, and verify coordinate mapping at wrapped and scrolled positions

## 2. Add Safe User-Initiated Opening

- [x] 2.1 Add a dedicated TUI user-click opener path that validates HTTP(S) and local `file:///` targets, honors explicit no-browser opt-outs, and reuses the system opener; verify valid URLs, existing local files, missing files, unsupported schemes, and opt-outs with a fake opener
- [x] 2.2 Handle left-clicks on link hit ranges before normal message selection, preserve existing behavior outside links, and verify both cases with mouse-event tests
- [x] 2.3 Surface opener startup or resolver failures without exiting the TUI, and verify the error path with an injected failing opener

## 3. Integration and Regression Verification

- [x] 3.1 Verify agent `open` tool requests remain harness-suppressed and no test launches a real external application
- [x] 3.2 Run the focused renderer, mouse-routing, and open-tool test suites, then run the relevant crate checks and record results
