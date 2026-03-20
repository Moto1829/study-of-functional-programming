# 第1章: 関数型プログラミングの基礎概念

## 1.1 関数型プログラミングとは何か

関数型プログラミング（Functional Programming、以下FP）は、**計算を数学的な関数の評価として表現する**プログラミングパラダイムです。

FPの中心的な考え方は次の2点です。

- **関数を第一級市民（First-class citizen）として扱う**: 関数を変数に代入したり、他の関数の引数や戻り値として使える
- **状態の変更と副作用を避ける**: データは変更するのではなく、新しいデータを生成することで変換する

Rustは純粋な関数型言語ではありませんが、クロージャ、イテレータ、`Option`/`Result` 型など、関数型スタイルを強力にサポートする機能を多数備えています。

---

## 1.2 命令型プログラミングとの違い

**命令型プログラミング**は「どのように（How）」処理するかを手順として記述します。変数の状態を順に変化させながら結果を得ます。

**関数型プログラミング**は「何を（What）」計算するかを宣言的に記述します。データをどのように変換するかを式で表現します。

同じ処理（1から10の偶数の合計）を2つのスタイルで比較してみましょう。

```rust
/// 命令型スタイル: ループで状態を変更しながら合計を求める
fn sum_evens_imperative(numbers: &[i32]) -> i32 {
    let mut total = 0;          // 可変な状態
    for &n in numbers {         // 手続きとして逐次処理
        if n % 2 == 0 {
            total += n;         // 状態を変更
        }
    }
    total
}

/// 関数型スタイル: イテレータで変換を宣言的に記述する
fn sum_evens_functional(numbers: &[i32]) -> i32 {
    numbers
        .iter()
        .filter(|&&n| n % 2 == 0)   // 偶数のみを取り出す変換
        .sum()                       // 合計を宣言
}

fn main() {
    let numbers: Vec<i32> = (1..=10).collect();

    let result_imp = sum_evens_imperative(&numbers);
    let result_fun = sum_evens_functional(&numbers);

    println!("命令型: {}", result_imp); // 30
    println!("関数型: {}", result_fun); // 30
}
```

関数型スタイルのコードは「偶数でフィルタして合計する」という意図が直接コードに現れており、**可読性と意図の明確さ**が向上しています。

---

## 1.3 純粋関数（Pure Functions）

純粋関数とは、以下の2つの性質を持つ関数です。

1. **同じ引数には常に同じ戻り値を返す**（決定論的）
2. **副作用を持たない**（外部の状態を読み書きしない）

```rust
/// 純粋関数の例: 同じ x, y には必ず同じ結果を返す
/// 外部の状態に一切依存しない
fn add(x: i32, y: i32) -> i32 {
    x + y
}

/// 純粋関数の例: 円の面積を計算する
/// PIは定数なので外部状態ではない
fn circle_area(radius: f64) -> f64 {
    std::f64::consts::PI * radius * radius
}

/// 純粋関数の例: リストの各要素を2倍にした新しいリストを返す
/// 元のリストは変更しない
fn double_all(numbers: &[i32]) -> Vec<i32> {
    numbers.iter().map(|&n| n * 2).collect()
}
```

一方、**不純な関数**は外部状態に依存したり、副作用を持ちます。

```rust
use std::sync::atomic::{AtomicI32, Ordering};

static COUNTER: AtomicI32 = AtomicI32::new(0);

/// 不純な関数の例: グローバルカウンターに依存しているため、
/// 同じ引数でも呼び出すたびに異なる値を返す可能性がある
fn add_with_counter(x: i32) -> i32 {
    let count = COUNTER.fetch_add(1, Ordering::SeqCst);
    x + count
}

/// 不純な関数の例: I/O という副作用を持つ
fn print_and_return(x: i32) -> i32 {
    println!("値: {}", x); // 標準出力への書き込みは副作用
    x
}
```

純粋関数を使うと、以下のメリットが得られます。

- **テストが容易**: 外部状態の準備が不要
- **推論しやすい**: 関数の入出力だけを考えればよい
- **並列化しやすい**: 状態の競合が起きない

---

## 1.4 参照透過性（Referential Transparency）

参照透過性とは「**式をその計算結果（値）で置き換えても、プログラムの意味が変わらない**」性質です。

純粋関数は参照透過性を保証します。

```rust
/// 純粋関数: 参照透過性がある
fn square(x: i32) -> i32 {
    x * x
}

fn main() {
    // square(5) は 25 という値で置き換えられる
    // 以下の2行は等価
    let a = square(5) + square(5); // 式のまま
    let b = 25 + 25;               // 値に置換

    assert_eq!(a, b); // 常に成立する

    // さらに式の展開も可能
    let c = square(5) * 2;
    let d = 25 * 2;
    assert_eq!(c, d);
}
```

参照透過性があると、コンパイラによる最適化（共通部分式の削除、メモ化など）が可能になります。また、コードを読む人間も局所的な推論ができるため、バグを発見しやすくなります。

不純な関数では参照透過性が失われます。

```rust
fn get_timestamp() -> u64 {
    // 呼び出すたびに異なる値を返す: 参照透過でない
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn main() {
    let t1 = get_timestamp();
    let t2 = get_timestamp();
    // t1 と t2 は必ずしも等しくない
    // get_timestamp() を t1 の値で置き換えると意味が変わる
}
```

