# 第9章: 実践 — 関数型スタイルでの設計と実装

## はじめに

これまでの章で学んだ概念（純粋関数、イテレータ、パターンマッチ、`Result`、トレイト）を組み合わせ、実際のプログラムをどのように設計するかを学びます。本章では「小さな式インタープリタ」を題材に、関数型スタイルの設計指針を体系的に示します。

---

## 9.1 副作用の分離 — 純粋なコアとI/O境界

### 設計指針

関数型スタイルの中心的な設計指針は、**副作用を持つコードと持たないコードを明確に分離すること**です。これを「関数コア、命令型シェル（Functional Core, Imperative Shell）」パターンと呼ぶこともあります。

```
┌─────────────────────────────────────┐
│           命令型シェル（I/O 境界）     │
│  ファイル読み込み / 標準入出力 / 時刻   │
│                                     │
│  ┌───────────────────────────────┐  │
│  │      関数型コア（純粋関数群）    │  │
│  │  変換 / 計算 / バリデーション   │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
```

- **内側（純粋なコア）**: 入力を受け取り出力を返すだけ。`mut` なし、I/O なし、単体テストが容易
- **外側（I/O 境界）**: ファイル・ネットワーク・標準入出力など副作用を一箇所に集める

### 実装例

```rust
use std::fs;

// ---- 純粋なコア（副作用なし、テスト容易） ----

/// CSV 形式の1行をパースして (名前, スコア) のペアを返す純粋関数。
fn parse_score_line(line: &str) -> Option<(&str, u32)> {
    let mut parts = line.splitn(2, ',');
    let name = parts.next()?.trim();
    let score = parts.next()?.trim().parse().ok()?;
    Some((name, score))
}

/// スコアリストから合格者（60点以上）の名前一覧を返す純粋関数。
fn passing_names(scores: &[(&str, u32)]) -> Vec<&str> {
    scores
        .iter()
        .filter(|(_, score)| *score >= 60)
        .map(|(name, _)| *name)
        .collect()
}

// ---- 命令型シェル（副作用をここだけに集める） ----

/// ファイルを読み込み、合格者を標準出力へ表示する。
/// I/O に関わる処理はこの関数の中だけに収まっている。
fn report_passing_from_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;           // 副作用: ファイル読み込み

    let scores: Vec<(&str, u32)> = content             // ここからは純粋な変換
        .lines()
        .filter_map(parse_score_line)
        .collect();

    let names = passing_names(&scores);                // 純粋なコアを呼ぶ

    for name in names {
        println!("合格: {name}");                       // 副作用: 標準出力
    }
    Ok(())
}
```

`parse_score_line` と `passing_names` は引数だけに依存するため、ファイルシステムをモックすることなく単体テストできます。副作用は `report_passing_from_file` の中だけに閉じ込められています。

---

## 9.2 パイプラインスタイルのデータ処理

### イテレータによるパイプライン

Rust のイテレータは**遅延評価**（lazy evaluation）です。`collect` や `sum` などの終端操作が呼ばれるまで実際の計算は行われません。これにより、変換の各ステップを独立した純粋な関数として表現しながら、効率的に処理できます。

```rust
/// 単語リストを受け取り、長さ4以上の単語を大文字にして
/// アルファベット順に並べた Vec を返す純粋関数。
fn process_words(words: &[&str]) -> Vec<String> {
    words
        .iter()
        .filter(|w| w.len() >= 4)          // 1. 短い単語を除外
        .map(|w| w.to_uppercase())          // 2. 大文字化（変換）
        .collect::<std::collections::BTreeSet<_>>()  // 3. ソート済みセットで重複除去
        .into_iter()
        .collect()                          // 4. Vec に変換
}
```

### `flat_map` による構造の平坦化

```rust
/// 各文をスペースで分割し、全単語を一つのイテレータとして得る。
fn all_words<'a>(sentences: &[&'a str]) -> Vec<&'a str> {
    sentences
        .iter()
        .flat_map(|s| s.split_whitespace())  // 文 → 単語列 → 平坦化
        .collect()
}

#[test]
fn test_all_words() {
    let sentences = ["hello world", "foo bar baz"];
    assert_eq!(
        all_words(&sentences),
        vec!["hello", "world", "foo", "bar", "baz"]
    );
}
```

### `scan` で状態を持つストリーム処理

