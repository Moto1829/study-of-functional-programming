//! # 第8章: トレイトと型クラス的パターン
//!
//! このクレートは Haskell の型クラス（`Functor`・`Monad`・`Monoid`）と
//! 対応する概念を Rust のトレイトで表現する方法を示します。
//!
//! ## 主要な型・トレイト
//!
//! - [`Maybe<T>`] — Haskell の `Maybe` に相当する独自の Option 型
//! - [`Functor`]  — `fmap` を抽象化するトレイト
//! - [`Monad`]    — `bind`（`>>=`）を抽象化するトレイト
//! - [`Monoid`]   — `empty` と `combine` を持つトレイト
//! - [`fold_monoid`] — `Monoid` 境界を使った汎用畳み込み関数

// ---------------------------------------------------------------------------
// 1. Maybe<T> — 独自の Option 型
// ---------------------------------------------------------------------------

/// 値が存在するか (`Just`)、存在しないか (`Nothing`) を表す型。
///
/// Haskell の `Maybe a` に相当します。
/// 標準ライブラリの [`Option<T>`] と同じ意味論を持ちますが、
/// 本章ではトレイト実装の学習目的で独自に定義します。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Maybe<T> {
    /// 値が存在する。
    Just(T),
    /// 値が存在しない。
    Nothing,
}

impl<T> Maybe<T> {
    /// `Just` のときのみ内部値への参照を返すヘルパー。
    pub fn as_ref(&self) -> Maybe<&T> {
        match self {
            Maybe::Just(v) => Maybe::Just(v),
            Maybe::Nothing => Maybe::Nothing,
        }
    }

    /// `Just` のとき `true` を返す。
    pub fn is_just(&self) -> bool {
        matches!(self, Maybe::Just(_))
    }

    /// `Nothing` のとき `true` を返す。
    pub fn is_nothing(&self) -> bool {
        matches!(self, Maybe::Nothing)
    }

    /// `Just` なら内部値を、`Nothing` なら `default` を返す。
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Maybe::Just(v) => v,
            Maybe::Nothing => default,
        }
    }
}

// ---------------------------------------------------------------------------
// 2. Functor トレイト
// ---------------------------------------------------------------------------

/// コンテナ内の値を関数で変換する能力を抽象化するトレイト。
///
/// Haskell の `Functor` 型クラスに相当します。
///
/// ```haskell
/// class Functor f where
///   fmap :: (a -> b) -> f a -> f b
/// ```
///
/// ## ファンクター則
///
/// 正しい実装は以下の2つの法則を満たす必要があります。
///
/// 1. **同一性**: `fmap(id) == id`
/// 2. **合成**: `fmap(g ∘ f) == fmap(g) ∘ fmap(f)`
pub trait Functor<A> {
    /// 変換後のコンテナ型。
    /// 例: `Maybe<A>` に対して `fmap` で `Maybe<B>` を返すため `Output = Maybe<B>`。
    type Output<B>;

    /// コンテナ内の値に関数 `f` を適用し、新しいコンテナを返す。
    ///
    /// # 例
    ///
    /// ```
    /// use traits::{Functor, Maybe};
    ///
    /// let result = Maybe::Just(3).fmap(|x| x * 2);
    /// assert_eq!(result, Maybe::Just(6));
    ///
    /// let nothing: Maybe<i32> = Maybe::Nothing;
    /// assert_eq!(nothing.fmap(|x| x * 2), Maybe::Nothing);
    /// ```
    fn fmap<B, F>(self, f: F) -> Self::Output<B>
    where
        F: Fn(A) -> B;
}

/// `Maybe<A>` への `Functor` 実装。
///
/// `Just(a)` なら `f(a)` を `Just` に包んで返す。
/// `Nothing` ならそのまま `Nothing` を返す。
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

// ---------------------------------------------------------------------------
// 3. Monad トレイト
// ---------------------------------------------------------------------------

/// コンテナ内の値を受け取り、新しいコンテナを返す関数を連鎖させる能力を
/// 抽象化するトレイト。
///
/// Haskell の `Monad` 型クラス（`>>=` 演算子）に相当します。
///
/// ```haskell
/// class Monad m where
///   return :: a -> m a
///   (>>=)  :: m a -> (a -> m b) -> m b
/// ```
///
/// ## モナド則
///
/// 正しい実装は以下の3つの法則を満たす必要があります。
///
/// 1. **左単位元**: `wrap(a).bind(f) == f(a)`
/// 2. **右単位元**: `m.bind(wrap) == m`
/// 3. **結合律**:  `m.bind(f).bind(g) == m.bind(|x| f(x).bind(g))`
pub trait Monad<A>: Functor<A> {
    /// 値をモナドに包む（Haskell の `return`）。
    fn wrap(value: A) -> Self;

