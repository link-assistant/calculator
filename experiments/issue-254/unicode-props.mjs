// Prints the Unicode version and a few property memberships of the running
// engine (Node or Bun), to compare engines with Rust's tables.
const probe = (re, code) => re.test(String.fromCodePoint(code));
const versions = typeof process !== 'undefined' ? { unicode: process.versions.unicode, icu: process.versions.icu, bun: process.versions.bun, node: process.versions.node } : {};
console.log(JSON.stringify({
  versions,
  ff9e_grapheme_extend: probe(/^\p{Grapheme_Extend}$/u, 0xff9e),
  ff9e_lm: probe(/^\p{Lm}$/u, 0xff9e),
  hangul_filler_di: probe(/^\p{Default_Ignorable_Code_Point}$/u, 0x3164),
  u1c89_assigned: !probe(/^\p{Cn}$/u, 0x1c89),
  u31e4_assigned: !probe(/^\p{Cn}$/u, 0x31e4),
  u1e5d0_alpha: probe(/^\p{Alphabetic}$/u, 0x1e5d0),
}));
