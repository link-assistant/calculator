// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=c86f4da3646aee57a9d1238f12cf0c556cec512016acafa5facbdaf5c8346c3a bytes=7256

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Links notation support for expression representation (port of `src/lino/mod.rs`).
// |  *
// |  * Links notation (lino) is a simple, intuitive format for representing
// |  * structured data as links between references to links.
// |  *
// |  * # Examples
// |  *
// |  * Basic expressions:
// |  * ```text
// |  * (2 + 3)           -> ((2) + (3))
// |  * 84 USD - 34 EUR   -> (((84 USD)) - ((34 EUR)))
// |  * ```
// |  *
// |  * Temporal expressions:
// |  * ```text
// |  * (84 USD - 34 EUR) at (22 Jan 2026)  -> ((((84 USD) - (34 EUR)) at (22 Jan 2026)))
// |  * ```
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=a60e57c5311f55f86c6800e43b3fe87df70f0cb92573037490c3f0a85d5ef5e5
// | import { err, ok } from "../rust/core.js";
use crate::rust::core_::{err, ok};
// meta-language:translated JavaScript import_statement items=1 sha256=3c0c11d6881f2337a6d291d7cf7bdccbde854b0f0adda76ad880c39068c43db4
// | import { isWhitespace, trim } from "../rust/str.js";
use crate::rust::str_::{isWhitespace, trim};

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * Formats an expression into links notation.
// |  * @param {import('../types/expression.js').Expression} expr
// |  * @returns {string}
// |  */
// | export function toLino(expr) {
// |   return expr.toLino();
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** Represents a link in links notation. */
// | export class Link {
// |   /** @param {{ id: string | null, refs: LinkRef[] }} fields */
// |   constructor({ id, refs }) {
// |     /** Optional identifier for this link. */
// |     this.id = id;
// |     /** The references that make up this link. */
// |     this.refs = refs;
// |   }
// |
// |   /**
// |    * Creates a new link with the given references.
// |    * @param {LinkRef[]} refs
// |    * @returns {Link}
// |    */
// |   static new(refs) {
// |     return new Link({ id: null, refs });
// |   }
// |
// |   /**
// |    * Creates a new link with an identifier.
// |    * @param {string} id
// |    * @param {LinkRef[]} refs
// |    * @returns {Link}
// |    */
// |   static withId(id, refs) {
// |     return new Link({ id, refs });
// |   }
// |
// |   /**
// |    * Formats this link as a links notation string.
// |    * @returns {string}
// |    */
// |   toLino() {
// |     const refsStr = this.refs.map((r) => r.toLino());
// |     const content = refsStr.join(" ");
// |
// |     if (this.id !== null) {
// |       return `(${this.id}: ${content})`;
// |     }
// |     if (this.refs.length > 1) {
// |       return `(${content})`;
// |     }
// |     return content;
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** A reference within a link. */
// | export class LinkRef {
// |   /** @param {'Literal' | 'Ref' | 'Nested'} tag @param {object} [fields] */
// |   constructor(tag, fields) {
// |     this.tag = tag;
// |     /** @type {any} */ this._0 = /** @type {any} */ (fields)?._0;
// |     Object.freeze(this);
// |   }
// |
// |   /** A literal value (number, string, etc.). @param {string} s */
// |   static Literal(s) {
// |     return new LinkRef("Literal", { _0: s });
// |   }
// |
// |   /** A reference to another link by its identifier. @param {string} id */
// |   static Ref(id) {
// |     return new LinkRef("Ref", { _0: id });
// |   }
// |
// |   /** A nested link. @param {Link} link */
// |   static Nested(link) {
// |     return new LinkRef("Nested", { _0: link });
// |   }
// |
// |   /**
// |    * Formats this reference as a links notation string.
// |    * @returns {string}
// |    */
// |   toLino() {
// |     switch (this.tag) {
// |       case "Literal":
// |         return this._0;
// |       case "Ref":
// |         return this._0;
// |       case "Nested":
// |         return this._0.toLino();
// |     }
// |   }
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /** Parser for links notation. */
// | export class LinoParser {
// |   /**
// |    * Creates a new links notation parser.
// |    * @returns {LinoParser}
// |    */
// |   static new() {
// |     return new LinoParser();
// |   }
// |
// |   /** @returns {LinoParser} */
// |   static default() {
// |     return new LinoParser();
// |   }
// |
// |   /**
// |    * Parses links notation into a list of links.
// |    * @param {string} input
// |    * @returns {Link[]} throws `Err(string)`
// |    */
// |   parse(input) {
// |     input = trim(input);
// |     if (input.length === 0) {
// |       return [];
// |     }
// |
// |     /** @type {Link[]} */
// |     const links = [];
// |     let pos = 0;
// |     const chars = [...input];
// |
// |     while (pos < chars.length) {
// |       // Skip whitespace
// |       while (pos < chars.length && isWhitespace(chars[pos])) {
// |         pos += 1;
// |       }
// |
// |       if (pos >= chars.length) {
// |         break;
// |       }
// |
// |       const [link, newPos] = this.parseLink(chars, pos);
// |       links.push(link);
// |       pos = newPos;
// |     }
// |
// |     return links;
// |   }
// |
// |   /**
// |    * @param {string[]} chars
// |    * @param {number} pos
// |    * @returns {[Link, number]}
// |    */
// |   parseLink(chars, pos) {
// |     if (pos >= chars.length) {
// |       return err("Unexpected end of input");
// |     }
// |
// |     if (chars[pos] === "(") {
// |       // Parenthesized link
// |       return this.parseParenthesizedLink(chars, pos);
// |     }
// |     // Single token (could be identifier or literal)
// |     return LinoParser.parseTokenAsLink(chars, pos);
// |   }
// |
// |   /**
// |    * @param {string[]} chars
// |    * @param {number} pos
// |    * @returns {[Link, number]}
// |    */
// |   parseParenthesizedLink(chars, pos) {
// |     pos = pos + 1; // Skip '('
// |     /** @type {LinkRef[]} */
// |     const refs = [];
// |     /** @type {string | null} */
// |     let id = null;
// |
// |     while (pos < chars.length && chars[pos] !== ")") {
// |       // Skip whitespace
// |       while (pos < chars.length && isWhitespace(chars[pos])) {
// |         pos += 1;
// |       }
// |
// |       if (pos >= chars.length || chars[pos] === ")") {
// |         break;
// |       }
// |
// |       // Check for identifier with colon
// |       if (id === null && refs.length !== 0) {
// |         // Already have refs, can't set id now
// |       } else {
// |         const parsed = LinoParser.tryParseIdentifierWithColon(chars, pos);
// |         if (parsed !== null) {
// |           id = parsed[0];
// |           pos = parsed[1];
// |           continue;
// |         }
// |       }
// |
// |       // Parse a reference
// |       const [linkRef, newPos] = this.parseRef(chars, pos);
// |       refs.push(linkRef);
// |       pos = newPos;
// |     }
// |
// |     if (pos >= chars.length) {
// |       return err("Unclosed parenthesis");
// |     }
// |
// |     pos += 1; // Skip ')'
// |
// |     return [new Link({ id, refs }), pos];
// |   }
// |
// |   /**
// |    * @param {string[]} chars
// |    * @param {number} pos
// |    * @returns {[Link, number]}
// |    */
// |   static parseTokenAsLink(chars, pos) {
// |     const [token, newPos] = LinoParser.parseToken(chars, pos);
// |     return [new Link({ id: null, refs: [LinkRef.Literal(token)] }), newPos];
// |   }
// |
// |   /**
// |    * @param {string[]} chars
// |    * @param {number} pos
// |    * @returns {[LinkRef, number]}
// |    */
// |   parseRef(chars, pos) {
// |     if (pos >= chars.length) {
// |       return err("Unexpected end of input");
// |     }
// |
// |     if (chars[pos] === "(") {
// |       const [link, newPos] = this.parseParenthesizedLink(chars, pos);
// |       return [LinkRef.Nested(link), newPos];
// |     }
// |     const [token, newPos] = LinoParser.parseToken(chars, pos);
// |     return [LinkRef.Literal(token), newPos];
// |   }
// |
// |   /**
// |    * @param {string[]} chars
// |    * @param {number} pos
// |    * @returns {[string, number] | null}
// |    */
// |   static tryParseIdentifierWithColon(chars, pos) {
// |     const parsed = ok(() => LinoParser.parseToken(chars, pos));
// |     if (parsed === null) {
// |       return null;
// |     }
// |     const [token, newPos] = parsed;
// |
// |     // Skip whitespace
// |     let checkPos = newPos;
// |     while (checkPos < chars.length && isWhitespace(chars[checkPos])) {
// |       checkPos += 1;
// |     }
// |
// |     if (checkPos < chars.length && chars[checkPos] === ":") {
// |       return [token, checkPos + 1];
// |     }
// |     return null;
// |   }
// |
// |   /**
// |    * @param {string[]} chars
// |    * @param {number} pos
// |    * @returns {[string, number]}
// |    */
// |   static parseToken(chars, pos) {
// |     let token = "";
// |
// |     // Skip leading whitespace
// |     while (pos < chars.length && isWhitespace(chars[pos])) {
// |       pos += 1;
// |     }
// |
// |     while (pos < chars.length) {
// |       const ch = chars[pos];
// |       if (isWhitespace(ch) || ch === "(" || ch === ")" || ch === ":") {
// |         break;
// |       }
// |       token += ch;
// |       pos += 1;
// |     }
// |
// |     if (token.length === 0) {
// |       return err(`Expected token at position ${pos}`);
// |     }
// |     return [token, pos];
// |   }
// | }