    /// モナドの値を取り出して関数 `f` に渡し、新しいモナドを返す（Haskell の `>>=`）。
    ///
    /// # 例
    ///
    /// ```
    /// use traits::{Monad, Maybe};
    ///
    /// let result = Maybe::Just(10).bind(|x| {
    ///     if x > 5 { Maybe::Just(x * 2) } else { Maybe::Nothing }
    /// });
    /// assert_eq!(result, Maybe::Just(20));
    ///
    /// let result2 = Maybe::Just(3).bind(|x| {
    ///     if x > 5 { Maybe::Just(x * 2) } else { Maybe::Nothing }
    /// });
    /// assert_eq!(result2, Maybe::Nothing);
    /// ```
    fn bind<B, F>(self, f: F) -> Maybe<B>
    where
        F: Fn(A) -> Maybe<B>;
}

/// `Maybe<A>` への `Monad` 実装。
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

// ---------------------------------------------------------------------------
// 4. Monoid トレイト
// ---------------------------------------------------------------------------

/// 結合律を満たす二項演算と単位元を持つ型を抽象化するトレイト。
///
/// Haskell の `Monoid` 型クラスに相当します。
///
/// ```haskell
/// class Monoid a where
///   mempty  :: a
///   mappend :: a -> a -> a
/// ```
///
/// ## モノイド則
///
/// 正しい実装は以下の3つの法則を満たす必要があります。
///
/// 1. **左単位元**: `empty().combine(x) == x`
/// 2. **右単位元**: `x.combine(empty()) == x`
/// 3. **結合律**:  `(x.combine(y)).combine(z) == x.combine(y.combine(z))`
pub trait Monoid: Sized {
    /// 単位元（Haskell の `mempty`）を返す。
    ///
    /// 加算モノイドでは `0`、文字列連結モノイドでは `""` が単位元となる。
    fn empty() -> Self;

    /// `self` と `other` を結合して新しい値を返す（Haskell の `mappend` / `<>`）。
    fn combine(self, other: Self) -> Self;
}

/// `i32` の加算モノイド実装。
///
/// - 単位元: `0`
/// - 二項演算: 加算 `+`
impl Monoid for i32 {
    fn empty() -> Self {
        0
    }

    fn combine(self, other: Self) -> Self {
        self + other
    }
}

/// `String` の連結モノイド実装。
///
/// - 単位元: `""` (空文字列)
/// - 二項演算: 文字列の連結
impl Monoid for String {
    fn empty() -> Self {
        String::new()
    }

    fn combine(self, other: Self) -> Self {
        self + &other
    }
}

// ---------------------------------------------------------------------------
// 5. fold_monoid — Monoid 境界を使った汎用畳み込み関数
// ---------------------------------------------------------------------------

/// `Monoid` を実装した型のイテレータを畳み込む汎用関数。
///
/// Haskell の `mconcat :: Monoid a => [a] -> a` に相当します。
/// イテレータの全要素を `Monoid::combine` で結合し、空の場合は `Monoid::empty()` を返します。
///
/// # 型パラメータ
///
/// - `T`: `Monoid` を実装した型
/// - `I`: `Iterator<Item = T>` を実装したイテレータ型
///
/// # 例
///
/// ```
/// use traits::{fold_monoid, Monoid};
///
/// // i32 の加算モノイドで合計を求める
/// let sum = fold_monoid(vec![1, 2, 3, 4, 5].into_iter());
/// assert_eq!(sum, 15);
///
/// // String の連結モノイドで文字列を結合する
/// let words = vec!["Hello".to_string(), ", ".to_string(), "World".to_string()];
/// let sentence = fold_monoid(words.into_iter());
/// assert_eq!(sentence, "Hello, World");
/// ```
pub fn fold_monoid<T, I>(iter: I) -> T
where
    T: Monoid,
    I: Iterator<Item = T>,
{
    iter.fold(T::empty(), |acc, x| acc.combine(x))
}

// ---------------------------------------------------------------------------
// 6. Iterator の flat_map を Monad の >>= として使う例
// ---------------------------------------------------------------------------

/// 整数のスライスを受け取り、各要素に対して `f` を適用した結果をすべて平坦化して返す。
///
/// これは `Iterator::flat_map` を使っており、Haskell の `concatMap` や
/// リストモナドの `>>=` と同じ意味論を持ちます。
///
/// ```haskell
/// -- Haskell でのリストモナド bind
/// [1, 2, 3] >>= \x -> [x, x * 10]
/// -- => [1, 10, 2, 20, 3, 30]
/// ```
///
/// # 例
///
/// ```
/// use traits::flat_map_example;
///
/// let result = flat_map_example(&[1, 2, 3], |x| vec![x, x * 10]);
/// assert_eq!(result, vec![1, 10, 2, 20, 3, 30]);
/// ```
pub fn flat_map_example<A, B, F>(xs: &[A], f: F) -> Vec<B>
where
    A: Copy,
    F: Fn(A) -> Vec<B>,
{
    // Iterator::flat_map は Monad の >>= に対応する:
    //   リスト内の各 a を取り出し、f(a) というリストを生成し、
    //   それを連結した単一のリストを返す。
    xs.iter().copied().flat_map(f).collect()
}

