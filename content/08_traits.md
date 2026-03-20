# 第8章: トレイトと型クラス的パターン

## 8.1 トレイトと Haskell の型クラスの対応関係

Haskell の**型クラス**（type class）は「ある型に対して特定の操作を保証する」仕組みです。
Rust の**トレイト**（trait）は同じ目的を果たします。2つの言語で並べると対応がよくわかります。

| Haskell の型クラス | Rust のトレイト |
|--------------------|-----------------|
| `class Functor f where ...` | `trait Functor<A> { ... }` |
| インスタンス宣言 `instance Functor Maybe` | `impl Functor<A> for Maybe<A>` |
| 型クラス制約 `(Functor f) =>` | トレイト境界 `F: Functor<A>` |
| デフォルトメソッド | トレイトのデフォルト実装 |
| 派生 `deriving (Show, Eq)` | derive マクロ `#[derive(Debug, PartialEq)]` |

Haskell の型クラスと Rust のトレイトには以下の違いもあります。

- Haskell は高カインド型（Higher-Kinded Types, HKT）を直接サポートするため、`fmap :: (a -> b) -> f a -> f b` のように `f` を型コンストラクタとして扱える
- Rust は HKT を直接サポートしないため、関連型（associated types）やジェネリクスで代用する工夫が必要
- Haskell は型推論が強力で制約を省略できることが多いが、Rust は明示的な記述が求められる

---

## 8.2 Functor 的なパターン: fmap を Rust のトレイトで表現する

### Haskell の Functor

```haskell
class Functor f where
  fmap :: (a -> b) -> f a -> f b

-- Maybe への実装
instance Functor Maybe where
  fmap f (Just x) = Just (f x)
  fmap _ Nothing  = Nothing
```

### Rust での表現

Rust で同じ抽象を作るには「コンテナの中身を変換して、別の型のコンテナを返す」ことを
関連型 `Output<B>` で表現します。

```rust
/// コンテナ内の値を関数で変換する能力を抽象化するトレイト。
pub trait Functor<A> {
    /// 変換後のコンテナ型。
    /// 例: Maybe<A> に fmap を適用すると Maybe<B> が返るため Output = Maybe<B>
    type Output<B>;

    /// コンテナ内の値に関数 f を適用し、新しいコンテナを返す。
    fn fmap<B, F>(self, f: F) -> Self::Output<B>
    where
        F: Fn(A) -> B;
}
```

独自の `Maybe<T>` 型を定義します。

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Maybe<T> {
    Just(T),
    Nothing,
}
```

`Maybe<A>` への `Functor` 実装は以下のようになります。

```rust
impl<A> Functor<A> for Maybe<A> {
    type Output<B> = Maybe<B>;

    fn fmap<B, F>(self, f: F) -> Maybe<B>
    where
        F: Fn(A) -> B,
    {
        match self {
            Maybe::Just(a) => Maybe::Just(f(a)),
            Maybe::Nothing => Maybe::Nothing,
        }
    }
}
```

使い方は以下の通りです。

```rust
let result = Maybe::Just(5).fmap(|x| x * 3);
assert_eq!(result, Maybe::Just(15));

let nothing: Maybe<i32> = Maybe::Nothing;
assert_eq!(nothing.fmap(|x| x * 3), Maybe::Nothing);

// 型を変換することもできる
let label = Maybe::Just(42).fmap(|x| format!("value={}", x));
assert_eq!(label, Maybe::Just("value=42".to_string()));
```

### ファンクター則

正しい `Functor` 実装は 2 つの法則を満たす必要があります。

**同一性則**: `fmap(id) == id`

```rust
let m = Maybe::Just(42);
assert_eq!(m.clone().fmap(|x| x), m); // fmap(id) は何も変えない
```

**合成則**: `fmap(g ∘ f) == fmap(g) ∘ fmap(f)`

```rust
let m = Maybe::Just(3);
let f = |x: i32| x + 1;
let g = |x: i32| x * 2;