---

## 1.5 副作用（Side Effects）とその管理

副作用（Side Effects）とは、関数がその戻り値以外に外部の状態に影響を与えることです。

**代表的な副作用**:
- 標準出力・ファイルへの書き込み（I/O）
- データベースの読み書き
- グローバル変数・共有状態の変更
- 例外・パニックの発生

副作用は避けられませんが（画面表示もI/Oも必要）、**純粋なコアロジックと副作用を持つ処理を明確に分離する**ことで管理できます。

```rust
// ---- 純粋なコアロジック（副作用なし）----

/// 文字列が回文かどうかを判定する（純粋関数）
fn is_palindrome(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    let reversed: Vec<char> = chars.iter().rev().cloned().collect();
    chars == reversed
}

/// 単語リストから回文のみを抽出する（純粋関数）
fn filter_palindromes(words: &[&str]) -> Vec<&str> {
    words.iter().copied().filter(|w| is_palindrome(w)).collect()
}

// ---- 副作用を持つ外側の処理 ----

/// 回文を標準出力に表示する（副作用あり）
/// 純粋関数 filter_palindromes を呼び出し、結果を表示するだけ
fn print_palindromes(words: &[&str]) {
    let palindromes = filter_palindromes(words); // 純粋な計算
    // I/O は外側に閉じ込める
    for word in &palindromes {
        println!("回文: {}", word);
    }
}

fn main() {
    let words = vec!["level", "hello", "radar", "world", "civic"];
    print_palindromes(&words);
}
```

このパターンでは「回文を探す」ロジックは純粋関数として切り出されているため、I/O とは独立してテストできます。

---

## まとめ

| 概念 | ポイント |
|------|----------|
| 関数型プログラミング | 計算を関数の評価として表現するパラダイム |
| 命令型との違い | How（手続き）ではなく What（宣言）で記述 |
| 純粋関数 | 同じ入力→同じ出力、副作用なし |
| 参照透過性 | 式を値に置き換えても意味が変わらない性質 |
| 副作用の管理 | 純粋なコアと副作用の境界を明確に分ける |

---

## よくある落とし穴と対処法

### 落とし穴1: 副作用を持つ関数を「純粋」だと思い込む

```rust
// NG: 呼び出すたびに結果が変わる（副作用あり）
fn current_time_greeting(name: &str) -> String {
    let now = std::time::SystemTime::now(); // 副作用！
    format!("{:?}: Hello, {}", now, name)
}

// OK: 副作用を引数として受け取る（純粋関数）
fn greet_with_time(name: &str, timestamp: u64) -> String {
    format!("[{}]: Hello, {}", timestamp, name)
}
```

**対処法:** 外部依存（時刻・乱数・I/O）は引数として受け取るか、呼び出し側で処理する。

### 落とし穴2: 参照透過性を破る隠れた状態

```rust
use std::cell::Cell;
thread_local! { static COUNTER: Cell<i32> = Cell::new(0); }

// NG: スレッドローカルな状態に依存している
fn next_id() -> i32 {
    COUNTER.with(|c| { c.set(c.get() + 1); c.get() })
}
```

**対処法:** 状態を関数の引数/戻り値で明示的に渡す。

---

## 章末演習問題

### 問題 1

以下の関数のうち、純粋関数はどれですか？理由も述べてください。

```rust
fn multiply(x: i32, y: i32) -> i32 { x * y }

fn read_line() -> String {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input
}

fn factorial(n: u64) -> u64 {
    if n == 0 { 1 } else { n * factorial(n - 1) }
}
```

<details>
<summary>解答</summary>

`multiply` と `factorial` は純粋関数です。同じ引数には常に同じ値を返し、外部状態を変更しません。

`read_line` は不純です。標準入力という外部状態に依存しており、呼び出すたびに異なる値を返す可能性があります。

</details>

---

### 問題 2

以下の命令型の関数を、関数型スタイル（`iter()` のメソッドチェーン）で書き直してください。

```rust
fn sum_of_squares_imperative(numbers: &[i32]) -> i32 {
    let mut result = 0;
    for &n in numbers {
        result += n * n;
    }
    result
}
```

<details>
<summary>解答</summary>

```rust
fn sum_of_squares_functional(numbers: &[i32]) -> i32 {
    numbers.iter().map(|&n| n * n).sum()
}
```

</details>

---

### 問題 3

次のコードは参照透過ではありません。なぜですか？また、参照透過にするにはどう修正すればよいでしょうか？

```rust
fn greet(name: &str) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("[{}] Hello, {}!", now, name)
}
```

<details>
<summary>解答</summary>

`greet("Alice")` は呼び出すたびに異なるタイムスタンプを含む文字列を返します。そのため、`greet("Alice")` という式を特定の値で置き換えることができず、参照透過ではありません。

修正方法: タイムスタンプを引数として受け取ることで純粋関数にする。

```rust
fn greet(name: &str, timestamp: u64) -> String {
    format!("[{}] Hello, {}!", timestamp, name)
}
```

こうすると、`greet("Alice", 1000)` は常に `"[1000] Hello, Alice!"` を返すため参照透過になります。タイムスタンプの取得（副作用）は呼び出し側に委ねます。

</details>