/// `Maybe` の `bind` を連鎖させて安全な計算パイプラインを構築する例。
///
/// 各ステップが失敗する可能性のある計算を、`bind` を使って短絡評価しながら繋げます。
/// これは Haskell の `do` 記法の脱糖に対応します。
///
/// # 例
///
/// ```
/// use traits::{safe_pipeline, Maybe};
///
/// assert_eq!(safe_pipeline(50), Maybe::Just(1)); // 100/50=2 -> 偶数 -> 2/2=1
/// assert_eq!(safe_pipeline(0),  Maybe::Nothing); // ゼロ除算で失敗
/// assert_eq!(safe_pipeline(3),  Maybe::Nothing); // 奇数で失敗
/// ```
///
/// [`Maybe`]: crate::Maybe
pub fn safe_pipeline(x: i32) -> Maybe<i32> {
    // step1: ゼロ除算を回避
    let step1 = |n: i32| -> Maybe<i32> {
        if n == 0 {
            Maybe::Nothing
        } else {
            Maybe::Just(100 / n)
        }
    };

    // step2: 偶数のみを通す
    let step2 = |n: i32| -> Maybe<i32> {
        if n % 2 == 0 {
            Maybe::Just(n)
        } else {
            Maybe::Nothing
        }
    };

    // step3: 値を半分にする
    let step3 = |n: i32| -> Maybe<i32> { Maybe::Just(n / 2) };

    // bind を連鎖させることで失敗した時点で Nothing が伝播する
    Maybe::Just(x).bind(step1).bind(step2).bind(step3)
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Functor のテスト ---

    /// fmap は Just の中の値を変換する
    #[test]
    fn test_functor_fmap_just() {
        let result = Maybe::Just(5).fmap(|x| x * 3);
        assert_eq!(result, Maybe::Just(15));
    }

    /// fmap は Nothing をそのまま返す
    #[test]
    fn test_functor_fmap_nothing() {
        let nothing: Maybe<i32> = Maybe::Nothing;
        assert_eq!(nothing.fmap(|x| x * 3), Maybe::Nothing);
    }

    /// ファンクター同一性則: fmap(id) == id
    #[test]
    fn test_functor_identity_law() {
        let m = Maybe::Just(42);
        assert_eq!(m.clone().fmap(|x| x), m);
    }

    /// ファンクター合成則: fmap(g ∘ f) == fmap(g) ∘ fmap(f)
    #[test]
    fn test_functor_composition_law() {
        let m = Maybe::Just(3);
        let f = |x: i32| x + 1;
        let g = |x: i32| x * 2;

        // fmap(g ∘ f)
        let composed = m.clone().fmap(|x| g(f(x)));
        // fmap(g) ∘ fmap(f)
        let chained = m.fmap(f).fmap(g);

        assert_eq!(composed, chained); // Just(8)
    }

    /// fmap で型を変換できる (i32 -> String)
    #[test]
    fn test_functor_type_change() {
        let result = Maybe::Just(42).fmap(|x| format!("value={}", x));
        assert_eq!(result, Maybe::Just("value=42".to_string()));
    }

    // --- Monad のテスト ---

    /// bind は Just から値を取り出して関数に渡す
    #[test]
    fn test_monad_bind_just() {
        let result = Maybe::Just(10).bind(|x| Maybe::Just(x + 5));
        assert_eq!(result, Maybe::Just(15));
    }

    /// bind は Nothing をそのまま伝播する
    #[test]
    fn test_monad_bind_nothing_propagation() {
        let nothing: Maybe<i32> = Maybe::Nothing;
        let result = nothing.bind(|x| Maybe::Just(x + 5));
        assert_eq!(result, Maybe::Nothing);
    }

    /// bind の連鎖: 途中で Nothing になったらそこで止まる
    #[test]
    fn test_monad_bind_chain_short_circuit() {
        let result = Maybe::Just(10)
            .bind(|x| if x > 5 { Maybe::Just(x * 2) } else { Maybe::Nothing })
            .bind(|x| Maybe::Just(x + 1))
            .bind(|x| if x > 100 { Maybe::Just(x) } else { Maybe::Nothing });

        // 10 > 5 なので Just(20) -> Just(21) -> 21 <= 100 なので Nothing
        assert_eq!(result, Maybe::Nothing);
    }

    /// モナド左単位元則: wrap(a).bind(f) == f(a)
    #[test]
    fn test_monad_left_identity_law() {
        let a = 7;
        let f = |x: i32| Maybe::Just(x * x);

        assert_eq!(Maybe::wrap(a).bind(f), f(a));
    }

    /// モナド右単位元則: m.bind(wrap) == m
    #[test]
    fn test_monad_right_identity_law() {
        let m = Maybe::Just(99);
        assert_eq!(m.clone().bind(Maybe::wrap), m);
    }

    /// safe_pipeline の正常ケース
    #[test]
    fn test_safe_pipeline_success() {
        // 100 / 50 = 2 -> 2 は偶数 -> 2 / 2 = 1 -> Just(1)
        assert_eq!(safe_pipeline(50), Maybe::Just(1));
    }

    /// safe_pipeline のゼロ除算失敗ケース
    #[test]
    fn test_safe_pipeline_zero_division() {
        assert_eq!(safe_pipeline(0), Maybe::Nothing);
    }

    /// safe_pipeline の奇数失敗ケース
    #[test]
    fn test_safe_pipeline_odd_failure() {
        // 100 / 3 = 33 -> 33 は奇数なので Nothing
        assert_eq!(safe_pipeline(3), Maybe::Nothing);
    }

    // --- Monoid のテスト ---

    /// i32 の単位元は 0
    #[test]
    fn test_monoid_i32_empty() {
        assert_eq!(i32::empty(), 0);
    }

    /// i32 の combine は加算
    #[test]
    fn test_monoid_i32_combine() {
        assert_eq!(3_i32.combine(4), 7);
        assert_eq!(0_i32.combine(42), 42); // 左単位元則
        assert_eq!(42_i32.combine(0), 42); // 右単位元則
    }

    /// String の単位元は空文字列
    #[test]
    fn test_monoid_string_empty() {
        assert_eq!(String::empty(), "");
    }

    /// String の combine は文字列連結
    #[test]
    fn test_monoid_string_combine() {
        let s = "Hello".to_string().combine(", World".to_string());
        assert_eq!(s, "Hello, World");
    }

    /// i32 のモノイド結合律
    #[test]
    fn test_monoid_i32_associativity() {
        let x = 1_i32;
        let y = 2_i32;
        let z = 3_i32;
        // (x + y) + z == x + (y + z)
        assert_eq!(x.combine(y).combine(z), x.combine(y.combine(z)));
    }

    // --- fold_monoid のテスト ---

    /// i32 の合計を fold_monoid で求める
    #[test]
    fn test_fold_monoid_i32_sum() {
        let result = fold_monoid(vec![1, 2, 3, 4, 5].into_iter());
        assert_eq!(result, 15_i32);
    }

    /// 空のイテレータは単位元を返す
    #[test]
    fn test_fold_monoid_empty_iterator() {
        let result: i32 = fold_monoid(std::iter::empty());
        assert_eq!(result, 0);
    }

    /// String の連結を fold_monoid で求める
    #[test]
    fn test_fold_monoid_string_concat() {
        let words = vec![
            "Rust".to_string(),
            " is".to_string(),
            " great".to_string(),
        ];
        let result = fold_monoid(words.into_iter());
        assert_eq!(result, "Rust is great");
    }

    // --- Iterator::flat_map (Monad の >>=) のテスト ---

    /// flat_map_example はリストモナドの >>= に対応する
    #[test]
    fn test_flat_map_list_monad() {
        let result = flat_map_example(&[1, 2, 3], |x| vec![x, x * 10]);
        assert_eq!(result, vec![1, 10, 2, 20, 3, 30]);
    }

    /// 空のスライスでは空のベクタを返す
    #[test]
    fn test_flat_map_empty() {
        let result = flat_map_example(&[], |x: i32| vec![x]);
        assert_eq!(result, Vec::<i32>::new());
    }

    /// f が常に空ベクタを返す場合は空のベクタになる
    #[test]
    fn test_flat_map_returns_empty_vecs() {
        let result = flat_map_example(&[1, 2, 3], |_x: i32| Vec::<i32>::new());
        assert_eq!(result, Vec::<i32>::new());
    }

    /// flat_map で「範囲展開」: 各整数 n を 1..=n の Vec に展開する
    #[test]
    fn test_flat_map_range_expansion() {
        let result = flat_map_example(&[1, 2, 3], |x| (1..=x).collect::<Vec<_>>());
        // [1] ++ [1,2] ++ [1,2,3] = [1, 1, 2, 1, 2, 3]
        assert_eq!(result, vec![1, 1, 2, 1, 2, 3]);
    }
}