let composed = m.clone().fmap(|x| g(f(x)));  // fmap(g ∘ f)
let chained  = m.fmap(f).fmap(g);             // fmap(g) ∘ fmap(f)

assert_eq!(composed, chained); // どちらも Just(8)
```

---

## 8.3 Iterator を Monad 的に読み解く

Haskell のリストモナドでは、`>>=`（bind）は「各要素に関数を適用し、結果のリストをすべて繋げる」操作です。

```haskell
[1, 2, 3] >>= \x -> [x, x * 10]
-- => [1, 10, 2, 20, 3, 30]
```

Rust の `Iterator::flat_map` はまさにこれに対応します。

```rust
let result: Vec<i32> = vec![1, 2, 3]
    .into_iter()
    .flat_map(|x| vec![x, x * 10])
    .collect();

assert_eq!(result, vec![1, 10, 2, 20, 3, 30]);
```

Haskell の `do` 記法で書いたコードと Rust のイテレータ連鎖を対比してみましょう。

```haskell
-- Haskell: リストモナドの do 記法
pairs :: [(Int, Int)]
pairs = do
  x <- [1, 2, 3]       -- bind: [1,2,3] >>= \x -> ...
  y <- [10, 20]        -- bind: [10,20] >>= \y -> ...
  return (x, y)        -- wrap: [(x, y)]
-- => [(1,10),(1,20),(2,10),(2,20),(3,10),(3,20)]
```

```rust
// Rust: flat_map による等価な表現
let pairs: Vec<(i32, i32)> = vec![1, 2, 3]
    .into_iter()
    .flat_map(|x| vec![10, 20].into_iter().map(move |y| (x, y)))
    .collect();

assert_eq!(
    pairs,
    vec![(1, 10), (1, 20), (2, 10), (2, 20), (3, 10), (3, 20)]
);
```

この対応を汎用関数として切り出すと次のようになります。

```rust
/// Iterator::flat_map をリストモナドの >>= として使う例。
pub fn flat_map_example<A, B, F>(xs: &[A], f: F) -> Vec<B>
where
    A: Copy,
    F: Fn(A) -> Vec<B>,
{
    xs.iter().copied().flat_map(f).collect()
}

// 使用例: 各整数 n を 1..=n の Vec に展開する
let result = flat_map_example(&[1, 2, 3], |x| (1..=x).collect::<Vec<_>>());
// [1] ++ [1,2] ++ [1,2,3] = [1, 1, 2, 1, 2, 3]
assert_eq!(result, vec![1, 1, 2, 1, 2, 3]);
```

---

## 8.4 Option / Result の and_then を Monad の bind として理解する

### Option::and_then は Monad の bind

`Option::and_then` は `Some` なら関数を適用し `None` をそのまま伝播させます。
これはまさに Monad の `bind`（`>>=`）の振る舞いです。

```rust
fn safe_div(x: i32, y: i32) -> Option<i32> {
    if y == 0 { None } else { Some(x / y) }
}

fn safe_sqrt(x: i32) -> Option<f64> {
    if x < 0 { None } else { Some((x as f64).sqrt()) }
}

// and_then を連鎖させる = do 記法の bind を並べる
let result = Some(100)
    .and_then(|x| safe_div(x, 4))   // Some(25)
    .and_then(|x| safe_sqrt(x));    // Some(5.0)

assert_eq!(result, Some(5.0));

// 途中で None になるとそこで止まる
let failure = Some(100)
    .and_then(|x| safe_div(x, 0))   // None (ゼロ除算)
    .and_then(|x| safe_sqrt(x));    // None のまま伝播

assert_eq!(failure, None);
```

### Result::and_then はエラー情報付きの bind

`Result::and_then` はエラー情報を持ちながら同じ連鎖を実現します。

```rust
fn parse_number(s: &str) -> Result<i32, String> {
    s.parse::<i32>().map_err(|e| format!("parse error: {}", e))
}

fn positive_only(n: i32) -> Result<i32, String> {
    if n > 0 {
        Ok(n)
    } else {
        Err(format!("{} is not positive", n))
    }
}

