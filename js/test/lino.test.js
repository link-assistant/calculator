import { describe, expect, test } from "vitest";
import { Link, LinkRef, LinoParser } from "../src/lino/index.js";
import { unwrapResult } from "../src/rust/core.js";

describe("lino", () => {
  test("test_link_to_lino", () => {
    const link = Link.new([
      LinkRef.Literal("2"),
      LinkRef.Literal("+"),
      LinkRef.Literal("3"),
    ]);
    expect(link.toLino()).toBe("(2 + 3)");
  });

  test("test_link_with_id", () => {
    const link = Link.withId("sum", [
      LinkRef.Literal("2"),
      LinkRef.Literal("+"),
      LinkRef.Literal("3"),
    ]);
    expect(link.toLino()).toBe("(sum: 2 + 3)");
  });

  test("test_parse_simple", () => {
    const parser = LinoParser.new();
    const links = unwrapResult(() => parser.parse("hello"));
    expect(links.length).toBe(1);
    expect(links[0].refs.length).toBe(1);
  });

  test("test_parse_parenthesized", () => {
    const parser = LinoParser.new();
    const links = unwrapResult(() => parser.parse("(2 + 3)"));
    expect(links.length).toBe(1);
    expect(links[0].refs.length).toBe(3);
  });

  test("test_parse_nested", () => {
    const parser = LinoParser.new();
    const links = unwrapResult(() => parser.parse("((2 + 3) * 4)"));
    expect(links.length).toBe(1);
  });
});
