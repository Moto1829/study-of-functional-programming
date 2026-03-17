/// 高階関数（Higher-Order Functions）の例
///
/// 高階関数とは、関数を引数として受け取ったり、
/// 関数を戻り値として返したりする関数です。
/// Rustでは`map`、`filter`、`fold`などがその代表例です。

/// 関数を引数に取る高階関数の例: リストの各要素に関数を適用する
pub fn apply_to_each<T, U, F>(items: &[T], f: F) -> Vec<U>
where
    F: Fn(&T) -> U,
{
    items.iter().map(f).collect()
}

/// 関数を引数に取る高階関数の例: 条件を満たす要素のみ残す
pub fn keep_if<T, F>(items: Vec<T>, predicate: F) -> Vec<T>
where
    F: Fn(&T) -> bool,
{
    items.into_iter().filter(predicate).collect()
}

/// 関数を引数に取る高階関数の例: 累積計算
pub fn fold_left<T, U, F>(items: &[T], initial: U, f: F) -> U
where
    F: Fn(U, &T) -> U,
{
    items.iter().fold(initial, f)
}

/// 関数を返す高階関数の例: 指定した値を加算する関数を生成する
pub fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

/// 関数を返す高階関数の例: 指定した値で乗算する関数を生成する
pub fn make_multiplier(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x * n
}

/// 2つの関数を合成する
pub fn compose<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> C
where
    F: Fn(A) -> B,
    G: Fn(B) -> C,
{
    move |x| g(f(x))
}

fn main() {
    println!("=== 高階関数（Higher-Order Functions）===");

    let numbers = vec![1, 2, 3, 4, 5];

    // map: 各要素を2乗する
    let squared: Vec<i32> = apply_to_each(&numbers, |&n| n * n);
    println!("squared = {:?}", squared);

    // filter: 偶数のみ残す
    let evens = keep_if(numbers.clone(), |&n| n % 2 == 0);
    println!("evens = {:?}", evens);

    // fold: 合計を求める
    let sum = fold_left(&numbers, 0, |acc, &n| acc + n);
    println!("sum = {}", sum);

    // 関数を返す高階関数
    let add5 = make_adder(5);
    let double = make_multiplier(2);
    println!("add5(10) = {}", add5(10));
    println!("double(7) = {}", double(7));

    // 関数の合成: まず+3してから*2する
    let add3_then_double = compose(make_adder(3), make_multiplier(2));
    println!("add3_then_double(4) = {}", add3_then_double(4)); // (4+3)*2 = 14
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_to_each() {
        let nums = vec![1, 2, 3];
        let doubled = apply_to_each(&nums, |&n| n * 2);
        assert_eq!(doubled, vec![2, 4, 6]);
    }

    #[test]
    fn test_keep_if() {
        let nums = vec![1, 2, 3, 4, 5, 6];
        let evens = keep_if(nums, |&n| n % 2 == 0);
        assert_eq!(evens, vec![2, 4, 6]);
    }

    #[test]
    fn test_fold_left_sum() {
        let nums = vec![1, 2, 3, 4, 5];
        let sum = fold_left(&nums, 0, |acc, &n| acc + n);
        assert_eq!(sum, 15);
    }

    #[test]
    fn test_fold_left_product() {
        let nums = vec![1, 2, 3, 4, 5];
        let product = fold_left(&nums, 1, |acc, &n| acc * n);
        assert_eq!(product, 120);
    }

    #[test]
    fn test_make_adder() {
        let add10 = make_adder(10);
        assert_eq!(add10(5), 15);
        assert_eq!(add10(-3), 7);
    }

    #[test]
    fn test_make_multiplier() {
        let triple = make_multiplier(3);
        assert_eq!(triple(4), 12);
        assert_eq!(triple(0), 0);
    }

    #[test]
    fn test_compose() {
        let add3_then_double = compose(make_adder(3), make_multiplier(2));
        assert_eq!(add3_then_double(4), 14); // (4+3)*2
        assert_eq!(add3_then_double(0), 6);  // (0+3)*2
    }
}
