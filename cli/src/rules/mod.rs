use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};

pub mod dkb;
pub mod gls;

pub use dkb::read_csv as read_dkb_csv;
pub use gls::read_csv as read_gls_csv;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Category {
    pub index: usize,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CategoryLookupEntry {
    pub field: String,
    pub value: String,
    pub category: usize,
    pub match_type: MatchType,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CsvRecord {
    pub date: Option<String>,
    pub amount: String,
    pub reference: String,
    pub iban: String,
    pub name: String,
}

impl CsvRecord {
    pub fn non_empty_fields(&self) -> &[&str] {
        match (
            self.reference.is_empty(),
            self.name.is_empty(),
            self.iban.is_empty(),
        ) {
            (true, true, true) => &[],
            (true, true, false) => &["IBAN"],
            (true, false, true) => &["NAME"],
            (false, true, true) => &["REFERENCE"],
            (true, false, false) => &["NAME", "IBAN"],
            (false, true, false) => &["REFERENCE", "IBAN"],
            (false, false, true) => &["REFERENCE", "NAME"],
            (false, false, false) => &["REFERENCE", "NAME", "IBAN"],
        }
    }

    pub fn get_title(&self) -> String {
        format!("{} - {}", self.name, self.reference)
    }
}

impl CategoryLookupEntry {
    pub fn matches(&self, record: &CsvRecord) -> bool {
        let field_upper = self.field.to_uppercase();

        let expect_value = match field_upper.as_str() {
            "IBAN" => &record.iban,
            "REFERENCE" => &record.reference,
            "NAME" => &record.name,
            _ => panic!("Unknown field: {}", self.field),
        };
        match self.match_type {
            MatchType::Exact => &self.value == expect_value,
            MatchType::Contains => expect_value
                .to_uppercase()
                .contains(&self.value.to_uppercase()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum MatchType {
    Exact,
    Contains,
}

pub fn load_categories(path: &str) -> Result<Vec<Category>, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read categories file: {}", e))?;
    let categories: Vec<Category> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse categories JSON: {}", e))?;
    Ok(categories)
}

pub fn load_lookup(path: &str) -> Result<Vec<CategoryLookupEntry>, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read lookup file: {}", e))?;
    let entries: Vec<CategoryLookupEntry> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse lookup JSON: {}", e))?;
    Ok(entries)
}

pub fn find_category(record: &CsvRecord, lookup: &[CategoryLookupEntry]) -> bool {
    lookup.iter().any(|x| x.matches(record))
}

pub fn prompt<'a>(field: &'a str, elements: &'a [&'a str]) -> Option<usize> {
    loop {
        eprintln!("## {field}");
        for (idx, str) in elements.iter().enumerate() {
            eprintln!("{}: {str}", idx + 1);
        }
        eprintln!();
        print!("Select {field} index (or 's' to skip): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).unwrap() == 0 {
            return None;
        }
        let input = input.trim();

        if input == "s" {
            return None;
        }

        if let Ok(index) = input.parse::<usize>()
            && index - 1 < elements.len()
        {
            return Some(index - 1);
        }

        eprintln!("Invalid input. Please enter a valid {field} index or 's' to skip.");
    }
}

pub fn prompt_field(record: &CsvRecord) -> Option<(&str, &str)> {
    let fields: &[&str] = record.non_empty_fields();
    let field = prompt("field", fields).map(|x| fields[x]);
    field
        .iter()
        .filter_map(|x| match *x {
            "IBAN" => Some((*x, &record.iban as &str)),
            "NAME" => Some((*x, &record.name as &str)),
            "REFERENCE" => Some((*x, &record.reference as &str)),
            _ => None,
        })
        .next()
}

pub fn prompt_category(categories: &[Category]) -> Option<usize> {
    let prompt_title: Vec<&str> = categories.iter().map(|x| &x.title as &str).collect();
    prompt("category", &prompt_title)
}

pub fn prompt_search_value(field: &str, default: &str) -> String {
    eprint!(
        "Enter search value for {} (default: '{}'): ",
        field, default
    );
    io::stdout().flush().unwrap();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).unwrap() == 0 {
        return default.to_string();
    }
    let input = input.trim();

    if input.is_empty() {
        default.to_string()
    } else {
        input.to_string()
    }
}

pub fn prompt_for_category(
    categories: &[Category],
    record: &CsvRecord,
) -> Option<CategoryLookupEntry> {
    eprintln!();
    eprintln!();
    eprintln!("# transaction");
    eprintln!("AMOUNT   : {}", record.amount);
    eprintln!("NAME     : {}", record.name);
    eprintln!("IBAN     : {}", record.iban);
    eprintln!("REFERENCE: {}", record.reference);
    eprintln!();

    let category_index = prompt_category(categories)?;

    eprintln!();
    eprintln!();

    let (field, default_value) = prompt_field(record)?;

    let value = prompt_search_value(field, default_value);

    let match_type = if value == default_value {
        MatchType::Exact
    } else {
        MatchType::Contains
    };

    let rule = CategoryLookupEntry {
        field: field.to_string(),
        value,
        category: category_index,
        match_type,
    };

    Some(rule)
}

pub fn date_to_timestamp(date_str: &str, counter: u32) -> Option<u64> {
    let parts: Vec<&str> = date_str.trim().split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let day = parts[0].parse::<u32>().ok()?;
    let month = parts[1].parse::<u32>().ok()?;
    let mut year = parts[2].parse::<i32>().ok()?;
    if year < 2000 {
        year += 2000
    };
    let hour = counter % 24;
    let minute = (counter / 24) % 60;
    let second = (counter / (24 * 60)) % 60;

    let date = chrono::NaiveDate::from_ymd_opt(year, month, day)?;
    let datetime = date.and_hms_opt(hour, minute, second)?;
    let timestamp = datetime.and_utc().timestamp_millis();
    Some(timestamp as u64)
}

pub fn generate_html(
    records: &[CsvRecord],
    lookup: &[CategoryLookupEntry],
    categories: &[Category],
) -> String {
    let mut html = String::from("<div id=\"details\">");
    let mut date_counter = std::collections::HashMap::new();

    for record in records {
        let class = lookup.iter().find_map(|entry| {
            if entry.matches(record) {
                Some(entry.category)
            } else {
                None
            }
        });
        let class = class.and_then(|x| categories.iter().find(|y| y.index == x));

        let title = record.get_title();
        let amount = record.amount.replace("-", "").replace(',', ".");
        let amount = format!("{:.2}€", amount.parse::<f64>().unwrap_or(0.0));

        // TODO: that should actually not be an option
        let date_str = record.date.as_deref().unwrap_or("01.01.1111");
        let counter = date_counter.entry(date_str.to_string()).or_insert(0u32);
        let timestamp = date_to_timestamp(date_str, *counter).unwrap_or(0);
        *counter += 1;
        html.push_str(&format!(
            "<details class=\"cat{}\" id=\"{timestamp}\">",
            class.map(|x| x.index + 1).unwrap_or(404)
        ));
        html.push_str(&format!(
            "<summary><span>{title}</span><span>{amount}</span></summary>"
        ));
        html.push_str(
        &format!("<div><div class=\"row\"><span>Category</span><span class=\"right-align\">{}</span></div>", 
            class.map(|x|&x.title as &str).unwrap_or("Unknown"))
        );
        html.push_str("<a href=\"#\">remove</a></div>");
        html.push_str("</details>");
    }

    html.push_str("</div>");
    html
}
