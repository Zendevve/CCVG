# Testing Strategy

## Test Levels

### Manual Testing (Primary)
This is a desktop GUI app — primary testing is manual via `cargo run --release`.

Test matrix:
- [ ] Welcome screen displays correctly
- [ ] Download Manager cards are clickable
- [ ] Download button opens browser
- [ ] PreCheck detects CapCut
- [ ] PreCheck detects if CapCut is running
- [ ] VersionSelect shows installed versions
- [ ] Version selection works
- [ ] Apply Protection runs all steps
- [ ] Complete screen shows success
- [ ] Error screen shows on failure
- [ ] Responsive at small window size
- [ ] Responsive at large/fullscreen size

### Unit Tests (Future)
`cargo test` — currently minimal coverage.

Priority areas for unit tests:
- Version parsing logic
- Directory size calculation
- Version sorting

## Test Environment

- Windows 10 or 11
- CapCut installed with multiple versions (ideal)
- Or CapCut installed with single version (minimum)

## Running Tests

```bash
# Manual test
cargo run --release

# Unit tests
cargo test
```