// and_then の連鎖: 失敗したステップのエラーがそのまま返る
let result = Ok("42")
    .and_then(parse_number)        // Ok(42)
    .and_then(positive_only);      // Ok(42)

assert_eq!(result, Ok(42));

let fail = Ok("-5")
    .and_then(parse_number)        // Ok(-5)
    .and_then(positive_only);      // Err("-5 is not positive")

assert!(fail.is_err());
```

### Haskell との対応表

| Haskell | Rust (`Option`) | Rust (`Result`) |
|---------|-----------------|-----------------|
| `return x` | `Some(x)` | `Ok(x)` |
| `m >>= f` | `m.and_then(f)` | `m.and_then(f)` |
| `Nothing` | `None` | `Err(e)` |
| `do { x <- m; f x }` | `m.and_then(\|x\| f(x))` | `m.and_then(\|x\| f(x))` |

---

## 8.5 ジェネリクスとトレイト境界で型クラス的な抽象を作る

### Monad トレイトの定義

`Maybe<A>` に対する `Monad` トレイトを定義します。`Functor` を supertrait にすることで
「Monad は必ず Functor でもある」という Haskell の階層構造を表現します。

```rust
pub trait Monad<A>: Functor<A> {
    /// 値をモナドに包む (Haskell の return)。
    fn wrap(value: A) -> Self;

    /// モナドの値を取り出して f に渡し、新しいモナドを返す (Haskell の >>=)。
    fn bind<B, F>(self, f: F) -> Maybe<B>
    where
        F: Fn(A) -> Maybe<B>;
}

impl<A> Monad<A> for Maybe<A> {
    fn wrap(value: A) -> Self {
        Maybe::Just(value)
    }

    fn bind<B, F>(self, f: F) -> Maybe<B>
    where
        F: Fn(A) -> Maybe<B>,
    {
        match self {
            Maybe::Just(a) => f(a),
            Maybe::Nothing => Maybe::Nothing,
        }
    }
}
```

`bind` を連鎖させることで、失敗が起きた時点で処理を中断する計算パイプラインを作れます。

```rust
/// Maybe の bind を連鎖させた安全な計算パイプライン。
pub fn safe_pipeline(x: i32) -> Maybe<i32> {
    let step1 = |n: i32| -> Maybe<i32> {
        if n == 0 { Maybe::Nothing } else { Maybe::Just(100 / n) }
    };
    let step2 = |n: i32| -> Maybe<i32> {
        if n % 2 == 0 { Maybe::Just(n) } else { Maybe::Nothing }
    };
    let step3 = |n: i32| -> Maybe<i32> { Maybe::Just(n / 2) };

    // bind の連鎖 = Haskell の do 記法の脱糖
    Maybe::Just(x).bind(step1).bind(step2).bind(step3)
}

// x=50 の場合: step1: Just(100/50=2) -> step2: 2%2==0 → Just(2) -> step3: Just(2/2=1) -> Just(1)
assert_eq!(safe_pipeline(50), Maybe::Just(1));
// x=20 の場合: step1: Just(100/20=5) -> step2: 5%2!=0 -> Nothing
assert_eq!(safe_pipeline(20), Maybe::Nothing);
// x=0 の場合: step1: Nothing (ゼロ除算回避)
assert_eq!(safe_pipeline(0),  Maybe::Nothing);
```

### トレイト境界を使った汎用関数

複数のトレイトを組み合わせた境界（trait bound）で、型クラスを活用した汎用処理を書けます。

```rust
/// 型 T が Debug・PartialOrd・Clone を実装しているときだけ使える汎用関数。
fn print_max<T>(a: T, b: T) -> T
where
    T: PartialOrd + Clone + std::fmt::Debug,
{
    let result = if a >= b { a.clone() } else { b.clone() };
    println!("max = {:?}", result);
    result
}
```

---

## 8.6 Monoid トレイトの実装例

### Haskell の Monoid

```haskell
class Monoid a where
  mempty  :: a
  mappend :: a -> a -> a  -- (<>) と同義
  mconcat :: [a] -> a
  mconcat = foldr mappend mempty
