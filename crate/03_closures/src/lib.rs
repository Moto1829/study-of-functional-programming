//! 第3章: クロージャと高階関数
//!
//! このクレートは「クロージャと高階関数」の概念を Rust で示すサンプル集です。
//! 各モジュールが独立したトピックに対応しています。

// ============================================================
// 1. 基本的なクロージャと環境のキャプチャ
// ============================================================

/// 指定した値だけオフセットする加算クロージャを生成して返す。
///
/// 環境の `offset` を**不変参照**でキャプチャするため、
/// 返り値は `Fn(i32) -> i32` を実装する。
///
/// # Examples
///
/// ```rust
/// let add5 = closures::make_offset_adder(5);
/// assert_eq!(add5(10), 15);
/// ```
pub fn make_offset_adder(offset: i32) -> impl Fn(i32) -> i32 {
    move |x| x + offset
}

/// 呼び出すたびに内部カウンタを増やし、現在値を返すクロージャを生成する。
///
/// 内部の `count` を**可変参照**でキャプチャするため、
/// 返り値は `FnMut() -> i32` を実装する。
///
/// # Examples
///
/// ```rust
/// let mut counter = closures::make_counter();
/// assert_eq!(counter(), 1);
/// assert_eq!(counter(), 2);
/// assert_eq!(counter(), 3);
/// ```
pub fn make_counter() -> impl FnMut() -> i32 {
    let mut count = 0;
    move || {
        count += 1;
        count
    }
}

// ============================================================
// 2. Fn / FnMut / FnOnce を引数に取る高階関数
// ============================================================

/// `Fn(T) -> T` を受け取り、スライスの各要素に適用した新しい `Vec` を返す。
///
/// `Fn` を要求するため、クロージャは何度でも呼び出せる。
///
/// # Examples
///
/// ```rust
/// let result = closures::map_fn(&[1, 2, 3], |x| x * 10);
/// assert_eq!(result, vec![10, 20, 30]);
/// ```
pub fn map_fn<T, F>(values: &[T], f: F) -> Vec<T>
where
    T: Copy,
    F: Fn(T) -> T,
{
    values.iter().map(|&x| f(x)).collect()
}

/// `FnMut(i32) -> i32` を受け取り、スライスの各要素に順番に適用する。
///
/// `FnMut` を要求するため、内部で状態を持つクロージャも渡せる。
/// 呼び出しごとにクロージャの状態が変わる可能性があることに注意。
///
/// # Examples
///
/// ```rust
/// let mut calls = 0;
/// let result = closures::map_fn_mut(&[10, 20, 30], |x| { calls += 1; x + calls });
/// assert_eq!(result, vec![11, 22, 33]);
/// ```
pub fn map_fn_mut<F>(values: &[i32], mut f: F) -> Vec<i32>
where
    F: FnMut(i32) -> i32,
{
    values.iter().map(|&x| f(x)).collect()
}

/// `FnOnce() -> String` を受け取り、ちょうど1回だけ呼び出してその結果を返す。
///
/// `FnOnce` を要求するため、所有権を消費するクロージャも渡せる。
///
/// # Examples
///
/// ```rust
/// let name = String::from("World");
/// let greeting = closures::call_once(move || format!("Hello, {}!", name));
/// assert_eq!(greeting, "Hello, World!");
/// ```
pub fn call_once<F: FnOnce() -> String>(f: F) -> String {
    f()
}

// ============================================================
// 3. 関数を返す関数（impl Fn を返す）
// ============================================================

/// 2つの変換関数を合成した関数を返す。
///
/// `g(x)` を先に適用し、その結果に `f` を適用する（数学的な `f ∘ g`）。
///
/// # Examples
///
/// ```rust
/// let double = |x: i32| x * 2;
/// let add_one = |x: i32| x + 1;
/// let double_then_add = closures::compose(add_one, double);
/// assert_eq!(double_then_add(5), 11); // (5 * 2) + 1
/// ```
pub fn compose<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> C
where
    F: Fn(B) -> C,
    G: Fn(A) -> B,
{
    move |x| f(g(x))
}

