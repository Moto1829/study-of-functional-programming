# 第7章: 関数合成とコンビネータパターン

## はじめに

関数型プログラミングの強みのひとつは、**小さな関数を組み合わせて大きな処理を構築できる**ことです。本章では「関数合成（Function Composition）」の数学的背景から出発し、Rustでの実装方法、そして実用的な「コンビネータパターン」まで体系的に学びます。

---

## 1. 関数合成の数学的背景

数学では、2つの関数 `f: A → B` と `g: B → C` があるとき、**合成関数（composite function）** `g ∘ f` を次のように定義します。

```
(g ∘ f)(x) = g(f(x))
```

つまり「まず `f` を適用し、その結果に `g` を適用する」という操作の組み合わせです。

合成関数には次の性質があります。

- **結合律（Associativity）**: `(h ∘ g) ∘ f = h ∘ (g ∘ f)`
  どの順で合成しても同じ結果になります。
- **恒等関数（Identity）**: 恒等関数 `id(x) = x` に対して `f ∘ id = id ∘ f = f`
  合成しても関数は変わりません。

この性質が成り立つ構造を数学では**圏（Category）** と呼びます。関数型プログラミングの理論的基盤「圏論（Category Theory）」に繋がる考え方です。

---

## 2. Rustで `compose` / `pipe` を実現する

Rustでは関数を値として扱える（クロージャ、関数ポインタ）ため、関数合成を関数として表現できます。

### 2.1 `compose` 関数

数学の `g ∘ f` に対応する実装です。引数の順は「右から左」になります。

```rust
/// `compose(f, g)` は `|x| g(f(x))` を返す
/// 数学の g ∘ f に対応: まず f を適用し、次に g を適用する
fn compose<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> C
where
    F: Fn(A) -> B,
    G: Fn(B) -> C,
{
    move |x| g(f(x))
}

fn main() {
    let add_one = |x: i32| x + 1;
    let double = |x: i32| x * 2;

    // compose(add_one, double) = double ∘ add_one
    // x=3 のとき: add_one(3)=4, double(4)=8
    let add_then_double = compose(add_one, double);
    println!("{}", add_then_double(3)); // 8
}
```

### 2.2 `pipe` 関数

`pipe` は `compose` と引数の意味が同じですが、**左から右**に読める点が異なります。データの流れを可視化する際に自然な表現です。

```rust
/// `pipe(f, g)` は `|x| g(f(x))` を返す
/// データが f → g の順に流れることを強調する命名
fn pipe<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> C
where
    F: Fn(A) -> B,
    G: Fn(B) -> C,
{
    move |x| g(f(x))
}

fn main() {
    let trim = |s: &str| s.trim().to_string();
    let to_upper = |s: String| s.to_uppercase();

    // "  hello  " -> trim -> to_upper -> "HELLO"
    let normalize = pipe(trim, to_upper);
    println!("{}", normalize("  hello  ")); // "HELLO"
}
```

### 2.3 3つ以上の関数を合成するマクロ

Rustのマクロを使うと、任意個数の関数を左から右に連鎖させるパイプラインを簡潔に記述できます。

```rust
/// 複数の関数を左から右に合成するマクロ
/// pipe_all!(f, g, h) は |x| h(g(f(x))) と等価
macro_rules! pipe_all {
    ($f:expr) => { $f };
    ($f:expr, $($rest:expr),+) => {
        {
            let f = $f;
            let rest = pipe_all!($($rest),+);
            move |x| rest(f(x))
        }
    };
}

fn main() {
    let step1 = |x: i32| x + 1;   // +1
    let step2 = |x: i32| x * 2;   // *2
    let step3 = |x: i32| x - 3;   // -3

    // (3 + 1) * 2 - 3 = 5
    let pipeline = pipe_all!(step1, step2, step3);
    println!("{}", pipeline(3)); // 5
}
```

---

## 3. コンビネータパターンの概念

**コンビネータ（Combinator）** とは、「既存の値や関数を受け取り、新しい値や関数を返す高階関数」のことです。

Rustの標準ライブラリにはすでに多くのコンビネータが組み込まれています。

| コンビネータ | 対象 | 説明 |
|---|---|---|
| `Option::map` | `Option<T>` | `Some(x)` なら中の値を変換する |
| `Option::and_then` | `Option<T>` | `Some(x)` なら別の `Option` を返す関数を適用する（モナディックバインド） |
| `Result::map_err` | `Result<T, E>` | `Err(e)` のエラー型を変換する |
| `Iterator::filter_map` | `Iterator` | 変換と絞り込みを同時に行う |

