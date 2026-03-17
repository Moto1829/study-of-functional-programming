/// クロージャ（Closures）の例
///
/// クロージャとは、定義されたスコープの変数をキャプチャできる匿名関数です。
/// Rustのクロージャは `|引数| 式` の形で書きます。

/// クロージャを引数として受け取る関数
pub fn apply_twice<T, F>(f: F, x: T) -> T
where
    F: Fn(T) -> T,
{
    f(f(x))
}

/// クロージャで環境をキャプチャする例
pub fn make_counter(start: i32) -> impl FnMut() -> i32 {
    let mut count = start;
    move || {
        let current = count;
        count += 1;
        current
    }
}

/// クロージャで状態を保持するアキュムレータ
pub fn make_accumulator(initial: f64) -> impl FnMut(f64) -> f64 {
    let mut total = initial;
    move |n| {
        total += n;
        total
    }
}

/// クロージャを使ってリストをソートする
pub fn sort_by_length(mut words: Vec<String>) -> Vec<String> {
    words.sort_by(|a, b| a.len().cmp(&b.len()));
    words
}

fn main() {
    println!("=== クロージャ（Closures）===");

    // 基本的なクロージャ
    let square = |x: i32| x * x;
    let add_one = |x: i32| x + 1;
    println!("square(5) = {}", square(5));
    println!("add_one(9) = {}", add_one(9));

    // 2回適用
    println!("apply_twice(square, 3) = {}", apply_twice(square, 3)); // 3^4 = 81
    println!("apply_twice(add_one, 0) = {}", apply_twice(add_one, 0)); // 2

    // 環境をキャプチャするクロージャ
    let base = 100;
    let add_base = |x: i32| x + base;
    println!("add_base(42) = {}", add_base(42));

    // ミュータブルなキャプチャ（FnMut）
    let mut counter = make_counter(0);
    println!("counter() = {}", counter());
    println!("counter() = {}", counter());
    println!("counter() = {}", counter());

    // アキュムレータ
    let mut acc = make_accumulator(0.0);
    println!("acc(10.0) = {}", acc(10.0));
    println!("acc(20.0) = {}", acc(20.0));
    println!("acc(5.0)  = {}", acc(5.0));

    // クロージャでソート
    let words = vec!["banana".to_string(), "apple".to_string(), "fig".to_string(), "cherry".to_string()];
    let sorted = sort_by_length(words);
    println!("sort_by_length = {:?}", sorted);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_twice_square() {
        let square = |x: i32| x * x;
        // apply_twice(square, 3) = square(square(3)) = square(9) = 81
        assert_eq!(apply_twice(square, 3), 81);
    }

    #[test]
    fn test_apply_twice_add_one() {
        let add_one = |x: i32| x + 1;
        assert_eq!(apply_twice(add_one, 5), 7);
    }

    #[test]
    fn test_make_counter() {
        let mut counter = make_counter(0);
        assert_eq!(counter(), 0);
        assert_eq!(counter(), 1);
        assert_eq!(counter(), 2);
    }

    #[test]
    fn test_make_counter_with_offset() {
        let mut counter = make_counter(10);
        assert_eq!(counter(), 10);
        assert_eq!(counter(), 11);
    }

    #[test]
    fn test_make_accumulator() {
        let mut acc = make_accumulator(0.0);
        assert!((acc(10.0) - 10.0).abs() < 1e-10);
        assert!((acc(20.0) - 30.0).abs() < 1e-10);
        assert!((acc(5.0) - 35.0).abs() < 1e-10);
    }

    #[test]
    fn test_sort_by_length() {
        let words = vec![
            "banana".to_string(),
            "fig".to_string(),
            "apple".to_string(),
        ];
        let sorted = sort_by_length(words);
        assert_eq!(sorted[0], "fig");
        assert_eq!(sorted[2], "banana");
    }

    #[test]
    fn test_capture_environment() {
        let multiplier = 3;
        let triple = |x: i32| x * multiplier;
        assert_eq!(triple(4), 12);
        assert_eq!(triple(7), 21);
    }
}