累積値を保ちながら各要素を変換したい場合は `scan` を使います。

```rust
/// 数列の累積和を返す純粋関数。
fn running_sum(numbers: &[i32]) -> Vec<i32> {
    numbers
        .iter()
        .scan(0, |acc, &x| {
            *acc += x;
            Some(*acc)
        })
        .collect()
}

#[test]
fn test_running_sum() {
    assert_eq!(running_sum(&[1, 2, 3, 4]), vec![1, 3, 6, 10]);
}
```

---

## 9.3 小さなインタープリタの設計と実装

四則演算のみを扱う式インタープリタを題材に、関数型設計の全体像を示します。処理は次の3層に分かれます。

```
入力文字列
    │
    ▼
[トークナイザ]  純粋関数 + イテレータ
    │  Vec<Token>
    ▼
[パーサ]        ADT (Expr) + Result
    │  Expr (構文木)
    ▼
[評価器]        パターンマッチ + 再帰
    │  f64
    ▼
結果
```

各層は**独立した純粋関数**として実装し、`?` 演算子でエラー伝播を行います。

### 9.3.1 代数的データ型（ADT）によるモデリング

```rust
/// 字句解析器が生成するトークンの種類。
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// 数値リテラル（浮動小数点）
    Number(f64),
    /// `+` 演算子
    Plus,
    /// `-` 演算子
    Minus,
    /// `*` 演算子
    Star,
    /// `/` 演算子
    Slash,
}

/// 二項演算子の種類。
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

/// 式の構文木。再帰的な構造を `Box` で表現する。
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// 数値リテラル
    Num(f64),
    /// 二項演算（左辺 op 右辺）
    BinOp {
        op: Op,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
}
```

`enum` で型の全ての取りうる形を列挙することで、コンパイラがパターンマッチの網羅性を検査します。不正な状態を型として表現できないようにするのが関数型設計の要諦です。

### 9.3.2 トークナイザ — 純粋関数 + イテレータ

```rust
/// 入力文字列をトークン列に変換する純粋関数。
///
/// スペースはスキップし、数値・演算子をトークンに変換する。
/// 認識できない文字があった場合は `Err` を返す。
pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();  // peekable で先読みを可能に

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '\n' => { chars.next(); }   // 空白はスキップ
            '+' => { tokens.push(Token::Plus);  chars.next(); }
            '-' => { tokens.push(Token::Minus); chars.next(); }
            '*' => { tokens.push(Token::Star);  chars.next(); }
            '/' => { tokens.push(Token::Slash); chars.next(); }
            '0'..='9' | '.' => {
                // 数値の全桁を先読みしながら収集する
                let num_str: String = std::iter::from_fn(|| {
                    chars.peek().copied().filter(|c| c.is_ascii_digit() || *c == '.')
                        .map(|c| { chars.next(); c })
                }).collect();

                let value = num_str
                    .parse::<f64>()
                    .map_err(|e| format!("数値のパースに失敗: {e}"))?;
                tokens.push(Token::Number(value));
            }
            other => return Err(format!("認識できない文字: {other:?}")),
        }
    }
    Ok(tokens)
}
```

`Peekable` イテレータを使うことで、「次の文字を消費せずに確認する」という先読みを純粋に表現しています。

### 9.3.3 パーサ — ADT + Result

演算子優先順位を考慮した再帰下降パーサです。優先順位の低い加減算を外側に、高い乗除算を内側に置く古典的な設計です。

