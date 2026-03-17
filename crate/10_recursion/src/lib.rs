// 第10章: 再帰と末尾呼び出し

// ─── 基本的な再帰 ───────────────────────────────────────────

/// 素朴な再帰によるフィボナッチ数列（スタック消費が大きい）
pub fn fib_naive(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fib_naive(n - 1) + fib_naive(n - 2),
    }
}

// ─── 末尾再帰スタイル（累積引数） ────────────────────────────

/// 末尾再帰スタイルのフィボナッチ（累積引数を使用）
pub fn fib_tail(n: u64) -> u64 {
    fn go(n: u64, a: u64, b: u64) -> u64 {
        match n {
            0 => a,
            _ => go(n - 1, b, a + b),
        }
    }
    go(n, 0, 1)
}

/// 末尾再帰スタイルの階乗
pub fn factorial_tail(n: u64) -> u64 {
    fn go(n: u64, acc: u64) -> u64 {
        match n {
            0 | 1 => acc,
            _ => go(n - 1, n * acc),
        }
    }
    go(n, 1)
}

// ─── Trampoline パターン ──────────────────────────────────────

/// Trampoline: 継続を表す列挙型
pub enum Trampoline<T> {
    Done(T),
    More(Box<dyn FnOnce() -> Trampoline<T>>),
}

impl<T> Trampoline<T> {
    /// Trampoline を実行してスタックオーバーフローなく結果を得る
    pub fn run(self) -> T {
        let mut current = self;
        loop {
            match current {
                Trampoline::Done(value) => return value,
                Trampoline::More(thunk) => current = thunk(),
            }
        }
    }
}

/// Trampoline を使った大きな数の階乗
pub fn factorial_trampoline(n: u64) -> u64 {
    fn go(n: u64, acc: u64) -> Trampoline<u64> {
        match n {
            0 | 1 => Trampoline::Done(acc),
            _ => Trampoline::More(Box::new(move || go(n - 1, n * acc))),
        }
    }
    go(n, 1).run()
}

// ─── イテレータによる再帰の置き換え ──────────────────────────

/// イテレータを使ったフィボナッチ（再帰不要）
pub struct FibIter {
    a: u64,
    b: u64,
}

impl FibIter {
    pub fn new() -> Self {
        FibIter { a: 0, b: 1 }
    }
}

impl Default for FibIter {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for FibIter {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.a;
        let next = self.a + self.b;
        self.a = self.b;
        self.b = next;
        Some(result)
    }
}

// ─── 相互再帰 ────────────────────────────────────────────────

/// 相互再帰: 偶数判定
pub fn is_even(n: u32) -> bool {
    if n == 0 {
        true
    } else {
        is_odd(n - 1)
    }
}

/// 相互再帰: 奇数判定
pub fn is_odd(n: u32) -> bool {
    if n == 0 {
        false
    } else {
        is_even(n - 1)
    }
}

// ─── 木構造の再帰処理 ────────────────────────────────────────

#[derive(Debug)]
pub enum Tree<T> {
    Leaf,
    Node(T, Box<Tree<T>>, Box<Tree<T>>),
}

impl<T: std::fmt::Display + Clone> Tree<T> {
    /// 木の深さを再帰で計算
    pub fn depth(&self) -> usize {
        match self {
            Tree::Leaf => 0,
            Tree::Node(_, left, right) => 1 + left.depth().max(right.depth()),
        }
    }

    /// 木のノード数を再帰で計算
    pub fn count(&self) -> usize {
        match self {
            Tree::Leaf => 0,
            Tree::Node(_, left, right) => 1 + left.count() + right.count(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fib_naive() {
        assert_eq!(fib_naive(0), 0);
        assert_eq!(fib_naive(1), 1);
        assert_eq!(fib_naive(10), 55);
    }

    #[test]
    fn test_fib_tail() {
        assert_eq!(fib_tail(0), 0);
        assert_eq!(fib_tail(1), 1);
        assert_eq!(fib_tail(10), 55);
        assert_eq!(fib_tail(30), 832040);
    }

    #[test]
    fn test_factorial_tail() {
        assert_eq!(factorial_tail(0), 1);
        assert_eq!(factorial_tail(1), 1);
        assert_eq!(factorial_tail(5), 120);
        assert_eq!(factorial_tail(10), 3628800);
    }

    #[test]
    fn test_factorial_trampoline() {
        assert_eq!(factorial_trampoline(5), 120);
        assert_eq!(factorial_trampoline(10), 3628800);
        assert_eq!(factorial_trampoline(20), 2432902008176640000);
    }

    #[test]
    fn test_fib_iter() {
        let fibs: Vec<u64> = FibIter::new().take(8).collect();
        assert_eq!(fibs, vec![0, 1, 1, 2, 3, 5, 8, 13]);
    }

    #[test]
    fn test_mutual_recursion() {
        assert!(is_even(4));
        assert!(!is_even(3));
        assert!(is_odd(7));
        assert!(!is_odd(6));
    }

    #[test]
    fn test_tree() {
        let tree = Tree::Node(
            1,
            Box::new(Tree::Node(2, Box::new(Tree::Leaf), Box::new(Tree::Leaf))),
            Box::new(Tree::Leaf),
        );
        assert_eq!(tree.depth(), 2);
        assert_eq!(tree.count(), 2);
    }
}
