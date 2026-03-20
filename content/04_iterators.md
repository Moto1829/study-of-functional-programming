# 第4章: イテレータと遅延評価

## 4.1 `Iterator` トレイトの構造

Rust のイテレータは `Iterator` トレイトによって統一されています。このトレイトが要求するのは、たった1つのメソッド `next()` だけです。

```rust
pub trait Iterator {
    type Item;  // イテレータが返す要素の型

    fn next(&mut self) -> Option<Self::Item>;
    // 残りのメソッド（map, filter, fold ...）はすべてデフォルト実装が提供される
}
```

`next()` は要素が残っていれば `Some(value)` を、尽きたら `None` を返します。`for` ループはこの `next()` を繰り返し呼ぶ糖衣構文です。

```rust
fn main() {
    let v = vec![1, 2, 3];
    let mut iter = v.iter();

    // for ループと等価な手動呼び出し
    assert_eq!(iter.next(), Some(&1));
    assert_eq!(iter.next(), Some(&2));
    assert_eq!(iter.next(), Some(&3));
    assert_eq!(iter.next(), None);
}
```

### イテレータを生成する3つのメソッド

コレクションからイテレータを得る方法は用途によって3種類あります。

| メソッド | 所有権 | 要素の型 | 典型的な用途 |
|---------|--------|---------|------------|
| `iter()` | 借用（不変） | `&T` | 読み取り専用で使いたい |
| `iter_mut()` | 借用（可変） | `&mut T` | その場で書き換えたい |
| `into_iter()` | ムーブ（消費） | `T` | 所有権ごと処理したい |

```rust
fn main() {
    let mut v = vec![1, 2, 3];

    // 不変借用: 元の v はそのまま使える
    for x in v.iter() {
        println!("{}", x); // x は &i32
    }

    // 可変借用: 要素を直接書き換える
    for x in v.iter_mut() {
        *x *= 2; // x は &mut i32
    }
    assert_eq!(v, vec![2, 4, 6]);

    // ムーブ: v の所有権は for ループに移る
    for x in v.into_iter() {
        println!("{}", x); // x は i32
    }
    // この後 v は使えない
}
```

---

## 4.2 主要なイテレータアダプタ

`Iterator` トレイトには多数のデフォルトメソッドが用意されています。これらを**イテレータアダプタ**と呼びます。アダプタはイテレータを受け取り、別のイテレータを返します。

### `map` — 各要素を変換する

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5];

    // 各要素を2乗する
    let squared: Vec<i32> = numbers.iter()
        .map(|&x| x * x)
        .collect();

    assert_eq!(squared, vec![1, 4, 9, 16, 25]);
}
```

### `filter` — 条件を満たす要素だけ残す

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6];

    // 偶数だけを取り出す
    let evens: Vec<&i32> = numbers.iter()
        .filter(|&&x| x % 2 == 0)
        .collect();

    assert_eq!(evens, vec![&2, &4, &6]);
}
```

### `fold` — 要素を集約して1つの値にする