/// 条件に応じて異なる変換関数をボックスに包んで返す。
///
/// 型が静的に決まらない場合は `Box<dyn Fn>` を使う。
///
/// # Examples
///
/// ```rust
/// let negate = closures::choose_transform(true);
/// assert_eq!(negate(42), -42);
///
/// let identity = closures::choose_transform(false);
/// assert_eq!(identity(42), 42);
/// ```
pub fn choose_transform(negate: bool) -> Box<dyn Fn(i32) -> i32> {
    if negate {
        Box::new(|x| -x)
    } else {
        Box::new(|x| x)
    }
}

// ============================================================
// 4. move クロージャの例
// ============================================================

/// 与えたデータを所有権ごとスレッドに渡し、スレッド内でそれを2乗して返す。
///
/// スレッドのライフタイムはコンパイル時に不明なため、
/// `move` クロージャで `data` の所有権をクロージャに移す必要がある。
///
/// # Examples
///
/// ```rust
/// let result = closures::square_in_thread(7);
/// assert_eq!(result, 49);
/// ```
pub fn square_in_thread(value: i32) -> i32 {
    let handle = std::thread::spawn(move || value * value);
    handle.join().expect("スレッドがパニックした")
}

// ============================================================
// 5. 関数ポインタ fn とクロージャの違い
// ============================================================

/// 関数ポインタ `fn(i32) -> i32` を受け取り、値に適用する。
///
/// `fn` 型は環境をキャプチャしないため、環境を持つクロージャは渡せない。
/// 環境をキャプチャしない（`||` 内で外部変数を使わない）クロージャは
/// 自動的に `fn` 型に強制変換される。
///
/// # Examples
///
/// ```rust
/// fn double(x: i32) -> i32 { x * 2 }
///
/// assert_eq!(closures::apply_fn_ptr(double, 5), 10);
/// // 環境をキャプチャしないクロージャも渡せる
/// assert_eq!(closures::apply_fn_ptr(|x| x + 1, 5), 6);
/// ```
pub fn apply_fn_ptr(f: fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}

/// `Fn(i32) -> i32` を受け取り、値に適用する。
///
/// `impl Fn` は環境をキャプチャするクロージャも受け付ける。
/// 通常の高階関数にはこちらを使うのが望ましい。
///
/// # Examples
///
/// ```rust
/// let offset = 100;
/// // 環境をキャプチャするクロージャも渡せる
/// let result = closures::apply_closure(|x| x + offset, 5);
/// assert_eq!(result, 105);
/// ```
pub fn apply_closure<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(x)
}

// ============================================================
// 6. カリー化を模倣するパターン
// ============================================================

/// カリー化された加算関数。
///
/// `add(x)` は `|y| x + y` を返す。
/// これにより部分適用 (partial application) が実現できる。
///
/// # Examples
///
/// ```rust
/// let add10 = closures::add(10);
/// assert_eq!(add10(5), 15);
/// assert_eq!(closures::add(3)(4), 7);
/// ```
pub fn add(x: i32) -> impl Fn(i32) -> i32 {
    move |y| x + y
}

/// カリー化された乗算関数（3引数版）。
///
/// `multiply3(x)(y)(z)` は `x * y * z` を計算する。
///
/// # Examples
///
/// ```rust
/// assert_eq!(closures::multiply3(2)(3)(4), 24);
/// let double = closures::multiply3(2)(1); // 2 * 1 * z = 2z
/// assert_eq!(double(5), 10);
/// ```
pub fn multiply3(x: i32) -> impl Fn(i32) -> Box<dyn Fn(i32) -> i32> {
    move |y| Box::new(move |z| x * y * z)
}

// ============================================================
// 7. apply_twice のような汎用高階関数
// ============================================================