```

### Rust での定義と実装

```rust
/// 結合律を満たす二項演算と単位元を持つ型を抽象化するトレイト。
pub trait Monoid: Sized {
    /// 単位元 (Haskell の mempty)。
    fn empty() -> Self;

    /// 二項演算 (Haskell の mappend / <>)。
    fn combine(self, other: Self) -> Self;
}
```

`i32` の加算モノイドと `String` の連結モノイドを実装します。

```rust
/// i32 の加算モノイド: 単位元 = 0、演算 = 加算
impl Monoid for i32 {
    fn empty() -> Self { 0 }
    fn combine(self, other: Self) -> Self { self + other }
}

/// String の連結モノイド: 単位元 = ""、演算 = 文字列連結
impl Monoid for String {
    fn empty() -> Self { String::new() }
    fn combine(self, other: Self) -> Self { self + &other }
}
```

### モノイド則の確認

```rust
// 左単位元則: empty().combine(x) == x
assert_eq!(i32::empty().combine(42), 42);

// 右単位元則: x.combine(empty()) == x
assert_eq!(42_i32.combine(i32::empty()), 42);

// 結合律: (x + y) + z == x + (y + z)
assert_eq!(1_i32.combine(2).combine(3), 1_i32.combine(2_i32.combine(3)));
```

### fold_monoid: mconcat の Rust 実装

Haskell の `mconcat` に相当する汎用畳み込み関数を、`Monoid` トレイト境界で実装します。

```rust
/// Monoid を実装した型のイテレータを畳み込む汎用関数。
/// Haskell の mconcat :: Monoid a => [a] -> a に相当。
pub fn fold_monoid<T, I>(iter: I) -> T
where
    T: Monoid,
    I: Iterator<Item = T>,
{
    iter.fold(T::empty(), |acc, x| acc.combine(x))
}
```

型引数 `T` に `Monoid` 境界を付けるだけで、加算にも文字列連結にも使える汎用関数になります。

```rust
// i32 の合計
let sum: i32 = fold_monoid(vec![1, 2, 3, 4, 5].into_iter());
assert_eq!(sum, 15);

// 空イテレータ -> 単位元
let zero: i32 = fold_monoid(std::iter::empty());
assert_eq!(zero, 0);

// String の連結
let words = vec!["Rust".to_string(), " is".to_string(), " great".to_string()];
let sentence = fold_monoid(words.into_iter());
assert_eq!(sentence, "Rust is great");
```

---

## 8.7 全体像: Functor / Monad / Monoid の階層

```
Monoid<T>
  ├── empty() -> T
  └── combine(T, T) -> T

Functor<A>
  └── fmap<B>(Self, A -> B) -> Self::Output<B>

Monad<A>: Functor<A>    ← Functor を supertrait にする
  ├── wrap(A) -> Self
  └── bind<B>(Self, A -> Maybe<B>) -> Maybe<B>
```

Haskell では `Applicative` がこの階層の中間に入りますが、
本章では簡略化のため `Functor -> Monad` の 2 段階で扱います。

---

## まとめ

| 概念 | Haskell | Rust |
|------|---------|------|
| 型クラス定義 | `class Functor f where fmap :: ...` | `trait Functor<A> { fn fmap<B, F>(...) }` |
| インスタンス宣言 | `instance Functor Maybe where ...` | `impl Functor<A> for Maybe<A> { ... }` |
| Functor | `fmap :: (a -> b) -> f a -> f b` | `fn fmap<B, F: Fn(A)->B>(self, f: F) -> Self::Output<B>` |
| Monad の bind | `(>>=) :: m a -> (a -> m b) -> m b` | `fn bind<B, F: Fn(A)->Maybe<B>>(self, f: F) -> Maybe<B>` |
| リストモナドの bind | `xs >>= f` | `xs.into_iter().flat_map(f).collect()` |
| Option / Result | `Maybe a` | `Option<T>` / `Result<T, E>` |
| and_then | `>>=` 相当 | `.and_then(f)` |
| Monoid | `mempty`, `mappend` | `Monoid::empty()`, `Monoid::combine` |
| mconcat | `mconcat xs` | `fold_monoid(iter)` |

---

## 章末演習問題

### 問題 1

以下の `Functor` 実装はファンクター則を**破っています**。どちらの則が破られていますか？理由を説明し、正しい実装を書いてください。

```rust
impl<A: Clone> Functor<A> for Maybe<A> {
    type Output<B> = Maybe<B>;

