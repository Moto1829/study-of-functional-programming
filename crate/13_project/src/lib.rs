// 第13章: 実践プロジェクト
// CSV データの関数型パイプライン処理

// ─── データ定義 ───────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    pub name: String,
    pub category: String,
    pub value: f64,
}

#[derive(Debug, PartialEq)]
pub struct Summary {
    pub category: String,
    pub count: usize,
    pub total: f64,
    pub average: f64,
}

// ─── パースパイプライン ───────────────────────────────────────

/// CSV 行をパースして Record に変換
pub fn parse_record(line: &str) -> Option<Record> {
    let parts: Vec<&str> = line.splitn(3, ',').collect();
    if parts.len() != 3 {
        return None;
    }
    let value = parts[2].trim().parse::<f64>().ok()?;
    Some(Record {
        name: parts[0].trim().to_string(),
        category: parts[1].trim().to_string(),
        value,
    })
}

/// CSV テキスト全体をパースして有効な Record のみ返す
pub fn parse_csv(csv: &str) -> Vec<Record> {
    csv.lines()
        .skip(1) // ヘッダー行をスキップ
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| parse_record(line))
        .collect()
}

// ─── フィルタリング ───────────────────────────────────────────

/// 値が閾値以上のレコードだけ残す
pub fn filter_by_value(records: Vec<Record>, threshold: f64) -> Vec<Record> {
    records.into_iter().filter(|r| r.value >= threshold).collect()
}

/// カテゴリでフィルタリング
pub fn filter_by_category<'a>(records: &'a [Record], category: &str) -> Vec<&'a Record> {
    records.iter().filter(|r| r.category == category).collect()
}

// ─── 変換 ─────────────────────────────────────────────────────

/// 値に係数を乗算（税計算などに使用）
pub fn apply_multiplier(records: Vec<Record>, multiplier: f64) -> Vec<Record> {
    records
        .into_iter()
        .map(|r| Record {
            value: (r.value * multiplier * 100.0).round() / 100.0,
            ..r
        })
        .collect()
}

// ─── 集計 ─────────────────────────────────────────────────────

/// カテゴリ別に集計する
pub fn summarize_by_category(records: &[Record]) -> Vec<Summary> {
    use std::collections::HashMap;

    let mut groups: HashMap<String, Vec<f64>> = HashMap::new();
    for r in records {
        groups.entry(r.category.clone()).or_default().push(r.value);
    }

    let mut summaries: Vec<Summary> = groups
        .into_iter()
        .map(|(category, values)| {
            let count = values.len();
            let total = values.iter().sum::<f64>();
            let average = total / count as f64;
            Summary {
                category,
                count,
                total: (total * 100.0).round() / 100.0,
                average: (average * 100.0).round() / 100.0,
            }
        })
        .collect();

    summaries.sort_by(|a, b| a.category.cmp(&b.category));
    summaries
}

/// 全レコードの合計値
pub fn total_value(records: &[Record]) -> f64 {
    records.iter().map(|r| r.value).sum()
}

// ─── パイプライン全体 ─────────────────────────────────────────

/// CSV を受け取り、フィルタ・変換・集計まで行う完全パイプライン
pub fn process_pipeline(csv: &str, min_value: f64, tax_rate: f64) -> Vec<Summary> {
    let records = parse_csv(csv);
    let filtered = filter_by_value(records, min_value);
    let with_tax = apply_multiplier(filtered, 1.0 + tax_rate);
    summarize_by_category(&with_tax)
}

// ─── レポート生成 ─────────────────────────────────────────────

/// 集計結果をテキストレポートに変換
pub fn generate_report(summaries: &[Summary]) -> String {
    let lines: Vec<String> = summaries
        .iter()
        .map(|s| {
            format!(
                "{}: count={}, total={:.2}, avg={:.2}",
                s.category, s.count, s.total, s.average
            )
        })
        .collect();
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_CSV: &str = "name,category,value
Alice,food,100.0
Bob,food,200.0
Carol,tech,500.0
Dave,food,50.0
Eve,tech,300.0
Frank,travel,150.0";

    #[test]
    fn test_parse_record() {
        let record = parse_record("Alice, food, 100.0").unwrap();
        assert_eq!(record.name, "Alice");
        assert_eq!(record.category, "food");
        assert_eq!(record.value, 100.0);
    }

    #[test]
    fn test_parse_record_invalid() {
        assert!(parse_record("invalid").is_none());
        assert!(parse_record("a,b,not_a_number").is_none());
    }

    #[test]
    fn test_parse_csv() {
        let records = parse_csv(SAMPLE_CSV);
        assert_eq!(records.len(), 6);
        assert_eq!(records[0].name, "Alice");
    }

    #[test]
    fn test_filter_by_value() {
        let records = parse_csv(SAMPLE_CSV);
        let filtered = filter_by_value(records, 100.0);
        assert_eq!(filtered.len(), 5); // 50.0 は除外
        assert!(filtered.iter().all(|r| r.value >= 100.0));
    }

    #[test]
    fn test_filter_by_category() {
        let records = parse_csv(SAMPLE_CSV);
        let food = filter_by_category(&records, "food");
        assert_eq!(food.len(), 3);
    }

    #[test]
    fn test_apply_multiplier() {
        let records = vec![Record {
            name: "test".into(),
            category: "cat".into(),
            value: 100.0,
        }];
        let result = apply_multiplier(records, 1.1);
        assert_eq!(result[0].value, 110.0);
    }

    #[test]
    fn test_summarize_by_category() {
        let records = parse_csv(SAMPLE_CSV);
        let summaries = summarize_by_category(&records);

        assert_eq!(summaries.len(), 3);
        let food = summaries.iter().find(|s| s.category == "food").unwrap();
        assert_eq!(food.count, 3);
        assert_eq!(food.total, 350.0);
    }

    #[test]
    fn test_process_pipeline() {
        // min_value=100, tax_rate=10%
        let summaries = process_pipeline(SAMPLE_CSV, 100.0, 0.10);
        // food: Alice(110), Bob(220) = 330; Dave(50<100)は除外
        let food = summaries.iter().find(|s| s.category == "food").unwrap();
        assert_eq!(food.count, 2);
        assert_eq!(food.total, 330.0);
    }

    #[test]
    fn test_generate_report() {
        let summaries = vec![
            Summary { category: "food".into(), count: 2, total: 300.0, average: 150.0 },
        ];
        let report = generate_report(&summaries);
        assert!(report.contains("food"));
        assert!(report.contains("count=2"));
    }
}