`fold` は初期値と「たたみ込み関数」を取り、すべての要素を1つの値に集約します。`sum` や `product` などの多くのアダプタは `fold` で実装されています。

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5];

    // 合計を求める
    let sum = numbers.iter().fold(0, |acc, &x| acc + x);
    assert_eq!(sum, 15);

    // 文字列を連結する
    let words = vec!["Hello", ", ", "world", "!"];
    let sentence = words.iter().fold(String::new(), |mut acc, &w| {
        acc.push_str(w);
        acc
    });
    assert_eq!(sentence, "Hello, world!");
}
```

### `flat_map` — 変換してからフラット化する

`flat_map` は各要素をイテレータに変換し、そのすべてを結合（flatten）します。ネストした構造を扱うときに便利です。

```rust
fn main() {
    let sentences = vec!["hello world", "foo bar baz"];

    // 各文章を単語に分割してフラットに並べる
    let words: Vec<&str> = sentences.iter()
        .flat_map(|s| s.split_whitespace())
        .collect();

    assert_eq!(words, vec!["hello", "world", "foo", "bar", "baz"]);

    // 入れ子の Vec をフラット化する
    let nested = vec![vec![1, 2, 3], vec![4, 5], vec![6]];
    let flat: Vec<i32> = nested.into_iter()
        .flat_map(|v| v.into_iter())
        .collect();

    assert_eq!(flat, vec![1, 2, 3, 4, 5, 6]);
}
```

### `zip` — 2つのイテレータをペアにする

```rust
fn main() {
    let names = vec!["Alice", "Bob", "Carol"];
    let scores = vec![95, 87, 72];

    // 名前とスコアをペアにする
    let result: Vec<(&&str, &i32)> = names.iter()
        .zip(scores.iter())
        .collect();

    // 短い方で打ち切られる
    let a = vec![1, 2, 3];
    let b = vec![10, 20]; // 要素数が少ない
    let zipped: Vec<_> = a.iter().zip(b.iter()).collect();
    assert_eq!(zipped.len(), 2); // 2ペアだけ
}
```

### `chain` — 複数のイテレータを連結する

```rust
fn main() {
    let first = vec![1, 2, 3];
    let second = vec![4, 5, 6];

    // 2つのイテレータを順につなげる
    let chained: Vec<i32> = first.iter()
        .chain(second.iter())
        .copied()
        .collect();

    assert_eq!(chained, vec![1, 2, 3, 4, 5, 6]);
}
```

---

## 4.3 イテレータチェーンによるデータ変換パイプライン

複数のアダプタをつなぐことで、データ変換のパイプラインを宣言的に表現できます。

```rust
fn process_log_lines(lines: &[&str]) -> Vec<String> {
    lines.iter()
        .filter(|line| !line.is_empty())          // 空行を除外
        .filter(|line| !line.starts_with('#'))    // コメント行を除外
        .map(|line| line.trim())                  // 前後の空白を削除
        .map(|line| line.to_uppercase())          // 大文字に変換
        .collect()
}

fn main() {
    let log = vec![
        "# これはコメントです",
        "  error: connection refused  ",
        "",
        "warning: retrying...",
        "# 別のコメント",
        "info: success",
    ];

    let processed = process_log_lines(&log);
    assert_eq!(processed, vec![
        "ERROR: CONNECTION REFUSED",
        "WARNING: RETRYING...",
        "INFO: SUCCESS",
    ]);
}
```

このスタイルの利点は**意図がステップとして読み取れる**ことです。命令型のループよりも何をしているかが明確です。

---

## 4.4 遅延評価の仕組み

Rust のイテレータアダプタは**遅延評価（lazy evaluation）**です。`map` や `filter` を呼んでも、その時点では何も処理されません。処理が走るのは、要素を消費する**終端メソッド**（`collect`, `for_each`, `fold`, `sum`, `count` など）が呼ばれたときだけです。

```rust
fn main() {
    // これだけでは何も起きない（コンパイラ警告が出る）
    let _lazy = vec![1, 2, 3]
        .iter()
        .map(|x| {
            println!("processing {}", x); // 呼ばれない
            x * 2
        });

    println!("イテレータを作った直後");

    // collect() を呼んで初めて map のクロージャが実行される
    let result: Vec<i32> = vec![1, 2, 3]
        .iter()
        .map(|x| {
            println!("processing {}", x); // ここで3回実行される
            x * 2
        })
        .collect();

    println!("collect() の後");
    assert_eq!(result, vec![2, 4, 6]);
}
```

### なぜ遅延評価か

遅延評価の最大の恩恵は**不要な中間コレクションが生成されない**点です。

```rust
fn main() {
    // 命令型: map の結果を Vec に一時保存してから filter する（中間 Vec が生成される）
    let v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let tmp: Vec<i32> = v.iter().map(|&x| x * x).collect(); // 中間 Vec
    let result: Vec<i32> = tmp.into_iter().filter(|x| x > &10).collect();

    // 関数型: map と filter はチェーンするだけ。中間 Vec は作られない
    let result2: Vec<i32> = v.iter()
        .map(|&x| x * x)
        .filter(|x| x > &10)
        .collect(); // ここで初めて全ステップが要素1つずつ通り抜ける

    assert_eq!(result, result2);
}
```

遅延評価では、要素は1つずつパイプラインを通り抜けます。`map` → `filter` の順で1要素ずつ処理されるため、メモリ効率が高くなります。

---

## 4.5 無限イテレータ

遅延評価を活かすと、**無限に続くイテレータ**を安全に扱えます。`take` で必要な個数だけ取り出せばいいからです。

### 基本的な無限イテレータ

```rust
use std::iter;

