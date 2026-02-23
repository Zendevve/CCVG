# Testing Strategy

## Test Levels

### Manual Testing (Primary)
This is a desktop GUI app — primary testing is manual using internal QA builds.

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
Automated coverage is currently minimal.

Priority areas for unit tests:
- Version parsing logic
- Directory size calculation
- Version sorting

## Test Environment

- Windows 10 or 11
- CapCut installed with multiple versions (ideal)
- Or CapCut installed with single version (minimum)

## Running Tests

Testing execution details are maintained in internal QA workflows.