/// 関数 `f` を `x` に2回繰り返し適用する。
///
/// `T: Clone` が必要なのは、`f` が `T` を所有権で受け取るため、
/// 1回目の呼び出しで消費された値を2回目に渡せるよう複製が必要なためである。
///
/// # Examples
///
/// ```rust
/// let double = |x: i32| x * 2;
/// assert_eq!(closures::apply_twice(double, 3), 12); // 3 → 6 → 12
/// ```
pub fn apply_twice<T: Clone, F: Fn(T) -> T>(f: F, x: T) -> T {
    f(f(x.clone()))
}

/// 関数 `f` を `x` に `n` 回繰り返し適用する。
///
/// n = 0 のとき元の値をそのまま返す。
///
/// # Examples
///
/// ```rust
/// let add_one = |x: i32| x + 1;
/// assert_eq!(closures::apply_n_times(add_one, 0, 5), 5);
/// assert_eq!(closures::apply_n_times(|x: i32| x * 2, 3, 1), 8); // 1→2→4→8
/// ```
pub fn apply_n_times<T: Clone, F: Fn(T) -> T>(f: F, n: usize, x: T) -> T {
    let mut result = x;
    for _ in 0..n {
        result = f(result.clone());
    }
    result
}

// ============================================================
// テスト
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- 1. 基本クロージャと環境キャプチャ ---

    #[test]
    fn test_make_offset_adder() {
        let add7 = make_offset_adder(7);
        assert_eq!(add7(0), 7);
        assert_eq!(add7(3), 10);
        assert_eq!(add7(-5), 2);
    }

    #[test]
    fn test_make_counter_increments_sequentially() {
        let mut counter = make_counter();
        assert_eq!(counter(), 1);
        assert_eq!(counter(), 2);
        assert_eq!(counter(), 3);
    }

    #[test]
    fn test_make_counter_independent_instances() {
        // 2つのカウンタは独立して動作する
        let mut c1 = make_counter();
        let mut c2 = make_counter();
        assert_eq!(c1(), 1);
        assert_eq!(c1(), 2);
        assert_eq!(c2(), 1); // c1 とは無関係
    }

    // --- 2. Fn / FnMut / FnOnce を引数に取る ---

    #[test]
    fn test_map_fn_with_pure_closure() {
        let result = map_fn(&[1, 2, 3, 4], |x| x * x);
        assert_eq!(result, vec![1, 4, 9, 16]);
    }

    #[test]
    fn test_map_fn_mut_with_stateful_closure() {
        // 呼び出しごとに +1 ずつ加算量が増える
        let mut bonus = 0_i32;
        let result = map_fn_mut(&[10, 20, 30], |x| {
            bonus += 1;
            x + bonus
        });
        assert_eq!(result, vec![11, 22, 33]);
    }

    #[test]
    fn test_call_once_consumes_owned_value() {
        let data = String::from("Rust");
        let result = call_once(move || format!("Hello, {}!", data));
        assert_eq!(result, "Hello, Rust!");
    }

    // --- 3. 関数を返す関数 ---

    #[test]
    fn test_compose_applies_g_before_f() {
        let double = |x: i32| x * 2;
        let add_one = |x: i32| x + 1;
        let double_then_add = compose(add_one, double);
        assert_eq!(double_then_add(5), 11); // (5*2) + 1
    }

    #[test]
    fn test_compose_associativity() {
        // (f ∘ g) ∘ h  ==  f ∘ (g ∘ h) を確認
        let triple = |x: i32| x * 3;
        let add_two = |x: i32| x + 2;
        let square = |x: i32| x * x;

        let f_g = compose(add_two, triple); // x*3 + 2
        let composed1 = compose(f_g, square); // (x^2)*3 + 2

        let g_h = compose(triple, square); // x^2 * 3
        let composed2 = compose(add_two, g_h); // x^2 * 3 + 2

        assert_eq!(composed1(4), composed2(4)); // (16*3)+2 = 50
    }

    #[test]
    fn test_choose_transform_negate() {
        let negate = choose_transform(true);
        assert_eq!(negate(10), -10);
        assert_eq!(negate(-5), 5);
    }

    #[test]
    fn test_choose_transform_identity() {
        let identity = choose_transform(false);
        assert_eq!(identity(42), 42);
        assert_eq!(identity(-7), -7);
    }

    // --- 4. move クロージャ（スレッド） ---

    #[test]
    fn test_square_in_thread() {
        assert_eq!(square_in_thread(0), 0);
        assert_eq!(square_in_thread(7), 49);
        assert_eq!(square_in_thread(-3), 9);
    }

    // --- 5. 関数ポインタとクロージャの違い ---

    #[test]
    fn test_apply_fn_ptr_with_named_fn() {
        fn triple(x: i32) -> i32 {
            x * 3
        }
        assert_eq!(apply_fn_ptr(triple, 4), 12);
    }

    #[test]
    fn test_apply_fn_ptr_with_non_capturing_closure() {
        // 環境をキャプチャしないクロージャは fn 型に強制変換される
        assert_eq!(apply_fn_ptr(|x| x - 1, 10), 9);
    }

    #[test]
    fn test_apply_closure_with_capturing_closure() {
        let offset = 100;
        // offset をキャプチャするクロージャは fn 型に渡せないが、impl Fn には渡せる
        let result = apply_closure(|x| x + offset, 5);
        assert_eq!(result, 105);
    }

    // --- 6. カリー化 ---

    #[test]
    fn test_add_curried() {
        let add10 = add(10);
        assert_eq!(add10(0), 10);
        assert_eq!(add10(5), 15);
        assert_eq!(add10(-3), 7);
    }

    #[test]
    fn test_add_immediate_application() {
        assert_eq!(add(3)(4), 7);
        assert_eq!(add(0)(99), 99);
    }

    #[test]
    fn test_multiply3_curried() {
        assert_eq!(multiply3(2)(3)(4), 24);
        assert_eq!(multiply3(1)(1)(1), 1);
        assert_eq!(multiply3(5)(0)(100), 0);
    }

    #[test]
    fn test_multiply3_partial_application() {
        // 2引数まで適用して「2倍する関数」を作る
        let double = multiply3(2)(1);
        assert_eq!(double(5), 10);
        assert_eq!(double(0), 0);
    }

    // --- 7. apply_twice / apply_n_times ---

    #[test]
    fn test_apply_twice_integer() {
        let double = |x: i32| x * 2;
        assert_eq!(apply_twice(double, 1), 4);   // 1→2→4
        assert_eq!(apply_twice(double, 3), 12);  // 3→6→12
    }

    #[test]
    fn test_apply_twice_string() {
        let exclaim = |s: String| format!("{}!", s);
        assert_eq!(apply_twice(exclaim, String::from("Hi")), "Hi!!");
    }

    #[test]
    fn test_apply_n_times_zero() {
        let add_one = |x: i32| x + 1;
        assert_eq!(apply_n_times(add_one, 0, 42), 42); // 0回なので変わらない
    }

    #[test]
    fn test_apply_n_times_power_of_two() {
        let double = |x: i32| x * 2;
        assert_eq!(apply_n_times(double, 4, 1), 16); // 1→2→4→8→16
    }

    #[test]
    fn test_apply_n_times_equals_apply_twice_for_n2() {
        let add3 = |x: i32| x + 3;
        let x = 10;
        // apply_twice と apply_n_times(f, 2, x) は同じ結果になるはず
        assert_eq!(apply_n_times(add3, 2, x), apply_twice(add3, x));
    }

    // --- 複合テスト: カリー化 + apply_n_times ---

    #[test]
    fn test_curried_add_with_apply_n_times() {
        // add(5) で作った関数を3回適用: 0 → 5 → 10 → 15
        let add5 = add(5);
        assert_eq!(apply_n_times(add5, 3, 0), 15);
    }

    // --- 複合テスト: compose + map_fn ---

    #[test]
    fn test_compose_used_in_map() {
        let double = |x: i32| x * 2;
        let add_one = |x: i32| x + 1;
        let transform = compose(add_one, double);

        let result = map_fn(&[1, 2, 3], transform);
        assert_eq!(result, vec![3, 5, 7]); // [1*2+1, 2*2+1, 3*2+1]
    }
}