fn main() {
    // 0 から始まる無限の整数列
    let first_five: Vec<i32> = (0..).take(5).collect();
    assert_eq!(first_five, vec![0, 1, 2, 3, 4]);

    // 特定の値を無限に繰り返す
    let zeros: Vec<i32> = iter::repeat(0).take(3).collect();
    assert_eq!(zeros, vec![0, 0, 0]);

    // スライスを無限に繰り返す
    let cycling: Vec<i32> = vec![1, 2, 3].into_iter().cycle().take(7).collect();
    assert_eq!(cycling, vec![1, 2, 3, 1, 2, 3, 1]);

    // 関数で生成する無限列
    let mut n = 0;
    let naturals: Vec<i32> = iter::from_fn(move || {
        n += 1;
        Some(n)
    })
    .take(5)
    .collect();
    assert_eq!(naturals, vec![1, 2, 3, 4, 5]);
}
```

### 無限イテレータを使ったフィボナッチ数列

```rust
fn fibonacci() -> impl Iterator<Item = u64> {
    let mut state = (0u64, 1u64);
    std::iter::from_fn(move || {
        let next = state.0;
        state = (state.1, state.0 + state.1);
        Some(next)
    })
}

fn main() {
    let fibs: Vec<u64> = fibonacci().take(10).collect();
    assert_eq!(fibs, vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);

    // 100 未満のフィボナッチ数
    let under_100: Vec<u64> = fibonacci()
        .take_while(|&x| x < 100)
        .collect();
    assert_eq!(under_100, vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89]);
}
```

---

## 4.6 カスタムイテレータの実装

`Iterator` トレイトを自前実装することで、任意のデータ構造に対するイテレーションを定義できます。必要なのは `type Item` と `fn next()` だけです。残りの `map`, `filter`, `fold` などは自動的に使えるようになります。

### 例: カウントダウンイテレータ

```rust
/// カウントダウンするイテレータ
/// `from` から 1 まで 1 ずつ減少する
struct Countdown {
    current: u32,
}

impl Countdown {
    fn new(from: u32) -> Self {
        Countdown { current: from }
    }
}

impl Iterator for Countdown {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == 0 {
            None
        } else {
            let value = self.current;
            self.current -= 1;
            Some(value)
        }
    }
}

fn main() {
    let countdown: Vec<u32> = Countdown::new(5).collect();
    assert_eq!(countdown, vec![5, 4, 3, 2, 1]);

    // Iterator トレイトのメソッドがすべて使える
    let sum: u32 = Countdown::new(5).sum();
    assert_eq!(sum, 15);

    let doubled: Vec<u32> = Countdown::new(3).map(|x| x * 2).collect();
    assert_eq!(doubled, vec![6, 4, 2]);
}
```

### 例: ステップ付き範囲イテレータ

```rust
/// 開始値・終了値・ステップを指定できる範囲イテレータ
struct StepRange {
    current: i32,
    end: i32,
    step: i32,
}

impl StepRange {
    fn new(start: i32, end: i32, step: i32) -> Self {
        StepRange { current: start, end, step }
    }
}

impl Iterator for StepRange {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.end {
            return None;
        }
        let value = self.current;
        self.current += self.step;
        Some(value)
    }
}

