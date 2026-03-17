# 第5章: パターンマッチングと代数的データ型

## 5.1 代数的データ型（ADT）とは

**代数的データ型（Algebraic Data Types、ADT）**は、関数型プログラミングの中核をなすデータモデリングの概念です。「代数的」とは、型を**直和（Sum）**と**直積（Product）**という2種類の合成で構築できることを指します。

| 種別 | 数学的な意味 | Rust での表現 |
|------|-------------|--------------|
| 直積型（Product type） | 型 A と型 B の**両方**の値を持つ | `struct` |
| 直和型（Sum type） | 型 A **または** 型 B のどちらかの値を持つ | `enum` |

この2つの組み合わせだけで、ほぼあらゆるドメインのデータ構造を正確に表現できます。

---

## 5.2 直積型 — `struct` で表現する

直積型は「A **かつ** B」を表します。フィールドの値の組み合わせが型の値域を決めます。

`Point` が持てる値の数は `f64` の値域 × `f64` の値域 = 直積です。

```rust
/// 2次元座標を表す直積型
/// x と y の両方のフィールドを必ず持つ
#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

/// 長方形を表す直積型
/// origin と size の両方が必ず存在する
#[derive(Debug, Clone)]
struct Rectangle {
    origin: Point,
    width: f64,
    height: f64,
}

impl Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

fn main() {
    let rect = Rectangle {
        origin: Point { x: 0.0, y: 0.0 },
        width: 10.0,
        height: 5.0,
    };
    println!("面積: {}", rect.area()); // 50
}
```

タプル構造体（フィールド名なし）も直積型の一種です。

```rust
/// 単位を型で区別するためのタプル構造体
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Meters(f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Seconds(f64);

/// 速度 = 距離 / 時間（型が異なるので誤った演算をコンパイル時に防ぐ）
fn speed(distance: Meters, time: Seconds) -> f64 {
    distance.0 / time.0
}
```

---

## 5.3 直和型 — `enum` で表現する

直和型は「A **または** B **または** C」を表します。enum の各バリアントは独立した選択肢です。

```rust
/// 支払い方法を表す直和型
/// Cash, CreditCard, BankTransfer のいずれかひとつ
#[derive(Debug)]
enum PaymentMethod {
    Cash,
    CreditCard { number: String, expiry: String },
    BankTransfer { account_id: u64 },
}

fn describe_payment(method: &PaymentMethod) -> String {
    match method {
        PaymentMethod::Cash => "現金払い".to_string(),
        PaymentMethod::CreditCard { number, .. } => {
            format!("クレジットカード末尾 {}", &number[number.len() - 4..])
        }
        PaymentMethod::BankTransfer { account_id } => {
            format!("口座番号 {}", account_id)
        }
    }
}
```

直和型の強みは「存在しうるすべての状態を型として列挙できる」ことです。Rust の `match` は網羅性チェックを行うため、新しいバリアントを追加したときにコンパイラが未処理箇所を指摘してくれます。

---

## 5.4 `match` 式による網羅的パターンマッチング

`match` は Rust のパターンマッチングの中心機能です。すべてのバリアントを処理しないとコンパイルエラーになります（**網羅性（exhaustiveness）の保証**）。

```rust
#[derive(Debug)]
enum Direction {
    North,
    South,
    East,
    West,
}

fn opposite(dir: Direction) -> Direction {
    match dir {
        Direction::North => Direction::South,
        Direction::South => Direction::North,
        Direction::East  => Direction::West,
        Direction::West  => Direction::East,
        // すべてのバリアントを列挙しないとコンパイルエラー
    }
}
```

### 複数パターンのマッチ

`|` で複数のパターンをまとめて書けます。

```rust
fn is_weekend(day: &str) -> bool {
    match day {
        "Saturday" | "Sunday" => true,
        _ => false,
    }
}
```

### ガード条件（Match Guards）

`if` 条件を追加することで、パターンに追加の制約を課せます。

```rust
fn classify_temperature(celsius: f64) -> &'static str {
    match celsius {
        t if t < 0.0   => "氷点下",
        t if t < 10.0  => "寒い",
        t if t < 25.0  => "快適",
        t if t < 35.0  => "暑い",
        _              => "猛暑",
    }
}
```

### 値の束縛

`@` バインディングで、パターンにマッチした値を名前に束縛しながら条件を検査できます。

```rust
fn describe_number(n: i32) -> String {
    match n {
        0 => "ゼロ".to_string(),
        n @ 1..=9   => format!("1桁の正数: {}", n),
        n @ 10..=99 => format!("2桁の正数: {}", n),
        n if n < 0  => format!("負数: {}", n),
        n           => format!("大きな数: {}", n),
    }
}
```