コンビネータの特徴は次の3点です。

1. **合成可能**: コンビネータ同士を組み合わせて複雑な処理を構築できる
2. **再利用可能**: 小さく汎用的なため、様々な文脈で使い回せる
3. **宣言的**: 「何をするか」を表現し、「どうやるか」を隠蔽できる

```rust
fn process_input(input: Option<&str>) -> Option<u32> {
    input
        .map(|s| s.trim())              // 空白除去
        .filter(|s| !s.is_empty())      // 空文字排除
        .and_then(|s| s.parse().ok())   // 数値変換（失敗したら None）
        .map(|n: u32| n * 2)            // 2倍にする
}

fn main() {
    println!("{:?}", process_input(Some("  21  "))); // Some(42)
    println!("{:?}", process_input(Some("  "))); // None（空文字）
    println!("{:?}", process_input(Some("abc"))); // None（数値変換失敗）
    println!("{:?}", process_input(None)); // None
}
```

---

## 4. Builderパターンをコンビネータで実装する

**Builderパターン**は、多くのオプションを持つ構造体を段階的に構築するためのパターンです。コンビネータとして設計することで、メソッドチェーンが可能になり、宣言的な設定記述が実現します。

```rust
/// SQLのSELECTクエリを段階的に構築する Builder
#[derive(Debug, Default)]
pub struct QueryBuilder {
    table: String,
    columns: Vec<String>,
    conditions: Vec<String>,
    limit: Option<usize>,
    order_by: Option<String>,
}

impl QueryBuilder {
    /// 新しい QueryBuilder を生成する
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            ..Default::default()
        }
    }

    /// SELECTするカラムを追加する（コンビネータ: self を消費して新しい self を返す）
    pub fn select(mut self, column: impl Into<String>) -> Self {
        self.columns.push(column.into());
        self
    }

    /// WHERE 条件を追加する
    pub fn where_clause(mut self, condition: impl Into<String>) -> Self {
        self.conditions.push(condition.into());
        self
    }

    /// 取得件数の上限を設定する
    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    /// ORDER BY を設定する
    pub fn order_by(mut self, column: impl Into<String>) -> Self {
        self.order_by = Some(column.into());
        self
    }

    /// SQL文字列を生成する
    pub fn build(self) -> String {
        let columns = if self.columns.is_empty() {
            "*".to_string()
        } else {
            self.columns.join(", ")
        };

        let mut sql = format!("SELECT {} FROM {}", columns, self.table);

        if !self.conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.conditions.join(" AND "));
        }

        if let Some(order) = self.order_by {
            sql.push_str(&format!(" ORDER BY {}", order));
        }

        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        sql
    }
}

fn main() {
    let query = QueryBuilder::new("users")
        .select("id")
        .select("name")
        .select("email")
        .where_clause("age > 18")
        .where_clause("active = true")
        .order_by("name")
        .limit(10)
        .build();

    // SELECT id, name, email FROM users WHERE age > 18 AND active = true ORDER BY name LIMIT 10
    println!("{}", query);
}
```

このパターンのポイントは、各メソッドが `self` を受け取り、変更した `self` を返す点です。これにより値の消費と生成が繰り返され、型チェッカーが構築中の状態を追跡できます。

---

## 5. 簡単なパーサーコンビネータ

パーサーコンビネータは「コンビネータパターンをパーサーに応用した」設計です。基本的なパーサーを小さな単位として定義し、それらを組み合わせることで複雑な構文を解析します。

### 5.1 パーサーの型

パーサーを「入力文字列を受け取り、残りの文字列と解析結果のタプルを返す関数」として定義します。

```rust
/// パーサーの結果型
/// Ok((残りの入力, 解析結果)), Err(エラーメッセージ)
type ParseResult<'a, T> = Result<(&'a str, T), String>;

/// パーサーを表す型エイリアス (関数ポインタとして使う)
/// 実際の実装ではトレイトオブジェクトやジェネリクスを使うことが多い
```

### 5.2 基本パーサー