fn main() {
    let result: Vec<i32> = StepRange::new(0, 10, 3).collect();
    assert_eq!(result, vec![0, 3, 6, 9]);
}
```

---

## 4.7 その他の便利なアダプタ

### `scan` — 状態を持ちながら変換する

`fold` に似ていますが、各ステップで中間値を出力するイテレータを返します。

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5];

    // 累積和を求める（各ステップの値を出力する）
    let running_sum: Vec<i32> = numbers.iter()
        .scan(0, |acc, &x| {
            *acc += x;
            Some(*acc) // 現在の累積値を出力
        })
        .collect();

    assert_eq!(running_sum, vec![1, 3, 6, 10, 15]);
}
```

### `take_while` / `skip_while` — 条件で切り取る

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 4, 3, 2, 1];

    // 条件が真の間だけ取り出す（条件が偽になったら即停止）
    let until_five: Vec<i32> = numbers.iter()
        .copied()
        .take_while(|&x| x < 5)
        .collect();
    assert_eq!(until_five, vec![1, 2, 3, 4]);

    // 条件が真の間スキップし、偽になったら残りをすべて返す
    let after_three: Vec<i32> = numbers.iter()
        .copied()
        .skip_while(|&x| x <= 3)
        .collect();
    assert_eq!(after_three, vec![4, 5, 4, 3, 2, 1]);
}
```

### `enumerate` — インデックス付きで走査する

```rust
fn main() {
    let fruits = vec!["apple", "banana", "cherry"];

    for (index, fruit) in fruits.iter().enumerate() {
        println!("{}: {}", index, fruit);
    }

    // enumerate の結果を変換に使う
    let indexed: Vec<String> = fruits.iter()
        .enumerate()
        .map(|(i, &f)| format!("{:02}. {}", i + 1, f))
        .collect();

    assert_eq!(indexed, vec!["01. apple", "02. banana", "03. cherry"]);
}
```

### `peekable` — 先頭要素を消費せずにのぞく

```rust
fn main() {
    let mut iter = vec![1, 2, 3].into_iter().peekable();

    // peek() は next() と違い、要素を消費しない
    assert_eq!(iter.peek(), Some(&1));
    assert_eq!(iter.peek(), Some(&1)); // 何度でも見られる
    assert_eq!(iter.next(), Some(1));  // 消費して初めて進む
    assert_eq!(iter.next(), Some(2));
}
```

---

## 4.8 `collect` の型指定と `FromIterator`

`collect()` の結果の型はターボフィッシュ構文 (`::<>`) または型注釈で指定します。`Vec` 以外にも `HashMap`, `HashSet`, `String` などに収集できます。

```rust
use std::collections::{HashMap, HashSet};

fn main() {
    let pairs = vec![("one", 1), ("two", 2), ("three", 3)];

    // HashMap に収集する
    let map: HashMap<&str, i32> = pairs.into_iter().collect();
    assert_eq!(map["two"], 2);

    // HashSet に収集する（重複除去）
    let with_dups = vec![1, 2, 2, 3, 3, 3];
    let set: HashSet<i32> = with_dups.into_iter().collect();
    assert!(set.contains(&1));
    assert_eq!(set.len(), 3); // 重複が除かれる

    // String に収集する
    let chars = vec!['R', 'u', 's', 't'];
    let s: String = chars.into_iter().collect();
    assert_eq!(s, "Rust");

    // Result を収集する（1つでも Err があれば全体が Err になる）
    let strings = vec!["1", "2", "3"];
    let numbers: Result<Vec<i32>, _> = strings.iter()
        .map(|s| s.parse::<i32>())
        .collect();
    assert_eq!(numbers.unwrap(), vec![1, 2, 3]);
}
```

---

## まとめ

| 概念 | ポイント |
|------|---------|
| `Iterator` トレイト | `next()` 1つだけ実装すれば残りのメソッドはすべて使える |
| 遅延評価 | アダプタは評価されない。`collect` などの終端メソッドで初めて処理が走る |
| イテレータチェーン | 宣言的なデータ変換パイプラインを表現できる |
| 無限イテレータ | `take` や `take_while` と組み合わせて安全に使える |
| カスタムイテレータ | `Iterator` トレイトを実装するだけで標準アダプタがすべて使える |

---

## 章末演習問題

### 演習1: イテレータチェーンによる集計

以下の商品リストから、価格が1000円以上の商品名を五十音順に並べ替えたリストを、イテレータチェーンだけで作成してください。`for` ループは使用不可です。

```rust
struct Product {
    name: String,
    price: u32,
}