    fn fmap<B, F>(self, f: F) -> Maybe<B>
    where
        F: Fn(A) -> B,
    {
        match self {
            Maybe::Just(a) => Maybe::Just(f(a)),
            // 誤った実装: Nothing を Just に変えてしまう
            Maybe::Nothing => panic!("fmap called on Nothing"),
        }
    }
}
```

<details>
<summary>解答</summary>

同一性則 `fmap(id) == id` が破られています。`Nothing.fmap(|x| x)` は `Nothing` を返すべきですが、この実装はパニックします。
また `fmap` が `Nothing` に対して恒等関数を適用しても `Nothing` のままであるべきという性質も満たせません。

正しい実装:

```rust
impl<A> Functor<A> for Maybe<A> {
    type Output<B> = Maybe<B>;

    fn fmap<B, F>(self, f: F) -> Maybe<B>
    where
        F: Fn(A) -> B,
    {
        match self {
            Maybe::Just(a) => Maybe::Just(f(a)),
            Maybe::Nothing => Maybe::Nothing, // Nothing はそのまま返す
        }
    }
}
```

</details>

---

### 問題 2

`Vec<i32>` に `Monoid` トレイトを実装してください。

- 単位元: `vec![]`（空ベクタ）
- 二項演算: ベクタの連結

実装後、以下のテストが通ることを確認してください。

```rust
// 左単位元則
let v: Vec<i32> = vec![1, 2, 3];
assert_eq!(Vec::empty().combine(v.clone()), v);

// 右単位元則
assert_eq!(v.clone().combine(Vec::empty()), v);

// fold_monoid での利用
let result = fold_monoid(vec![vec![1, 2], vec![3, 4], vec![5]].into_iter());
assert_eq!(result, vec![1, 2, 3, 4, 5]);
```

<details>
<summary>解答</summary>

```rust
impl Monoid for Vec<i32> {
    fn empty() -> Self {
        vec![]
    }

    fn combine(mut self, other: Self) -> Self {
        self.extend(other);
        self
    }
}
```

より汎用的にするには `T: Clone` などの境界を付けた `impl<T> Monoid for Vec<T>` にできますが、
`Monoid: Sized` であるため型パラメータの扱いに注意が必要です。

</details>

---

### 問題 3

以下の 3 つのステップからなる計算パイプラインを `Maybe::bind` で実装してください。

1. 文字列を `i32` にパースする（失敗したら `Nothing`）
2. 値が 0 以上 100 以下の場合のみ通す（それ以外は `Nothing`）
3. 値を百分率の文字列にフォーマットする（例: `42` -> `"42%"`）

```rust
fn string_to_percent(s: &str) -> Maybe<String> {
    // ここを実装する
}