```rust
/// 特定の文字1つにマッチする基本パーサー
fn char_parser(expected: char) -> impl Fn(&str) -> ParseResult<char> {
    move |input: &str| {
        let mut chars = input.chars();
        match chars.next() {
            Some(c) if c == expected => {
                // 先頭1文字を消費し、残りの文字列と結果を返す
                let rest = &input[c.len_utf8()..];
                Ok((rest, c))
            }
            Some(c) => Err(format!("expected '{}', got '{}'", expected, c)),
            None => Err(format!("expected '{}', got end of input", expected)),
        }
    }
}

/// 指定した条件を満たす文字1つにマッチするパーサー
fn satisfy(predicate: impl Fn(char) -> bool) -> impl Fn(&str) -> ParseResult<char> {
    move |input: &str| {
        let mut chars = input.chars();
        match chars.next() {
            Some(c) if predicate(c) => {
                let rest = &input[c.len_utf8()..];
                Ok((rest, c))
            }
            Some(c) => Err(format!("unexpected char '{}'", c)),
            None => Err("unexpected end of input".to_string()),
        }
    }
}

fn main() {
    let parse_a = char_parser('a');
    println!("{:?}", parse_a("abc")); // Ok(("bc", 'a'))
    println!("{:?}", parse_a("xyz")); // Err("expected 'a', got 'x'")
}
```

### 5.3 `and_then` コンビネータで2つのパーサーを順に組み合わせる

```rust
/// 2つのパーサーを順に適用し、両方の結果をタプルで返すコンビネータ
fn and_then_parser<A, B, PA, PB>(pa: PA, pb: PB) -> impl Fn(&str) -> ParseResult<(A, B)>
where
    PA: Fn(&str) -> ParseResult<A>,
    PB: Fn(&str) -> ParseResult<B>,
{
    move |input: &str| {
        // まず pa を試みる
        let (rest, a) = pa(input)?;
        // 残りの入力に pb を試みる
        let (rest2, b) = pb(rest)?;
        Ok((rest2, (a, b)))
    }
}

/// パーサーに変換関数を適用するコンビネータ (Functor の map に相当)
fn map_parser<A, B, P, F>(parser: P, f: F) -> impl Fn(&str) -> ParseResult<B>
where
    P: Fn(&str) -> ParseResult<A>,
    F: Fn(A) -> B,
{
    move |input: &str| {
        let (rest, a) = parser(input)?;
        Ok((rest, f(a)))
    }
}

fn main() {
    let digit = satisfy(|c| c.is_ascii_digit());
    let letter = satisfy(|c| c.is_ascii_alphabetic());

    // "1a..." -> ('1', 'a') を解析する
    let digit_then_letter = and_then_parser(digit, letter);
    println!("{:?}", digit_then_letter("1abc")); // Ok(("bc", ('1', 'a')))
    println!("{:?}", digit_then_letter("12bc")); // Err("unexpected char '2'")

    // 結果を文字列に変換するコンビネータ合成
    let pair_as_string = map_parser(
        and_then_parser(char_parser('a'), char_parser('b')),
        |(a, b)| format!("{}{}", a, b),
    );
    println!("{:?}", pair_as_string("abcd")); // Ok(("cd", "ab"))
}
```

実際のパーサーコンビネータライブラリ（`nom`、`pest`、`chumsky` など）もこの原理に基づいて構築されており、これらの基本パーサーを使うことで JSON や独自DSLのパーサーを宣言的に記述できます。

---

## 6. NewTypeパターンによる型安全なドメインモデリング

**NewTypeパターン**は、既存の型を新しい型でラップして**意味上の区別**を型レベルで表現するパターンです。これにより「同じ型の値を誤って渡してしまう」バグをコンパイル時に検出できます。

### 6.1 基本的な NewType

```rust
/// ユーザーIDを表す型 (生の u64 とは別の型として扱われる)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(pub u64);

/// 投稿IDを表す型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PostId(pub u64);

/// 以下の誤用はコンパイルエラーになる
fn get_post(user_id: UserId, post_id: PostId) -> String {
    format!("user={}, post={}", user_id.0, post_id.0)
}

fn main() {
    let uid = UserId(42);
    let pid = PostId(100);

    println!("{}", get_post(uid, pid)); // OK

    // get_post(pid, uid); // コンパイルエラー: 型が異なる
    // get_post(42, 100);  // コンパイルエラー: u64 は UserId ではない
}
```

### 6.2 バリデーション付き NewType

NewTypeのコンストラクタをプライベートにし、スマートコンストラクタ（検証済みのものだけを構築できる）として設計できます。

