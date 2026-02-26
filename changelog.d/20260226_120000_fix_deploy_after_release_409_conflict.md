---
bump: patch
---

### Fixed
- Fixed `deploy-after-release` CI/CD job failing with 409 Conflict when uploading the GitHub Pages artifact (issue #78)
  - Root cause: `actions/upload-pages-artifact` always uses the artifact name `github-pages` by default. When both `deploy-pages` and `deploy-after-release` jobs run in the same workflow, the second upload fails because the name is already taken.
  - Fix: pass `name: github-pages-after-release` to `upload-pages-artifact` and `artifact_name: github-pages-after-release` to `deploy-pages` in the `deploy-after-release` job, so it uses a unique artifact name.
