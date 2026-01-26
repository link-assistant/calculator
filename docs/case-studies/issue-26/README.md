# Case Study: Issue #26 - Deploy Failed

## Summary

The GitHub Pages deployment workflow failed due to a path mismatch between where the WASM package was being built and where the web application expected to find it.

## Timeline

| Timestamp (UTC) | Event |
|----------------|-------|
| 2026-01-26 08:43:12 | Deploy workflow triggered on push to main branch (commit 060863ea) |
| 2026-01-26 08:43:16 | Build job started on ubuntu-latest |
| 2026-01-26 08:43:31 | WASM package built successfully to `web/pkg` |
| 2026-01-26 08:44:06 | npm dependencies installed |
| 2026-01-26 08:44:09 | Vite build failed with ENOENT error |
| 2026-01-26 08:44:12 | Build job completed with failure status |

## Root Cause Analysis

### The Problem

The deployment workflow built the WASM package to the wrong directory:

```yaml
# deploy.yml (INCORRECT)
- name: Build WASM package
  run: wasm-pack build --target web --out-dir web/pkg
```

However, the web application was configured to look for the WASM package at a different location:

```typescript
// vite.config.ts
resolve: {
  alias: {
    '@wasm': path.resolve(__dirname, 'public/pkg'),
  },
},

// tsconfig.json
"paths": {
  "@wasm/*": ["./public/pkg/*"]
}
```

### Path Mismatch

| Configuration | Expected Path | Actual Path (deploy.yml) |
|--------------|---------------|--------------------------|
| vite.config.ts | `web/public/pkg` | `web/pkg` |
| tsconfig.json | `web/public/pkg` | `web/pkg` |
| ci.yml | `web/public/pkg` | `web/public/pkg` |
| deploy.yml | `web/public/pkg` | `web/pkg` |

The CI workflow (`ci.yml`) had the correct path, but the deploy workflow (`deploy.yml`) had an incorrect path.

### Error Message

```
[vite:worker-import-meta-url] Could not load /home/runner/work/calculator/calculator/web/public/pkg/link_calculator (imported by src/worker.ts): ENOENT: no such file or directory, open '/home/runner/work/calculator/calculator/web/public/pkg/link_calculator'
```

## Secondary Issue: npm ci Warning

The deployment also showed a warning during dependency installation:

```
npm error `npm ci` can only install packages when your package.json and package-lock.json or npm-shrinkwrap.json are in sync.
npm error Missing: @types/katex@0.16.8 from lock file
npm error Missing: katex@0.16.28 from lock file
npm error Missing: commander@8.3.0 from lock file
```

This indicates that the `package-lock.json` was out of sync with `package.json`, causing `npm ci` to fail and fall back to `npm install`.

## Solution

### Fix 1: Correct WASM Output Path in deploy.yml

Changed line 31 in `.github/workflows/deploy.yml`:

```yaml
# Before (INCORRECT)
- name: Build WASM package
  run: wasm-pack build --target web --out-dir web/pkg

# After (CORRECT)
- name: Build WASM package
  run: wasm-pack build --target web --out-dir web/public/pkg
```

### Fix 2: Update package-lock.json

Regenerated the `package-lock.json` file by running `npm install` to ensure it's in sync with `package.json`.

## Prevention Recommendations

1. **Consistent Path Configuration**: Create a single source of truth for the WASM package output directory, possibly as an environment variable or in a shared configuration file.

2. **Pre-deployment Validation**: Add a step in the deployment workflow to verify the WASM package exists at the expected location before attempting to build the web app.

3. **Workflow Testing**: Test workflow changes in a separate branch with the `workflow_dispatch` trigger before merging to main.

4. **Lock File Maintenance**: Set up a CI check or pre-commit hook to ensure `package-lock.json` stays in sync with `package.json`.

5. **Unified Build Scripts**: Consider creating a shell script or npm script that handles WASM building with consistent paths across all workflows.

## Related Files

- `.github/workflows/deploy.yml` - Fixed WASM output path
- `.github/workflows/ci.yml` - Reference for correct configuration
- `web/vite.config.ts` - Vite alias configuration
- `web/tsconfig.json` - TypeScript path mappings
- `web/src/worker.ts` - WASM import location
- `web/package-lock.json` - Updated to sync with package.json

## References

- GitHub Actions Run: https://github.com/link-assistant/calculator/actions/runs/21351396015
- Issue: https://github.com/link-assistant/calculator/issues/26
- Pull Request: https://github.com/link-assistant/calculator/pull/27

## Logs

Full CI logs are available in the `logs/` directory:
- `logs/run-21351396015.log` - Complete GitHub Actions log
- `logs/run-21351396015-metadata.json` - Run metadata (status, jobs, timestamps)
- `logs/npm-install.log` - npm install output during fix
