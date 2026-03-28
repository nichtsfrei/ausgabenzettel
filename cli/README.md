# auseinnahmen

A CLI tool that helps you create and manage category-lookup rules from banking CSV exports.

## Description

`auseinnahmen` processes exported banking CSV files and builds category-lookup rules by:
- Checking existing rules from a previous lookup file
- Prompting interactively for new rules when transactions cannot be matched
- Writing output to stdout, file, or optionally overriding the lookup file

The tool outputs a JSON array of all rules that should be used for categorizing the current CSV.

## Installation

```bash
cd cli
cargo install --path .
```

Or build from source:

```bash
cd cli
cargo build --release
```

## Usage

```bash
auseinnahmen rules <csv> <categories> [--lookup <path>] [-o <output>] [-i] [--bank <type>]
```

### Arguments

- `<csv>`: Path to your banking CSV export (GLS or Sparda format supported)
- `<categories>`: Path to category definitions JSON file

### Options

- `-l, --lookup <path>`: Optional path to existing category lookup JSON file
- `-o, --output <path>`: Optional path to write output JSON (defaults to stdout)
- `-i, --override-lookup`: Override the lookup file with the new rules
- `-b, --bank <type>`: Bank type (gls or sparda) [default: gls]

### Output

The tool outputs a JSON array containing all rules used to match transactions from the CSV:

```json
[
  {
    "field": "Name",
    "value": "Amazon",
    "category": 0,
    "match_type": "Contains"
  },
  {
    "field": "IBAN",
    "value": "DE87300308801908262006",
    "category": 0,
    "match_type": "Exact"
  }
]
```

Each rule contains:
- `field`: The field type to match (IBAN/NAME/REFERENCE)
- `value`: The search value used for matching
- `category`: The category index (0-5 based on your definitions)
- `match_type`: Either `Exact` (exact match) or `Contains` (partial match)

## Category Definition

Create a `category.json` file like this:

```json
[
  {
    "index": 0,
    "title": "Alltag",
    "description": "Laufende Kosten f\u00fcr den t\u00e4glichen Bedarf"
  },
  {
    "index": 1,
    "title": "Ausgehen",
    "description": "Ausgaben f\u00fcr Freizeitaktivit\u00e4ten"
  }
]
```

The `index` must be sequential starting from 0.

## Matching Logic

The tool supports matching by:

1. **IBAN**: Exact match (case-sensitive)
2. **Name**: Exact match OR contains
3. **Reference**: Exact match OR contains

### How `match_type` works:
- `Exact`: The rule's value matches the transaction field exactly (e.g., `Amazon` == `Amazon`)
- `Contains`: The rule's value is found within the transaction field (e.g., `Amazon` in `AMAZON PAYMENTS`)

When you create a new rule:
- If you accept the default value shown, it's saved as `Exact`
- If you modify the default value, it's saved as `Contains`

## Interactive Prompt

When a transaction cannot be matched automatically, you'll see:

```
# transaction
AMOUNT   : 43,91
NAME     : AMAZON PAYMENTS EUROPE S.C.A.
IBAN     : DE87300308801908262006
REFERENCE: Somethihng

## category
1: Alltag - Laufende Kosten f\u00fcr den t\u00e4glichen Bedarf
2: Ausgehen - Ausgaben f\u00fcr Freizeitaktivit\u00e4ten

Select category index (or 's' to skip):
```

After selecting a category, you'll be prompted for the field type and search value:

```
## field
1: IBAN
2: NAME
3: REFERENCE

Select field index (or 's' to skip): 
Enter search value for NAME (default: 'AMAZON PAYMENTS EUROPE S.C.A.'): 
```

The tool automatically determines if the rule should be `Exact` or `Contains` based on whether you modified the default value.

## Example

```bash
# First run (no lookup file - creates initial rules, output to stdout)
auseinnahmen rules gls.csv category.json

# For Sparda bank CSV
auseinnahmen rules sparda.csv category.json --bank sparda

# Save output to file
auseinnahmen rules gls.csv category.json -o category-lookup.json

# Subsequent runs (extends rules from previous lookup)
auseinnahmen rules gls.csv category.json --lookup category-lookup.json -o updated.json

# Override lookup file with new rules
auseinnahmen rules gls.csv category.json --lookup category-lookup.json -i
```

## File Formats

### CSV Format

The tool expects GLS and Sparda bank CSV exports with semicolon (`;`) delimiters and UTF-8 BOM support. Required columns:
- `Betrag` (Amount)
- `Verwendungszweck` (Reference)
- `IBAN Zahlungsbeteiligter` (Counterparty IBAN)
- `Name Zahlungsbeteiligter` (Counterparty Name)

Use `--bank` flag to specify the bank type:
- `gls`: GLS Gemeinschaftsbank CSV format
- `sparda`: Sparda-Bank CSV format

### Lookup JSON Format

```json
[
  {
    "field": "IBAN",
    "value": "DE87300308801908262006",
    "category": 0,
    "match_type": "Exact"
  },
  {
    "field": "Name",
    "value": "Amazon",
    "category": 0,
    "match_type": "Contains"
  }
]
```

This rule would match:
- IBAN: exact match (`DE87300308801908262006`)
- Name: `Amazon` matches `AMAZON PAYMENTS`, `Amazon.de`, etc.
- Reference: `Amazon` matches `Amazon`, `Amazon something`, etc.

## Notes

- Rules are checked in the order they appear in the lookup JSON
- If a rule matches, the transaction is categorized and no further rules are checked
- The output JSON contains all rules found (existing + newly created)