---

## 5.5 ネストした `enum` のパターン分解

実際のコードでは enum がネストすることがよくあります。`match` は深いネストでも一度に分解できます。

```rust
#[derive(Debug)]
enum Color {
    Rgb(u8, u8, u8),
    Hsv { hue: f64, saturation: f64, value: f64 },
    Named(NamedColor),
}

#[derive(Debug)]
enum NamedColor {
    Red,
    Green,
    Blue,
    White,
    Black,
}

fn to_rgb(color: &Color) -> (u8, u8, u8) {
    match color {
        // タプルバリアントの分解
        Color::Rgb(r, g, b) => (*r, *g, *b),

        // 構造体バリアントの分解（不要なフィールドは .. で省略）
        Color::Hsv { hue, saturation, value } => {
            hsv_to_rgb(*hue, *saturation, *value)
        }

        // ネストした enum のパターン分解
        Color::Named(NamedColor::Red)   => (255, 0, 0),
        Color::Named(NamedColor::Green) => (0, 255, 0),
        Color::Named(NamedColor::Blue)  => (0, 0, 255),
        Color::Named(NamedColor::White) => (255, 255, 255),
        Color::Named(NamedColor::Black) => (0, 0, 0),
    }
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
    // 簡略実装（実際には詳細な変換が必要）
    let r = (v * (1.0 - s * (1.0 - (h / 60.0).fract()))) * 255.0;
    ((r as u8), (r as u8 / 2), (r as u8 / 4))
}
```

タプルを `match` することで、複数の値を同時にパターン分解することもできます。

```rust
/// 2つの Option を同時に分解する
fn merge_options(a: Option<i32>, b: Option<i32>) -> Option<i32> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x + y),
        (Some(x), None)    => Some(x),
        (None,    Some(y)) => Some(y),
        (None,    None)    => None,
    }
}
```

---

## 5.6 `if let` と `while let`

`match` は強力ですが、1つのパターンにだけ興味がある場合は **`if let`** の方が簡潔に書けます。

### `if let` — 1つのパターンにだけ処理する

```rust
fn print_even(n: Option<i32>) {
    // match を使った場合（冗長）
    match n {
        Some(x) if x % 2 == 0 => println!("偶数: {}", x),
        _ => {},
    }

    // if let を使った場合（簡潔）
    if let Some(x) = n {
        if x % 2 == 0 {
            println!("偶数: {}", x);
        }
    }
}
```

`if let` はパターンが合致しなかった場合の処理（`else` 節）も書けます。

```rust
#[derive(Debug)]
enum Config {
    File(String),
    Default,
}

fn load_config(config: &Config) -> String {
    if let Config::File(path) = config {
        format!("{}を読み込む", path)
    } else {
        "デフォルト設定を使用する".to_string()
    }
}
```

### `while let` — パターンが合致する間ループする

スタックやキューのポップ処理に自然に使えます。

```rust
fn process_stack() {
    let mut stack = vec![1, 2, 3, 4, 5];

    // スタックが空になるまで処理する
    while let Some(top) = stack.pop() {
        println!("処理中: {}", top);
    }
    println!("スタックが空になりました");
}
```

イテレータと組み合わせた例:

```rust
fn consume_while_positive(values: &mut std::collections::VecDeque<i32>) -> Vec<i32> {
    let mut result = Vec::new();

    while let Some(&front) = values.front() {
        if front > 0 {
            result.push(values.pop_front().unwrap());
        } else {
            break;
        }
    }

    result
}
```

---

## 5.7 再帰的なデータ型

再帰的なデータ型とは、型の定義の中に自分自身が含まれるデータ型です。関数型プログラミングにおいてリストやツリーなどの構造を表現するための基本パターンです。

Rust では再帰型をスタックに直接置くとサイズが決まらないため、`Box<T>` でヒープ上に間接参照します。

### 関数型リスト（Cons/Nil パターン）

Haskell などの関数型言語では連結リストを以下のように定義します。Rust でも同じ概念を `enum` で表現できます。

