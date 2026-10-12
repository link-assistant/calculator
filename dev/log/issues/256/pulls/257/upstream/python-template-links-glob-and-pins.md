## Problem

1. **Every Broken Link Checker run warns about an empty input.** `links.yml` passes `'./**/*.md' './**/*.html'` to lychee, and the repository has no HTML file, so every run logs:

   ```
   [WARN] ./**/*.html: No files found for this input source
   ```

   (for example, run [38041366382](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/actions/runs/38041366382)). The js template fixed the same warning in its issue #219.
2. **First-party actions are a major behind:** `actions/checkout@v6` (24 uses) and `actions/setup-python@v6` (9 uses). The latest releases are v7.0.1 and v7.0.0.

## Reproduction

```sh
docker run --rm -v "$PWD":/input -w /input lycheeverse/lychee --offline './**/*.md' './**/*.html' 2>&1 | grep 'No files found'
grep -rhoE 'uses: actions/[a-z-]+@[^ ]+' .github | sort | uniq -c
```

## Fix

1. Use a directory input with an extension filter, as the js template does. This also respects `.gitignore`:

   ```yaml
          args: >-
            ...
            --hidden
            --extensions md,html
            --exclude-path docs/case-studies
            --exclude-path scripts/fixtures
            .
   ```

   I verified this on link-assistant/calculator: the same 80 links are checked, and the warning is gone.
2. Bump checkout and setup-python to v7.

Downstream: link-assistant/calculator#257.

Found while working on link-assistant/calculator#256.
