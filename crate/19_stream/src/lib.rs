// --- Stream トレイト ---

pub trait Stream {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}

// --- コンビネータ型 ---

pub struct MapStream<S, F> {
    inner: S,
    f: F,
}

impl<S: Stream, B, F: FnMut(S::Item) -> B> Stream for MapStream<S, F> {
    type Item = B;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|item| (self.f)(item))
    }
}

pub struct FilterStream<S, F> {
    inner: S,
    predicate: F,
}

impl<S: Stream, F: FnMut(&S::Item) -> bool> Stream for FilterStream<S, F> {
    type Item = S::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next() {
                None => return None,
                Some(item) if (self.predicate)(&item) => return Some(item),
                Some(_) => continue,
            }
        }
    }
}

pub struct TakeStream<S> {
    inner: S,
    remaining: usize,
}

impl<S: Stream> Stream for TakeStream<S> {
    type Item = S::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;
        self.inner.next()
    }
}

pub struct EnumerateStream<S> {
    inner: S,
    index: usize,
}

impl<S: Stream> Stream for EnumerateStream<S> {
    type Item = (usize, S::Item);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|item| {
            let i = self.index;
            self.index += 1;
            (i, item)
        })
    }
}

// --- StreamExt: メソッドチェーン用拡張トレイト ---

pub trait StreamExt: Stream + Sized {
    fn map_stream<B, F: FnMut(Self::Item) -> B>(self, f: F) -> MapStream<Self, F> {
        MapStream { inner: self, f }
    }

    fn filter_stream<F: FnMut(&Self::Item) -> bool>(self, predicate: F) -> FilterStream<Self, F> {
        FilterStream { inner: self, predicate }
    }

    fn take_stream(self, n: usize) -> TakeStream<Self> {
        TakeStream { inner: self, remaining: n }
    }

    fn enumerate_stream(self) -> EnumerateStream<Self> {
        EnumerateStream { inner: self, index: 0 }
    }

    fn fold_stream<B, F: FnMut(B, Self::Item) -> B>(mut self, init: B, mut f: F) -> B {
        let mut acc = init;
        while let Some(item) = self.next() {
            acc = f(acc, item);
        }
        acc
    }

    fn collect_stream(mut self) -> Vec<Self::Item> {
        let mut result = Vec::new();
        while let Some(item) = self.next() {
            result.push(item);
        }
        result
    }
}

impl<S: Stream + Sized> StreamExt for S {}

// --- 具体的な Stream 実装 ---

/// 指定した範囲の整数を生成する Stream
pub struct RangeStream {
    current: u64,
    end: u64,
}

impl RangeStream {
    pub fn new(start: u64, end: u64) -> Self {
        RangeStream { current: start, end }
    }
}

impl Stream for RangeStream {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.end {
            let val = self.current;
            self.current += 1;
            Some(val)
        } else {
            None
        }
    }
}

/// フィボナッチ数列を無限に生成する Stream
pub struct FibStream {
    a: u64,
    b: u64,
}

impl FibStream {
    pub fn new() -> Self {
        FibStream { a: 0, b: 1 }
    }
}

impl Default for FibStream {
    fn default() -> Self {
        Self::new()
    }
}

impl Stream for FibStream {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.a;
        let next = self.a.saturating_add(self.b);
        self.a = self.b;
        self.b = next;
        Some(result)
    }
}

/// 前の値と現在の値をペアにして返す Stream
pub struct WindowsStream<S: Stream> {
    inner: S,
    prev: Option<S::Item>,
}

impl<S: Stream> WindowsStream<S>
where
    S::Item: Clone,
{
    pub fn new(mut inner: S) -> Self {
        let prev = inner.next();
        WindowsStream { inner, prev }
    }
}

impl<S: Stream> Stream for WindowsStream<S>
where
    S::Item: Clone,
{
    type Item = (S::Item, S::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.inner.next()?;
        let prev = self.prev.replace(current.clone())?;
        Some((prev, current))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_stream_basic() {
        let result = RangeStream::new(0, 5).collect_stream();
        assert_eq!(result, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_range_stream_empty() {
        let result = RangeStream::new(5, 5).collect_stream();
        assert!(result.is_empty());
    }

    #[test]
    fn test_map_stream() {
        let result = RangeStream::new(1, 6)
            .map_stream(|n| n * n)
            .collect_stream();
        assert_eq!(result, vec![1, 4, 9, 16, 25]);
    }

    #[test]
    fn test_filter_stream() {
        let result = RangeStream::new(0, 10)
            .filter_stream(|n| n % 2 == 0)
            .collect_stream();
        assert_eq!(result, vec![0, 2, 4, 6, 8]);
    }

    #[test]
    fn test_take_stream() {
        let result = RangeStream::new(0, 100)
            .take_stream(5)
            .collect_stream();
        assert_eq!(result, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_fold_stream() {
        let sum = RangeStream::new(1, 6).fold_stream(0, |acc, n| acc + n);
        assert_eq!(sum, 15);
    }

    #[test]
    fn test_pipeline() {
        // 0..100 の偶数を2倍して最初の5件の合計
        let result = RangeStream::new(0, 100)
            .filter_stream(|n| n % 2 == 0)
            .map_stream(|n| n * 2)
            .take_stream(5)
            .fold_stream(0, |acc, n| acc + n);
        // 0+4+8+12+16 = 40
        assert_eq!(result, 40);
    }

    #[test]
    fn test_fib_stream() {
        let result = FibStream::new().take_stream(10).collect_stream();
        assert_eq!(result, vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);
    }

    #[test]
    fn test_enumerate_stream() {
        let result = RangeStream::new(10, 13)
            .enumerate_stream()
            .collect_stream();
        assert_eq!(result, vec![(0, 10), (1, 11), (2, 12)]);
    }

    #[test]
    fn test_windows_stream() {
        let result = RangeStream::new(1, 6)
            .take_stream(5)
            .collect_stream()
            .into_iter()
            .collect::<Vec<_>>();

        // WindowsStream は Vec から作れないので RangeStream を使う
        let pairs: Vec<(u64, u64)> = WindowsStream::new(RangeStream::new(1, 6))
            .collect_stream();
        assert_eq!(pairs, vec![(1, 2), (2, 3), (3, 4), (4, 5)]);
        let _ = result; // suppress unused warning
    }

    #[test]
    fn test_fizzbuzz_multiples() {
        // 1から100の中で3の倍数かつ5の倍数の合計
        let sum = RangeStream::new(1, 101)
            .filter_stream(|n| n % 3 == 0 && n % 5 == 0)
            .fold_stream(0, |acc, n| acc + n);
        // 15+30+45+60+75+90 = 315
        assert_eq!(sum, 315);
    }

    #[test]
    fn test_fib_even_indexed() {
        // フィボナッチ数列の偶数番目（0-indexed）の値を10個取り出す
        let result: Vec<u64> = FibStream::new()
            .enumerate_stream()
            .filter_stream(|(i, _)| i % 2 == 0)
            .map_stream(|(_, n)| n)
            .take_stream(10)
            .collect_stream();
        // index 0,2,4,6,8,10,12,14,16,18 のフィボナッチ数
        // 0,1,3,8,21,55,144,377,987,2584
        assert_eq!(result, vec![0, 1, 3, 8, 21, 55, 144, 377, 987, 2584]);
    }
}
