use hashbrown::HashMap;

use std::env;
use std::fs::File;
use std::io::Read;
use std::time::Instant;

#[derive(Debug)]
struct Record {
    min: i64,
    max: i64,
    sum: i64,
    count: usize,
}

impl Default for Record {
    fn default() -> Self {
        Self {
            min: i64::MAX,
            max: i64::MIN,
            sum: 0,
            count: 0,
        }
    }
}

impl Record {
    fn add(&mut self, temperature: i64) {
        self.min = self.min.min(temperature);
        self.max = self.max.max(temperature);
        self.sum += temperature;
        self.count += 1;
    }

    fn average(&self) -> i64 {
        self.sum / self.count as i64
    }

    fn convert(temp: i64) -> f64 {
        temp as f64 * 0.1
    }

    /// Returns the value in *tenths of a degree* (e.g., `b"23.5" -> 235`, `b"-2.0" -> -20`).
    ///
    /// Expects a certain format:
    /// - A dot `'.'`
    /// - One or two digits
    /// - Exactly one fractional digit
    /// - Optional sign: `'-'`
    fn manual_parse(mut temp: &[u8]) -> i64 {
        let mut is_negative = false;
        if temp[0] == b'-' {
            temp = &temp[1..];
            is_negative = true;
        }

        let (a, b, c, d) = match temp {
            [c, b'.', d] => (0, 0, c - b'0', d - b'0'),
            [b, c, b'.', d] => (0, b - b'0', c - b'0', d - b'0'),
            [a, b, c, b'.', d] => (a - b'0', b - b'0', c - b'0', d - b'0'),
            // [c] => (0, 0, 0, c - b'0'),
            // [b, c] => (0, b - b'0', c - b'0', 0),
            // [a, b, c] => (a - b'0', b - b'0', c - b'0', 0),
            _ => panic!(
                "Temperature pattern not defined: {}",
                std::str::from_utf8(temp).unwrap()
            ),
        };

        let mut value = a as i64 * 1000 + b as i64 * 100 + c as i64 * 10 + d as i64;

        if is_negative {
            value *= -1;
        }

        value
    }
}

/// Stable helper: split a slice at the first element matching `pred`.
/// Returns (left, right_without_sep). `None` if no separator found.
fn split_once<T, F>(s: &[T], mut pred: F) -> Option<(&[T], &[T])>
where
    F: FnMut(&T) -> bool,
{
    let i = s.iter().position(pred)?;
    let (left, right_with_sep) = s.split_at(i);
    Some((left, &right_with_sep[1..]))
}

fn main() {
    let now = Instant::now();

    let filename = env::args().nth(1).unwrap_or("input.in".to_string());
    let mut file = File::open(filename).unwrap();

    let mut buffer = vec![];
    file.read_to_end(&mut buffer).unwrap();
    assert_eq!(buffer.pop(), Some(b'\n'));

    let mut city_records: HashMap<&[u8], Record> = HashMap::new();

    for line in buffer.split(|&c| c == b'\n') {
        let (city, temperature) =
            split_once(line, |&c| c == b';').expect("missing separator in line");

        let temperature = Record::manual_parse(temperature);

        city_records
            .entry(city)
            .or_insert(Record::default())
            .add(temperature);
    }

    let mut results = city_records.into_iter().collect::<Vec<_>>();
    results.sort_unstable_by(|a, b| a.0.cmp(b.0));

    for (city_name, record) in results {
        println!(
            "{:>16}: {:.1}/{:.1}/{:.1}",
            std::str::from_utf8(city_name).unwrap(),
            Record::convert(record.min),
            Record::convert(record.average()),
            Record::convert(record.max),
        );
    }

    println!("Elapsed time: {}", now.elapsed().as_secs());
}
