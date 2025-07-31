# Feature Branches Ready for Merge

## Current Branch
- `feat/prp-codebase-fixes-cleanup` - Codebase fixes and cleanup (THIS BRANCH)

## Branches to Merge

As requested, the following feature branches should be merged:

1. **feat/prp-codebase-fixes-cleanup** (current)
   - TypeId component checks
   - Entity duplication fix
   - Physics raycast implementation
   - World module functions
   - Script system cleanup
   - Various fixes and improvements

2. **feat/prp-tracy-profiler-integration** 
   - Tracy profiler integration
   - Profiling zones throughout codebase
   - Performance monitoring capabilities

## Merge Instructions

```bash
# Switch to main/master branch
git checkout master

# Merge codebase fixes
git merge feat/prp-codebase-fixes-cleanup

# Merge Tracy profiler integration
git merge feat/prp-tracy-profiler-integration

# Push to remote
git push origin master
```

## Test Before Merge

Run validation with single-threaded tests:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings  
cargo test --workspace -- --test-threads=1
```

## Notes
- Hierarchy tests require `--test-threads=1` due to shared static frame counters
- All other tests pass normally
- Both branches have been tested and validated