```rust
/// 関数型スタイルの連結リスト
/// Cons(head, tail) は先頭要素と残りのリストを保持する
/// Nil は空リストを表す
#[derive(Debug)]
enum List<T> {
    Cons(T, Box<List<T>>),
    Nil,
}

impl<T: std::fmt::Debug + Clone> List<T> {
    /// 空のリストを生成する
    fn empty() -> Self {
        List::Nil
    }

    /// 先頭に要素を追加した新しいリストを返す
    fn prepend(self, value: T) -> Self {
        List::Cons(value, Box::new(self))
    }

    /// リストの長さを返す（再帰実装）
    fn len(&self) -> usize {
        match self {
            List::Nil => 0,
            List::Cons(_, tail) => 1 + tail.len(),
        }
    }

    /// リストが空かどうかを返す
    fn is_empty(&self) -> bool {
        matches!(self, List::Nil)
    }

    /// リストを Vec に変換する
    fn to_vec(&self) -> Vec<T> {
        let mut result = Vec::new();
        let mut current = self;
        loop {
            match current {
                List::Nil => break,
                List::Cons(head, tail) => {
                    result.push(head.clone());
                    current = tail;
                }
            }
        }
        result
    }
}

impl List<i32> {
    /// 数値リストの合計を返す（再帰実装）
    fn sum(&self) -> i32 {
        match self {
            List::Nil => 0,
            List::Cons(head, tail) => head + tail.sum(),
        }
    }
}

fn main() {
    // Nil から始めて prepend で構築する
    let list = List::empty()
        .prepend(3)
        .prepend(2)
        .prepend(1);

    println!("長さ: {}", list.len());   // 3
    println!("合計: {}", list.sum());   // 6
    println!("内容: {:?}", list.to_vec()); // [1, 2, 3]
}
```

### 二分探索木（Binary Search Tree）

ツリーも再帰的なデータ型の典型例です。

```rust
/// 二分探索木のノード
#[derive(Debug)]
enum Tree<T> {
    Leaf,
    Node {
        value: T,
        left: Box<Tree<T>>,
        right: Box<Tree<T>>,
    },
}

impl<T: Ord + Clone> Tree<T> {
    /// 空のツリーを生成する
    fn empty() -> Self {
        Tree::Leaf
    }

    /// 値を挿入した新しいツリーを返す（不変スタイル）
    fn insert(self, new_value: T) -> Self {
        match self {
            Tree::Leaf => Tree::Node {
                value: new_value,
                left: Box::new(Tree::Leaf),
                right: Box::new(Tree::Leaf),
            },
            Tree::Node { value, left, right } => {
                if new_value < value {
                    Tree::Node {
                        value,
                        left: Box::new(left.insert(new_value)),
                        right,
                    }
                } else if new_value > value {
                    Tree::Node {
                        value,
                        left,
                        right: Box::new(right.insert(new_value)),
                    }
                } else {
                    // 重複は無視する
                    Tree::Node { value, left, right }
                }
            }
        }
    }

    /// 値が含まれるかどうかを調べる
    fn contains(&self, target: &T) -> bool {
        match self {
            Tree::Leaf => false,
            Tree::Node { value, left, right } => {
                if target == value {
                    true
                } else if target < value {
                    left.contains(target)
                } else {
                    right.contains(target)
                }
            }
        }
    }

    /// 中順（in-order）でソートされた Vec を返す
    fn to_sorted_vec(&self) -> Vec<T> {
        match self {
            Tree::Leaf => Vec::new(),
            Tree::Node { value, left, right } => {
                let mut result = left.to_sorted_vec();
                result.push(value.clone());
                result.extend(right.to_sorted_vec());
                result
            }
        }
    }
}

fn main() {
    let tree = Tree::empty()
        .insert(5)
        .insert(3)
        .insert(7)
        .insert(1)
        .insert(4);

    println!("3を含む: {}", tree.contains(&3)); // true
    println!("6を含む: {}", tree.contains(&6)); // false
    println!("ソート済み: {:?}", tree.to_sorted_vec()); // [1, 3, 4, 5, 7]
}
```

### 式の抽象構文木（AST）

ADT は言語処理系の式を表現するのにも使われます。

```rust
/// 算術式を表す再帰的なデータ型（抽象構文木）
#[derive(Debug, Clone)]
enum Expr {
    Num(f64),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
}

impl Expr {
    /// 式を評価して数値を返す
    fn eval(&self) -> Result<f64, String> {
        match self {
            Expr::Num(n) => Ok(*n),
            Expr::Add(l, r) => Ok(l.eval()? + r.eval()?),
            Expr::Sub(l, r) => Ok(l.eval()? - r.eval()?),
            Expr::Mul(l, r) => Ok(l.eval()? * r.eval()?),
            Expr::Div(l, r) => {
                let divisor = r.eval()?;
                if divisor == 0.0 {
                    Err("ゼロ除算エラー".to_string())
                } else {
                    Ok(l.eval()? / divisor)
                }
            }
        }
    }
}

fn main() {
    // (3 + 4) * 2 を表す AST
    let expr = Expr::Mul(
        Box::new(Expr::Add(
            Box::new(Expr::Num(3.0)),
            Box::new(Expr::Num(4.0)),
        )),
        Box::new(Expr::Num(2.0)),
    );

    match expr.eval() {
        Ok(result) => println!("結果: {}", result),  // 14
        Err(e)     => println!("エラー: {}", e),
    }
}
```