```rust
/// トークン列を構文木（`Expr`）に変換する純粋関数。
///
/// 演算子優先順位: `*` `/` > `+` `-`
/// 文法:
///   expr  = term  (('+' | '-') term)*
///   term  = number ('*' | '/') number)*
///   number = Number
pub fn parse(tokens: &[Token]) -> Result<Expr, String> {
    let (expr, rest) = parse_expr(tokens)?;
    if rest.is_empty() {
        Ok(expr)
    } else {
        Err(format!("パース後にトークンが残っています: {rest:?}"))
    }
}

/// 加減算レベルの式をパースし、(Expr, 残りトークン) を返す。
fn parse_expr(tokens: &[Token]) -> Result<(Expr, &[Token]), String> {
    let (mut lhs, mut rest) = parse_term(tokens)?;

    while let Some(token) = rest.first() {
        let op = match token {
            Token::Plus  => Op::Add,
            Token::Minus => Op::Sub,
            _ => break,
        };
        let (rhs, next) = parse_term(&rest[1..])?;
        lhs = Expr::BinOp { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
        rest = next;
    }
    Ok((lhs, rest))
}

/// 乗除算レベルの式をパースし、(Expr, 残りトークン) を返す。
fn parse_term(tokens: &[Token]) -> Result<(Expr, &[Token]), String> {
    let (mut lhs, mut rest) = parse_number(tokens)?;

    while let Some(token) = rest.first() {
        let op = match token {
            Token::Star  => Op::Mul,
            Token::Slash => Op::Div,
            _ => break,
        };
        let (rhs, next) = parse_number(&rest[1..])?;
        lhs = Expr::BinOp { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
        rest = next;
    }
    Ok((lhs, rest))
}

/// 数値トークン1つをパースし、(Expr::Num, 残りトークン) を返す。
fn parse_number(tokens: &[Token]) -> Result<(Expr, &[Token]), String> {
    match tokens.split_first() {
        Some((Token::Number(n), rest)) => Ok((Expr::Num(*n), rest)),
        Some((other, _)) => Err(format!("数値を期待しましたが {other:?} が来ました")),
        None => Err("トークンが不足しています".to_string()),
    }
}
```

各パース関数は `(Expr, 残りのトークンスライス)` を返します。スライスはポインタと長さのみを持つため、コピーコストがほぼゼロです。再帰の各段で「消費したトークン」を暗黙的に追跡できるのが、この設計の利点です。

### 9.3.4 評価器 — パターンマッチ + 再帰

```rust
/// 構文木を評価して `f64` を返す純粋関数。
///
/// ゼロ除算が発生した場合は `Err` を返す。
pub fn eval(expr: &Expr) -> Result<f64, String> {
    match expr {
        Expr::Num(n) => Ok(*n),

        Expr::BinOp { op, lhs, rhs } => {
            let l = eval(lhs)?;   // 左辺を再帰評価（失敗したら即 Err を返す）
            let r = eval(rhs)?;   // 右辺を再帰評価

            match op {
                Op::Add => Ok(l + r),
                Op::Sub => Ok(l - r),
                Op::Mul => Ok(l * r),
                Op::Div => {
                    if r == 0.0 {
                        Err("ゼロ除算".to_string())
                    } else {
                        Ok(l / r)
                    }
                }
            }
        }
    }
}
```

`eval` は `Expr` の構造をそのまま再帰で辿ります。`?` 演算子によって、左辺または右辺の評価でエラーが起きた場合は即座に呼び出し元へ伝播します。

### 9.3.5 全層を `?` で連結する `run`

```rust
/// 入力文字列を字句解析・パース・評価して結果を返す関数。
///
/// 各層のエラーは `?` で自動的に伝播する。
pub fn run(input: &str) -> Result<f64, String> {
    let tokens = tokenize(input)?;  // 字句解析
    let expr   = parse(&tokens)?;   // 構文解析
    eval(&expr)                     // 評価
}
```

3行で処理全体が表現されています。`?` によるエラー伝播のおかげで、成功パスだけを直線的に書け、エラー処理は各層に分散して記述されています。

---

## 9.4 命令型スタイルとの対比リファクタリング

同じ「単語の出現頻度カウント」を2つのスタイルで比較します。

### 命令型スタイル

```rust
use std::collections::HashMap;

/// 命令型: ループと可変な HashMap で単語頻度を数える。
fn word_count_imperative(text: &str) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for word in text.split_whitespace() {
        let lower = word.to_lowercase();
        let entry = counts.entry(lower).or_insert(0);
        *entry += 1;                     // 可変な状態を直接書き換える
    }
    counts
}
```

### 関数型スタイル

```rust
/// 関数型: fold で HashMap を「構築する変換」として表現する。
fn word_count_functional(text: &str) -> HashMap<String, usize> {
    text.split_whitespace()
        .map(|w| w.to_lowercase())       // 各単語を小文字化
        .fold(HashMap::new(), |mut acc, word| {
            *acc.entry(word).or_insert(0) += 1;
            acc
        })
}
```

`fold` を使うことで「`HashMap` を初期値として、単語を一つずつ追加していく変換」として読めます。`mut` は `fold` のクロージャ内部だけに局所化されます。

