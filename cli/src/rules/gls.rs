use crate::rules::CsvRecord;
use std::fs;

pub fn read_csv(path: &str) -> Result<Vec<CsvRecord>, String> {
    let mut content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read CSV file: {}", e))?;

    content = content
        .trim_start_matches('\u{FEFF}')
        .replace("\r\n", "\n")
        .replace('\r', "\n");

    let mut records = Vec::new();
    let lines = content.lines().skip(1);

    for line in lines {
        let fields: Vec<&str> = line.split(';').collect();
        if fields.len() < 12 {
            continue;
        }

        records.push(CsvRecord {
            date: Some(fields[4].to_string()),
            amount: fields[11].to_string(),
            reference: fields[10].to_string(),
            iban: fields[7].to_string(),
            name: fields[6].to_string(),
        });
    }

    Ok(records)
}
