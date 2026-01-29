### Changed
- Replace standard base64 URL encoding with URL-safe base64url encoding (RFC 4648 Section 5)
- URLs now use only `a-zA-Z0-9`, `-`, and `_` characters, avoiding URL encoding issues with `+`, `/`, and `=`

### Added
- Auto-calculate when expression is loaded from URL (calculation triggered immediately on page load)
- Backward compatibility: old base64 URLs are automatically redirected to new base64url format
- Prevent duplicate browser history entries when same expression is recalculated