assert_eq!(string_to_percent("42"),  Maybe::Just("42%".to_string()));
assert_eq!(string_to_percent("150"), Maybe::Nothing); // 範囲外
assert_eq!(string_to_percent("abc"), Maybe::Nothing); // パース失敗
```

<details>
<summary>解答</summary>

```rust
fn string_to_percent(s: &str) -> Maybe<String> {
    let parse = |s: &str| -> Maybe<i32> {
        s.parse::<i32>().map_or(Maybe::Nothing, Maybe::Just)
    };

    let clamp = |n: i32| -> Maybe<i32> {
        if (0..=100).contains(&n) { Maybe::Just(n) } else { Maybe::Nothing }
    };

    let format_percent = |n: i32| -> Maybe<String> {
        Maybe::Just(format!("{}%", n))
    };

    // 文字列をパースできたら s を参照できないため、先にパースして Maybe に包む
    parse(s).bind(clamp).bind(format_percent)
}
```

`parse` ステップでは `&str` を受け取って `Maybe<i32>` を返すクロージャを定義し、
`bind` の連鎖で各ステップをつなぎます。`Nothing` が発生した時点で以降のステップは実行されません。

</details>

---

## 強化: Applicative の概念と Rust での近似実装

### Applicative とは

**Applicative**（アプリカティブ）は、関数型プログラミングの型クラス階層の一つです。

```
Functor → Applicative → Monad
```

- **Functor**: `fmap` — コンテナ内の値に関数を適用
- **Applicative**: `ap`、`liftA2` — コンテナ内の関数をコンテナ内の値に適用
- **Monad**: `bind`（`>>=`） — コンテナ内の値を取り出して新しいコンテナを生成

### ap と liftA2

```rust
// ap: Option<関数> と Option<値> を組み合わせる
pub fn option_ap<A, B>(f: Option<impl Fn(A) -> B>, a: Option<A>) -> Option<B> {
    match (f, a) {
        (Some(func), Some(val)) => Some(func(val)),
        _ => None,
    }
}

// liftA2: 2引数関数を Option に「持ち上げる」
pub fn option_lift2<A, B, C>(
    f: impl Fn(A, B) -> C,
    a: Option<A>,
    b: Option<B>,
) -> Option<C> {
    match (a, b) {
        (Some(a_val), Some(b_val)) => Some(f(a_val, b_val)),
        _ => None,
    }
}

// 使用例
let result = option_lift2(|a, b| a + b, Some(3), Some(4));
assert_eq!(result, Some(7));

let result = option_lift2(|a, b: i32| a + b, Some(3), None);
assert_eq!(result, None);  // どちらかが None なら None
```

### Result への応用

```rust
pub fn result_lift2<A, B, C, E>(
    f: impl Fn(A, B) -> C,
    a: Result<A, E>,
    b: Result<B, E>,
) -> Result<C, E> {
    match (a, b) {
        (Ok(a_val), Ok(b_val)) => Ok(f(a_val, b_val)),
        (Err(e), _) | (_, Err(e)) => Err(e),
    }
}

// 使用例: 2つの Result を組み合わせる
let result: Result<i32, &str> = result_lift2(|a, b| a + b, Ok(3), Ok(4));
assert_eq!(result, Ok(7));
```

### Vec の Applicative: 全組み合わせ

`Vec` の Applicative は**デカルト積**（全組み合わせ）を表します。

```rust
pub fn vec_lift2<A: Clone, B: Clone, C>(
    f: impl Fn(A, B) -> C,
    xs: &[A],
    ys: &[B],
) -> Vec<C> {
    let mut result = Vec::new();
    for x in xs {
        for y in ys {
            result.push(f(x.clone(), y.clone()));
        }
    }
    result
}

// 使用例: サイコロ2つの全組み合わせ
let dice1 = vec![1, 2, 3];
let dice2 = vec![1, 2, 3];
let sums = vec_lift2(|a, b| a + b, &dice1, &dice2);
// [2, 3, 4, 3, 4, 5, 4, 5, 6]
```

### Applicative vs Monad の違い

```rust
// Applicative（liftA2）: 両方の結果が独立している
let result = option_lift2(|a, b| a + b, parse_a(), parse_b());

// Monad（and_then）: 前の結果に依存して次の計算を決める
let result = parse_a().and_then(|a| {
    if a > 0 { parse_b().map(|b| a + b) }
    else { None }
});
```

Applicative は各計算が**独立**している場合に使い、Monad は前の結果に**依存**する場合に使います。

### Rust での位置付け

Rust には Applicative 型クラスはありませんが、`Option` と `Result` の `map`・`and_then`・`zip` などのメソッドが実質的に同じ機能を提供します。

```rust
// zip は liftA2 の特殊ケース（タプルにまとめる）
let result = Some(3).zip(Some("hello")); // Some((3, "hello"))

// and_then のチェーンは Monad
let result = Some(3)
    .and_then(|x| if x > 0 { Some(x * 2) } else { None });
```