---

## まとめ

| 概念 | ポイント |
|------|----------|
| 直積型（`struct`） | 複数のフィールドを**すべて**持つ。値域は各フィールドの直積 |
| 直和型（`enum`） | 複数のバリアントの**いずれか**を持つ。型安全な条件分岐が可能 |
| `match` | 網羅性をコンパイル時に保証する。パターン分解・ガード条件が使える |
| ネストした enum | 深いネストでも一度に分解できる。タプルの `match` で複数値を同時処理 |
| `if let` | 1つのパターンだけ処理したい場合に簡潔に書ける |
| `while let` | パターンが合致する間繰り返す。スタック・キュー処理に便利 |
| 再帰型と `Box<T>` | 再帰的なデータ型はヒープを使って表現する。`Box<T>` で間接参照 |

ADT の最大の価値は「**表現できない状態を型として存在させない（Make Illegal States Unrepresentable）**」ことです。型設計の段階で不正な状態を排除することで、実行時エラーをコンパイル時のエラーに変換できます。

---

## 章末演習問題

### 問題 1

以下の `Shape` enum に `area()` と `perimeter()` メソッドを実装してください。`match` で各バリアントを処理してください。

```rust
use std::f64::consts::PI;

#[derive(Debug)]
enum Shape {
    Circle(f64),                    // radius
    Rectangle(f64, f64),            // width, height
    Triangle(f64, f64, f64),        // 3辺の長さ
}

impl Shape {
    fn area(&self) -> f64 {
        // TODO: 実装してください
        todo!()
    }

    fn perimeter(&self) -> f64 {
        // TODO: 実装してください
        todo!()
    }
}
```

<details>
<summary>解答</summary>

```rust
use std::f64::consts::PI;

impl Shape {
    fn area(&self) -> f64 {
        match self {
            Shape::Circle(r)           => PI * r * r,
            Shape::Rectangle(w, h)     => w * h,
            Shape::Triangle(a, b, c)   => {
                // ヘロンの公式
                let s = (a + b + c) / 2.0;
                (s * (s - a) * (s - b) * (s - c)).sqrt()
            }
        }
    }

    fn perimeter(&self) -> f64 {
        match self {
            Shape::Circle(r)           => 2.0 * PI * r,
            Shape::Rectangle(w, h)     => 2.0 * (w + h),
            Shape::Triangle(a, b, c)   => a + b + c,
        }
    }
}
```

</details>

---

### 問題 2

以下の `Message` enum に対して、`process` 関数を完成させてください。`if let` を使って `Quit` メッセージを処理し、残りは `match` で処理してください。

```rust
#[derive(Debug)]
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(u8, u8, u8),
}

/// メッセージを処理してログ文字列を返す
/// Quit が来たら None を返す
fn process(message: Message) -> Option<String> {
    // TODO: if let で Quit を先に処理し、残りを match で処理してください
    todo!()
}
```

<details>
<summary>解答</summary>

```rust
fn process(message: Message) -> Option<String> {
    if let Message::Quit = message {
        return None;
    }

    let log = match message {
        Message::Move { x, y }          => format!("移動: ({}, {})", x, y),
        Message::Write(text)             => format!("書き込み: {}", text),
        Message::ChangeColor(r, g, b)    => format!("色変更: rgb({}, {}, {})", r, g, b),
        Message::Quit                    => unreachable!(),
    };

    Some(log)
}
```

</details>

---

### 問題 3

以下の `List<T>` enum に `map` 関数を実装してください。各要素に関数を適用した新しいリストを返します。

```rust
#[derive(Debug)]
enum List<T> {
    Cons(T, Box<List<T>>),
    Nil,
}

impl<T> List<T> {
    fn map<U, F: Fn(T) -> U>(self, f: F) -> List<U> {
        // TODO: 再帰を使って実装してください
        todo!()
    }
}
```

ヒント: `match self` で `Cons` と `Nil` を分解し、`Cons` の場合は先頭要素に `f` を適用してから再帰します。

<details>
<summary>解答</summary>

```rust
impl<T> List<T> {
    fn map<U, F: Fn(T) -> U>(self, f: F) -> List<U> {
        match self {
            List::Nil => List::Nil,
            List::Cons(head, tail) => List::Cons(
                f(head),
                Box::new(tail.map(f)),
            ),
        }
    }
}
```

`F: Fn(T) -> U` ではなく `F: Fn(&T) -> U` にすると所有権を消費しない `map` になります。用途に応じて設計を検討してください。

</details>