```rust
/// 検証済みのメールアドレスを表す型
/// フィールドは非公開にし、`new` 経由でのみ生成できる
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);

#[derive(Debug, PartialEq)]
pub enum EmailError {
    Empty,
    MissingAtSign,
    MissingDomain,
}

impl Email {
    /// メールアドレスを検証してから生成する
    /// 不正な形式の場合は Err を返す
    pub fn new(raw: impl Into<String>) -> Result<Self, EmailError> {
        let s = raw.into();

        if s.is_empty() {
            return Err(EmailError::Empty);
        }

        let at_pos = s.find('@').ok_or(EmailError::MissingAtSign)?;

        if at_pos + 1 >= s.len() {
            return Err(EmailError::MissingDomain);
        }

        Ok(Email(s))
    }

    /// 内部の文字列への参照を返す
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn main() {
    match Email::new("user@example.com") {
        Ok(email) => println!("有効: {}", email),
        Err(e) => println!("エラー: {:?}", e),
    }

    match Email::new("invalid-email") {
        Ok(email) => println!("有効: {}", email),
        Err(e) => println!("エラー: {:?}", e), // MissingAtSign
    }
}
```

### 6.3 NewTypeと関数合成の組み合わせ

NewTypeを関数合成と組み合わせると、型安全なデータ変換パイプラインが構築できます。

```rust
#[derive(Debug, Clone)]
pub struct RawInput(pub String);

#[derive(Debug, Clone)]
pub struct TrimmedInput(pub String);

#[derive(Debug, Clone)]
pub struct NormalizedInput(pub String);

fn trim_input(raw: RawInput) -> TrimmedInput {
    TrimmedInput(raw.0.trim().to_string())
}

fn normalize_input(trimmed: TrimmedInput) -> NormalizedInput {
    NormalizedInput(trimmed.0.to_lowercase())
}

fn main() {
    let raw = RawInput("  Hello World  ".to_string());

    // 各ステップで型が変わるため、順序の誤りはコンパイルエラーになる
    let trimmed = trim_input(raw);
    let normalized = normalize_input(trimmed);

    println!("{}", normalized.0); // "hello world"

    // normalize_input(raw); // コンパイルエラー: RawInput を NormalizedInput に渡せない
}
```

---

## まとめ

本章では次の概念を学びました。

| 概念 | 要点 |
|---|---|
| 関数合成 | `compose(f, g)` = `g ∘ f`。小さな関数を組み合わせて大きな処理を構築する |
| `pipe` | 左から右に読める合成スタイル。データの流れを表現しやすい |
| コンビネータ | 高階関数で「処理の組み合わせ方」を抽象化する |
| Builderパターン | コンビネータとしてのメソッドチェーン。宣言的な構築を実現する |
| パーサーコンビネータ | 基本パーサーを `and_then` 等で組み合わせて複雑な構文を解析する |
| NewTypeパターン | 型ラッパーで意味的な区別をコンパイル時に保証する |

---

## 章末演習問題

### 演習1: 数値変換パイプラインの構築

次の3つの処理を `compose` または `pipe` を使って合成し、1つの関数として実装してください。

1. `f1`: 整数 `n` を受け取り、`n * n`（平方）を返す
2. `f2`: 整数 `n` を受け取り、`n + 100` を返す
3. `f3`: 整数 `n` を受け取り、その文字列表現 `"n"` を返す

`pipeline(5)` が `"125"` を返すことを確認してください（5² = 25、25 + 100 = 125）。

```rust
fn main() {
    let f1 = |n: i32| n * n;
    let f2 = |n: i32| n + 100;
    let f3 = |n: i32| n.to_string();

    // TODO: f1, f2, f3 を合成した pipeline を作る
    // let pipeline = ...;
    // assert_eq!(pipeline(5), "125");
}
```

### 演習2: バリデーション付き NewType の拡張

以下の NewType を実装してください。

- `PositiveInt(i32)`: 正の整数のみ受け付ける（0以下はエラー）
- `BoundedString { value: String, max_len: usize }` ではなく、`ShortString(String)`: 100文字以内の文字列のみ受け付ける

両者を組み合わせた構造体 `UserProfile { age: PositiveInt, bio: ShortString }` を定義し、バリデーションが正しく機能することをテストしてください。

```rust
// TODO: PositiveInt, ShortString, UserProfile を実装する
```

### 演習3: パーサーコンビネータの拡張

本章で実装したパーサーコンビネータを拡張して、次のコンビネータを実装してください。

1. `or_else_parser(pa, pb)`: `pa` が失敗したら `pb` を試みる（選択コンビネータ）
2. `many_parser(p)`: パーサー `p` を0回以上繰り返し、結果を `Vec` で返す

実装できたら、`many_parser(digit)` が `"123abc"` に対して `Ok(("abc", vec!['1', '2', '3']))` を返すことを確認してください。

```rust
// TODO: or_else_parser と many_parser を実装する
```
