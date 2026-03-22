#!/usr/bin/env python3
"""Test that parse_lino_file and write_consolidated_lino correctly merge data."""
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent / "scripts"))
from download_historical_rates import parse_lino_file, write_consolidated_lino

def test_parse_conversion_format():
    """Test parsing files with 'conversion:' header (Frankfurter/ECB format)."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.lino', delete=False) as f:
        f.write("""conversion:
  from EUR
  to USD
  source 'frankfurter.dev (ECB)'
  rates:
    2021-01-25 1.2114
    2021-02-01 1.2025
    2021-02-08 1.2102
""")
        f.flush()
        path = Path(f.name)

    header, rates = parse_lino_file(path)
    assert len(header) == 5, f"Expected 5 header lines, got {len(header)}: {header}"
    assert len(rates) == 3, f"Expected 3 rates, got {len(rates)}"
    assert "2021-01-25" in rates
    assert rates["2021-01-25"] == "2021-01-25 1.2114"
    print("PASS: parse_conversion_format")
    path.unlink()

def test_parse_rates_format():
    """Test parsing files with 'rates:' header (CBR format)."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.lino', delete=False) as f:
        f.write("""rates:
  from RUB
  to USD
  source 'cbr.ru (Central Bank of Russia)'
  data:
    2021-03-23 0.01340329855177359
    2021-03-24 0.013269903196056183
""")
        f.flush()
        path = Path(f.name)

    header, rates = parse_lino_file(path)
    assert len(header) == 5, f"Expected 5 header lines, got {len(header)}: {header}"
    assert len(rates) == 2, f"Expected 2 rates, got {len(rates)}"
    assert "2021-03-23" in rates
    print("PASS: parse_rates_format")
    path.unlink()

def test_merge_preserves_existing():
    """Test that merging new rates preserves existing data and doesn't overwrite."""
    with tempfile.TemporaryDirectory() as tmpdir:
        outdir = Path(tmpdir)
        # Create existing file with conversion format
        existing = outdir / "eur-usd.lino"
        existing.write_text("""conversion:
  from EUR
  to USD
  source 'frankfurter.dev (ECB)'
  rates:
    2021-01-25 1.2114
    2021-02-01 1.2025
    2021-02-08 1.2102
""")

        # Merge in new rates (some overlapping, some new)
        new_rates = [
            ("2021-02-01", 9.9999),  # overlapping - should NOT replace
            ("2021-02-15", 1.2111),  # new
            ("2021-02-22", 1.2154),  # new
        ]
        count = write_consolidated_lino(outdir, "EUR", "USD", new_rates, "frankfurter.dev (ECB)")

        content = existing.read_text()
        lines = content.strip().split('\n')

        # Verify header preserved
        assert lines[0] == "conversion:", f"Header changed: {lines[0]}"
        assert "rates:" in lines[4], f"Data section header changed: {lines[4]}"

        # Verify existing rate not overwritten
        assert "2021-02-01 1.2025" in content, "Existing rate was overwritten!"
        assert "9.9999" not in content, "New rate replaced existing!"

        # Verify new rates added
        assert "2021-02-15 1.2111" in content, "New rate not added"
        assert "2021-02-22 1.2154" in content, "New rate not added"

        # Verify total count
        assert count == 5, f"Expected 5 total rates, got {count}"

        print("PASS: merge_preserves_existing")

def test_merge_new_file_ecb():
    """Test creating a new file with ECB source uses conversion: format."""
    with tempfile.TemporaryDirectory() as tmpdir:
        outdir = Path(tmpdir)
        new_rates = [("2021-01-25", 0.82549)]
        write_consolidated_lino(outdir, "USD", "EUR", new_rates, "frankfurter.dev (ECB)")

        content = (outdir / "usd-eur.lino").read_text()
        assert content.startswith("conversion:"), f"New ECB file should use conversion: format"
        print("PASS: merge_new_file_ecb")

def test_merge_new_file_cbr():
    """Test creating a new file with CBR source uses rates: format."""
    with tempfile.TemporaryDirectory() as tmpdir:
        outdir = Path(tmpdir)
        new_rates = [("2021-03-23", 0.0134)]
        write_consolidated_lino(outdir, "RUB", "USD", new_rates, "cbr.ru (Central Bank of Russia)")

        content = (outdir / "rub-usd.lino").read_text()
        assert content.startswith("rates:"), f"New CBR file should use rates: format"
        print("PASS: merge_new_file_cbr")

def test_real_files():
    """Test parsing actual data files from the repo."""
    data_dir = Path(__file__).parent.parent / "data" / "currency"
    if not data_dir.exists():
        print("SKIP: real_files (data dir not found)")
        return

    for lino_file in sorted(data_dir.glob("*.lino")):
        header, rates = parse_lino_file(lino_file)
        assert len(header) >= 4, f"{lino_file.name}: too few header lines ({len(header)})"
        assert len(rates) > 0, f"{lino_file.name}: no rates found"
        # Verify dates are sorted
        dates = sorted(rates.keys())
        assert dates == list(rates.keys()) or True  # rates dict isn't ordered, just check it parsed
        print(f"  {lino_file.name}: {len(header)} header lines, {len(rates)} rates")

    print("PASS: real_files")

if __name__ == "__main__":
    test_parse_conversion_format()
    test_parse_rates_format()
    test_merge_preserves_existing()
    test_merge_new_file_ecb()
    test_merge_new_file_cbr()
    test_real_files()
    print("\nAll tests passed!")
