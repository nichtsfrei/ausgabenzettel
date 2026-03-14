// src/parser.rs

use quick_xml::events::Event;
use quick_xml::Reader;

#[derive(Debug, PartialEq, Clone)]
pub struct Currency {
    pub amount: f64,
    pub currency: String,
}

impl Currency {
    pub fn from_str(s: &str) -> Option<Currency> {
        let s = s.trim();
        let mut amount_str = String::new();
        let mut currency_str = String::new();
        let mut in_amount = true;

        for c in s.chars() {
            if c.is_ascii_digit() || c == '.' || c == ',' {
                if in_amount {
                    amount_str.push(c);
                }
            } else {
                in_amount = false;
                currency_str.push(c);
            }
        }

        let amount = amount_str.replace(',', ".");
        let amount: f64 = amount.parse().ok()?;

        let currency = if currency_str.is_empty() {
            "€".to_string()
        } else {
            currency_str
        };

        Some(Currency { amount, currency })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Category {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Expense {
    pub id: String,
    pub category: Category,
    pub amount: Currency,
}

/// Parse HTML with structure like current.html:
/// <div id="details">
///   <details class="cat1" id="1767380618000">
///     <summary>
///       <span>Groceries</span>
///       <span>12.00€</span>
///     </summary>
///     <a href="#">remove</a>
///   </details>
/// </div>
///
/// Returns Vec<Expense> with all parsed entries.
pub fn parse_html_simple(html: &str) -> Vec<Expense> {
    let mut reader = Reader::from_str(html);
    //reader.trim_text(true); // ignore whitespace-only text

    let mut buf = Vec::new();
    let mut expenses = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                if e.name().as_ref() == b"details" {
                    let mut details_id = String::new();
                    let mut class = String::new();

                    // Extract attributes
                    let attrs = e.attributes();
                    for attr in attrs.flatten() {
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        match attr.key.as_ref() {
                            b"id" => {
                                details_id = value;
                            }
                            b"class" => {
                                class = value;
                            }
                            _ => {}
                        }
                    }

                    // Extract category_id from class like "cat1"
                    let category_id = extract_category_id(&class);

                    // Initialize expense (we’ll fill it as we parse summary)
                    let mut expense = Expense {
                        id: details_id,
                        category: Category {
                            id: category_id,
                            name: String::new(),
                        },
                        amount: Currency {
                            amount: 0.0,
                            currency: String::new(),
                        },
                    };

                    // Now parse until </details> or </summary>
                    let mut in_summary = false;
                    let mut summary_span_count = 0;

                    loop {
                        match reader.read_event_into(&mut buf) {
                            Ok(Event::Start(e)) => match e.name().as_ref() {
                                b"summary" => in_summary = true,
                                b"span" if in_summary => {
                                    summary_span_count += 1;
                                }
                                b"details" => {
                                    // Nested <details>? Skip (not expected)
                                    // For safety, break on </details> only at same level
                                }
                                _ => {}
                            },
                            Ok(Event::End(e)) => match e.name().as_ref() {
                                b"summary" => {
                                    in_summary = false;
                                }
                                b"details" => {
                                    // Finalize and push
                                    if !expense.id.is_empty() {
                                        expenses.push(expense);
                                    }
                                    break;
                                }
                                _ => {}
                            },
                            Ok(Event::Text(e)) => {
                                if in_summary && summary_span_count <= 2 {
                                    let aha = e;
                                    let text = String::from_utf8_lossy(&aha).to_string();
                                    if !text.trim().is_empty() {
                                        match summary_span_count {
                                            1 => expense.category.name = text,
                                            2 => {
                                                expense.amount = Currency::from_str(&text)
                                                    .unwrap_or(Currency {
                                                        amount: 0.0,
                                                        currency: text,
                                                    });
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            Ok(Event::Eof) => break,
                            Err(e) => {
                                eprintln!(
                                    "Parser error at pos {}: {:?}",
                                    reader.buffer_position(),
                                    e
                                );
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                eprintln!("Parser error at pos {}: {:?}", reader.buffer_position(), e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    expenses
}

/// Extracts numeric ID from class like "cat1", "cat42", etc.
fn extract_category_id(class: &str) -> u64 {
    if let Some(stripped) = class.strip_prefix("cat") {
        stripped.parse::<u64>().unwrap_or(0)
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INITIAL_HTML: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/testdata/initial.html"
    ));

    #[test]
    fn test_parse_html_simple_full() {
        let expected = vec![
            Expense {
                id: "1767380618000".to_string(),
                category: Category {
                    id: 1,
                    name: "Groceries".to_string(),
                },
                amount: Currency {
                    amount: 12.00,
                    currency: "€".to_string(),
                },
            },
            Expense {
                id: "1767381117000".to_string(),
                category: Category {
                    id: 4,
                    name: "Transportation".to_string(),
                },
                amount: Currency {
                    amount: 12.00,
                    currency: "€".to_string(),
                },
            },
        ];

        let parsed = parse_html_simple(INITIAL_HTML);
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_extract_category_id() {
        assert_eq!(extract_category_id("cat1"), 1);
        assert_eq!(extract_category_id("cat100"), 100);
        assert_eq!(extract_category_id("cat"), 0);
        assert_eq!(extract_category_id("other"), 0);
        assert_eq!(extract_category_id("catX"), 0);
    }

    #[test]
    fn test_empty_input() {
        assert!(parse_html_simple("").is_empty());
    }

    #[test]
    fn test_missing_id_or_class() {
        let html =
            r#"<details id="123"><summary><span>Test</span><span>10€</span></summary></details>"#;
        let expenses = parse_html_simple(html);
        assert_eq!(expenses.len(), 1);
        assert_eq!(expenses[0].id, "123");
        assert_eq!(expenses[0].category.id, 0); // fallback for missing/invalid class
        assert_eq!(expenses[0].category.name, "Test");
        assert_eq!(
            expenses[0].amount,
            Currency {
                amount: 10.0,
                currency: "€".to_string()
            }
        );
    }

    #[test]
    fn test_malformed_class() {
        let html = r#"<details class="cat-5" id="123"><summary><span>Test</span><span>10€</span></summary></details>"#;
        let expenses = parse_html_simple(html);
        assert_eq!(expenses[0].category.id, 0);
    }

    #[test]
    fn test_currency_from_str() {
        assert_eq!(
            Currency::from_str("12.00€"),
            Some(Currency {
                amount: 12.00,
                currency: "€".to_string()
            })
        );
        assert_eq!(
            Currency::from_str("12.00euro"),
            Some(Currency {
                amount: 12.00,
                currency: "euro".to_string()
            })
        );
        assert_eq!(
            Currency::from_str("12.00EUR"),
            Some(Currency {
                amount: 12.00,
                currency: "EUR".to_string()
            })
        );
        assert_eq!(
            Currency::from_str("100"),
            Some(Currency {
                amount: 100.0,
                currency: "€".to_string()
            })
        );
        assert_eq!(Currency::from_str("invalid"), None);
    }
}
