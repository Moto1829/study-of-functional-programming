# 第13章: 実践プロジェクト — CSV データ処理パイプライン

## はじめに

本章では、これまで学んだ関数型プログラミングの概念を組み合わせた実践的な例として、**CSV データの処理パイプライン**を実装します。

パイプラインの流れ:
```
CSV テキスト → パース → フィルタ → 変換 → 集計 → レポート
```

---

## データ定義

```rust
#[derive(Debug, Clone)]
pub struct Record {
    pub name: String,
    pub category: String,
    pub value: f64,
}

#[derive(Debug)]
pub struct Summary {
    pub category: String,
    pub count: usize,
    pub total: f64,
    pub average: f64,
}
```

---

## Step 1: パース

```rust
/// CSV 行を Record に変換（失敗時は None）
pub fn parse_record(line: &str) -> Option<Record> {
    let parts: Vec<&str> = line.splitn(3, ',').collect();
    if parts.len() != 3 { return None; }
    let value = parts[2].trim().parse::<f64>().ok()?;
    Some(Record {
        name: parts[0].trim().to_string(),
        category: parts[1].trim().to_string(),
        value,
    })
}

/// CSV 全体をパース（ヘッダースキップ・エラー行除外）
pub fn parse_csv(csv: &str) -> Vec<Record> {
    csv.lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| parse_record(line))
        .collect()
}
```

`filter_map` でエラー行を自動的に除外します（第6章のエラー処理の応用）。

---

## Step 2: フィルタリング

```rust
pub fn filter_by_value(records: Vec<Record>, threshold: f64) -> Vec<Record> {
    records.into_iter().filter(|r| r.value >= threshold).collect()
}

pub fn filter_by_category<'a>(records: &'a [Record], category: &str) -> Vec<&'a Record> {
    records.iter().filter(|r| r.category == category).collect()
}
```

---

## Step 3: 変換

```rust
/// 値に係数を乗算（税率適用など）
pub fn apply_multiplier(records: Vec<Record>, multiplier: f64) -> Vec<Record> {
    records
        .into_iter()
        .map(|r| Record {
            value: (r.value * multiplier * 100.0).round() / 100.0,
            ..r  // 構造体更新構文：変更しないフィールドはそのまま
        })
        .collect()
}
```

`..r` は構造体更新構文で、変更するフィールドだけを明示できます。

---

## Step 4: 集計

```rust
use std::collections::HashMap;

pub fn summarize_by_category(records: &[Record]) -> Vec<Summary> {
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
```

`HashMap::entry().or_default()` でグループ化するパターンは頻出です。

---

## Step 5: パイプライン全体

```rust
pub fn process_pipeline(csv: &str, min_value: f64, tax_rate: f64) -> Vec<Summary> {
    parse_csv(csv)
        |> filter_by_value(_, min_value)     // Rust にパイプ演算子はないので…
        |> apply_multiplier(_, 1.0 + tax_rate)
        |> summarize_by_category(&_)
}
```

Rust にはパイプ演算子がないため、関数の連鎖で書きます:

```rust
pub fn process_pipeline(csv: &str, min_value: f64, tax_rate: f64) -> Vec<Summary> {
    let records = parse_csv(csv);
    let filtered = filter_by_value(records, min_value);
    let with_tax = apply_multiplier(filtered, 1.0 + tax_rate);
    summarize_by_category(&with_tax)
}
```

---

## Step 6: レポート生成

```rust
pub fn generate_report(summaries: &[Summary]) -> String {
    summaries
        .iter()
        .map(|s| format!(
            "{}: count={}, total={:.2}, avg={:.2}",
            s.category, s.count, s.total, s.average
        ))
        .collect::<Vec<_>>()
        .join("\n")
}
```

---

## 実行例

```rust
let csv = "name,category,value
Alice,food,100.0
Bob,food,200.0
Carol,tech,500.0
Dave,food,50.0
Eve,tech,300.0
Frank,travel,150.0";

let summaries = process_pipeline(csv, 100.0, 0.10);
let report = generate_report(&summaries);
println!("{}", report);
// food: count=2, total=330.00, avg=165.00
// tech: count=2, total=880.00, avg=440.00
// travel: count=1, total=165.00, avg=165.00
```

---

## 各章の技法まとめ

| 章 | 使用した技法 |
|----|------------|
| 第3章 | クロージャ（`filter`, `map` に渡す関数） |
| 第4章 | イテレータチェーン |
| 第5章 | `Option` による欠損値処理 |
| 第6章 | `filter_map` でエラー行を無視 |
| 第7章 | 関数合成によるパイプライン |

関数型スタイルの強みは、各ステップが**独立してテスト可能**で、**合成しやすい**点にあります。