fn expensive_products_sorted(products: &[Product]) -> Vec<&str> {
    // ここを実装してください
    todo!()
}
```

期待される動作:

```rust
let products = vec![
    Product { name: "apple".to_string(), price: 200 },
    Product { name: "melon".to_string(), price: 2000 },
    Product { name: "grape".to_string(), price: 1500 },
    Product { name: "banana".to_string(), price: 500 },
];
// 結果: ["grape", "melon"]
```

### 演習2: カスタムイテレータ — 素数列

`Iterator` トレイトを実装した `Primes` 構造体を作成してください。呼ぶたびに次の素数を返します。

```rust
struct Primes {
    // 必要なフィールドを定義してください
}

impl Iterator for Primes {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        // ここを実装してください
        todo!()
    }
}

fn main() {
    let first_ten: Vec<u64> = Primes::new().take(10).collect();
    assert_eq!(first_ten, vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29]);
}
```

### 演習3: `scan` を使った移動平均

整数のスライスを受け取り、直近 N 要素の移動平均を `scan` を使って計算する関数を実装してください（端はゼロ埋めなしで計算できる要素数が揃った時点から出力）。

```rust
fn moving_average(data: &[f64], window: usize) -> Vec<f64> {
    // scan と VecDeque（または固定長配列）を使って実装してください
    todo!()
}

fn main() {
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let avg = moving_average(&data, 3);
    // window=3 なら: [2.0, 3.0, 4.0] (先頭2要素分は window が揃わないため除外)
    assert_eq!(avg, vec![2.0, 3.0, 4.0]);
}
```

---

## 強化: rayon による並列イテレータ

### rayon とは

**rayon** は Rust の並列処理ライブラリで、通常のイテレータを並列イテレータに簡単に変換できます。`.iter()` を `.par_iter()` に変えるだけでマルチコアを活用できます。

```toml
# Cargo.toml
[dependencies]
rayon = "1"
```

### 基本的な使い方

```rust
use rayon::prelude::*;

let data: Vec<i64> = (1..=1000).collect();

// 通常のイテレータ
let seq_sum: i64 = data.iter().map(|&x| x * x).sum();

// 並列イテレータ（.iter() → .par_iter() に変えるだけ）
let par_sum: i64 = data.par_iter().map(|&x| x * x).sum();

assert_eq!(seq_sum, par_sum); // 結果は同じ
```

### 並列 map / filter / sum

```rust
// 並列 map
let squares: Vec<i64> = data.par_iter().map(|&x| x * x).collect();

// 並列 filter
let positives: Vec<i64> = data.par_iter()
    .filter(|&&x| x > 0)
    .copied()
    .collect();

// 並列 sum
let total: i64 = data.par_iter().sum();
```

### 並列パイプライン

複数のアダプタをつなげても並列で動作します:

```rust
let result: i64 = data.par_iter()
    .map(|&x| x * 2)
    .filter(|&x| x > 100)
    .sum();
```

### 注意点

| 観点 | 説明 |
|------|------|
| **順序** | `par_iter()` の結果の順序は保証されない（`collect()` はソートされない）。順序が必要なら `sort()` を追加する |
| **純粋関数** | `map` / `filter` に渡すクロージャは純粋である必要がある（共有状態を変更しない） |
| **オーバーヘッド** | 少量のデータではスレッド生成コストが上回ることがある。大きなデータセットで効果的 |

### 通常のイテレータとの使い分け

```rust
let small: Vec<i64> = (1..=10).collect();
// 少量データ: 通常イテレータの方が速い
let _ = small.iter().map(|&x| x * x).collect::<Vec<_>>();

let large: Vec<i64> = (1..=1_000_000).collect();
// 大量データ: rayon が効果的
let _ = large.par_iter().map(|&x| x * x).collect::<Vec<_>>();
```

