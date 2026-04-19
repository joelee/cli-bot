# Roadmap

This document tracks future features that may be added after the current release scope.

## Recently Shipped

### Multi-turn Session Memory

- Shipped in `0.3.0`
- Keeps short conversational context across follow-up requests in the same local session
- Uses working-directory-scoped sessions by default to avoid cross-project leakage
- See [Session Memory](session-memory.md) for the current behavior and future follow-up ideas

## Candidate Features

### External Dictionary And Language Tools

- Use external dictionary, spelling, or thesaurus tools when available
- Improve non-command requests such as spelling, definitions, or synonym lookups

### Better Non-command Result Types

- Expand beyond plain text and clarification with richer structured answers
- Support concise help, explanations, or command comparisons without executing anything

### Safer State-changing Command Classification

- Add a middle tier for state-changing commands that are not destructive but still deserve confirmation
- Examples: package installs, service restarts, and system updates

### Mocked Planner Integration Tests

- Add integration tests against a mocked Ollama endpoint
- Cover clarification loops, text responses, and malformed model output end-to-end

### Platform-aware Package Helpers

- Improve package-related prompts with richer examples per platform
- Add stronger handling for package search, upgrade, and uninstall workflows

### Session Logging Or History

- Optionally save prior requests and selected commands locally for review
- Useful for debugging, benchmarking, and workflow recall

## Notes

- Items here are candidates, not committed release promises.
- Keep CLI command generation as the primary product behavior.