// ============================================================
// 強化: メモ化と遅延評価（Lazy パターン）
// ============================================================

use std::cell::OnceCell;
use std::collections::HashMap;

/// 引数なしの計算をメモ化する Lazy<T>
///
/// 最初の呼び出し時のみ `f` を実行し、結果をキャッシュする。
pub struct Lazy<T> {
    cell: OnceCell<T>,
    init: Box<dyn Fn() -> T>,
}

impl<T> Lazy<T> {
    pub fn new(init: impl Fn() -> T + 'static) -> Self {
        Lazy {
            cell: OnceCell::new(),
            init: Box::new(init),
        }
    }

    pub fn get(&self) -> &T {
        self.cell.get_or_init(|| (self.init)())
    }
}

/// HashMap を使ったメモ化クロージャ
///
/// 同じ引数での計算結果をキャッシュし、2回目以降は即返す。
pub struct Memoize<A, B> {
    cache: HashMap<A, B>,
    func: Box<dyn Fn(A) -> B>,
}

impl<A: Eq + std::hash::Hash + Clone, B: Clone> Memoize<A, B> {
    pub fn new(func: impl Fn(A) -> B + 'static) -> Self {
        Memoize {
            cache: HashMap::new(),
            func: Box::new(func),
        }
    }

    pub fn call(&mut self, arg: A) -> B {
        if let Some(cached) = self.cache.get(&arg) {
            return cached.clone();
        }
        let result = (self.func)(arg.clone());
        self.cache.insert(arg, result.clone());
        result
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod memoize_tests {
    use super::*;

    #[test]
    fn test_lazy_evaluates_once() {
        let call_count = std::cell::Cell::new(0usize);
        // 注: Cell は Fn クロージャ内で使用可能
        let lazy = Lazy::new(move || {
            call_count.set(call_count.get() + 1);
            42
        });

        assert_eq!(*lazy.get(), 42);
        assert_eq!(*lazy.get(), 42); // 2回呼んでも同じ値
    }

    #[test]
    fn test_lazy_expensive_computation() {
        let lazy = Lazy::new(|| {
            // 高コストな計算の代わりに単純な例
            (1..=100).sum::<i32>()
        });

        assert_eq!(*lazy.get(), 5050);
        assert_eq!(*lazy.get(), 5050); // キャッシュから返る
    }

    #[test]
    fn test_memoize_caches_results() {
        let mut memo = Memoize::new(|x: i32| x * x);

        assert_eq!(memo.call(5), 25);
        assert_eq!(memo.cache_size(), 1);

        assert_eq!(memo.call(5), 25); // キャッシュから返る
        assert_eq!(memo.cache_size(), 1); // サイズ変わらず

        assert_eq!(memo.call(6), 36);
        assert_eq!(memo.cache_size(), 2);
    }

    #[test]
    fn test_memoize_fib() {
        // メモ化を使ったフィボナッチ（反復版）
        fn fib_memo(n: u32) -> u64 {
            let mut memo: HashMap<u32, u64> = HashMap::new();
            fn fib_inner(n: u32, memo: &mut HashMap<u32, u64>) -> u64 {
                if n <= 1 { return n as u64; }
                if let Some(&v) = memo.get(&n) { return v; }
                let result = fib_inner(n - 1, memo) + fib_inner(n - 2, memo);
                memo.insert(n, result);
                result
            }
            fib_inner(n, &mut memo)
        }

        assert_eq!(fib_memo(10), 55);
        assert_eq!(fib_memo(30), 832040);
    }
}