### さらに関数型らしく — `group_by` 的な使い方

```rust
/// 各単語の出現リストを収集し、最後にカウントする別アプローチ。
fn word_count_collect(text: &str) -> HashMap<String, usize> {
    text.split_whitespace()
        .map(|w| w.to_lowercase())
        .fold(HashMap::new(), |mut acc, word| {
            acc.entry(word).and_modify(|c| *c += 1).or_insert(1);
            acc
        })
}
```

どちらの関数型バージョンも命令型と同等の結果を返しますが、**変換の意図がコードの形に直接現れています**。

---

## 9.5 総まとめ — Rustで関数型スタイルを使う指針

### 指針1: 純粋関数を優先する

副作用なし・`mut` なし・外部状態への依存なしの関数を書くことを目指してください。純粋関数は単体テストが容易で、並列実行しても安全です。

```rust
// 良い: 入力だけに依存する純粋関数
fn normalize(s: &str) -> String {
    s.trim().to_lowercase()
}

// 注意: 外部状態（ファイル、時刻、乱数など）への依存は副作用
fn read_config() -> String {
    std::fs::read_to_string("config.toml").unwrap_or_default()
}
```

### 指針2: 型でドメインを表現する

`bool` や `String` でなく、専用の `enum` や `struct` を使って「不正な状態を表現できない型」を設計します。

```rust
// 避ける: bool は意味が薄い
fn set_status(active: bool) { /* ... */ }

// 良い: 状態を型で明示する
enum Status { Active, Inactive, Suspended }
fn set_status(status: Status) { /* ... */ }
```

### 指針3: エラーを `Result` で表現し `?` で連結する

パニックや `unwrap` の代わりに `Result` を使い、エラーを値として扱います。

```rust
fn pipeline(input: &str) -> Result<String, String> {
    let trimmed = validate_length(input)?;    // 長さチェック
    let parsed  = parse_fields(trimmed)?;     // フィールド分割
    let result  = transform(parsed)?;         // 変換
    Ok(result)
}
```

### 指針4: イテレータでデータ変換を宣言的に書く

ループよりもイテレータのメソッドチェーンを好みます。各ステップが何をするかが名前で明示され、読みやすくなります。

```rust
// 命令型: ループの中に複数の責務が混在する
fn process_imperative(items: &[i32]) -> Vec<String> {
    let mut result = Vec::new();
    for &item in items {
        if item > 0 {
            result.push(format!("{item}"));
        }
    }
    result
}

// 関数型: filter / map の責務が分離されている
fn process_functional(items: &[i32]) -> Vec<String> {
    items
        .iter()
        .filter(|&&x| x > 0)
        .map(|&x| format!("{x}"))
        .collect()
}
```

### 指針5: 副作用を境界に押し出す

アプリケーションのアーキテクチャとして、**内側を純粋な変換ロジック**、**外側をI/O**とするレイヤー構造を意識してください。これにより、コアロジックが完全にテスト可能になります。

```
main() / 非同期ランタイム
  └── I/O 層（ファイル・DB・HTTP）
        └── ドメイン層（純粋な変換ロジック）
              └── 型・バリデーション・計算
```

### 指針6: 必要なときだけ `mut` を使う

`let mut` は「ここに変更可能な状態がある」という強いシグナルです。イテレータや `fold` で書き換えられないか、まず検討してください。

```rust
// mut を使う前に: fold で書けないか検討する
let sum = (1..=100).fold(0_i64, |acc, x| acc + x);

// どうしても必要なときだけ mut を使う
let mut cache: HashMap<u64, u64> = HashMap::new();
```

---

## まとめ

本章で学んだことを整理します。

| テーマ | 関数型スタイルの実践 |
|---|---|
| 副作用の分離 | 純粋なコアと I/O 境界を明確に分ける |
| データ処理 | イテレータのメソッドチェーンで宣言的に書く |
| 型設計 | `enum` / `struct` で不正な状態を排除する |
| エラー処理 | `Result` + `?` で失敗を値として扱う |
| 評価ロジック | パターンマッチ + 再帰で構造を直接辿る |

Rust は純粋な関数型言語ではありませんが、所有権・型推論・パターンマッチ・イテレータという強力な道具が揃っています。「副作用を型や構造で明示する」という関数型の思考法を取り入れることで、**安全で読みやすく、テストしやすいコード**が自然と生まれます。
