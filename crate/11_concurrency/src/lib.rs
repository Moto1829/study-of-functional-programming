// 第11章: 並行処理と関数型スタイル

use std::sync::{Arc, Mutex};
use std::thread;

// ─── Arc による不変データの安全な共有 ────────────────────────

/// Arc<T> で複数スレッド間に不変データを共有する例
pub fn arc_shared_data() -> Vec<i32> {
    let data = Arc::new(vec![1, 2, 3, 4, 5]);
    let mut handles = vec![];

    for i in 0..3 {
        let data = Arc::clone(&data);
        let handle = thread::spawn(move || data[i]);
        handles.push(handle);
    }

    handles.into_iter().map(|h| h.join().unwrap()).collect()
}

// ─── チャネルによるメッセージパッシング ──────────────────────

/// mpsc チャネルで値を送受信する
pub fn channel_sum(values: Vec<i32>) -> i32 {
    let (tx, rx) = std::sync::mpsc::channel();

    for v in values {
        let tx = tx.clone();
        thread::spawn(move || {
            tx.send(v).unwrap();
        });
    }
    // 元の tx を drop して受信ループが終了できるようにする
    drop(tx);

    rx.into_iter().sum()
}

// ─── Mutex と関数型スタイルの組み合わせ ──────────────────────

/// Mutex で保護されたカウンターを複数スレッドからインクリメント
pub fn mutex_counter(n_threads: usize) -> i32 {
    let counter = Arc::new(Mutex::new(0i32));
    let mut handles = vec![];

    for _ in 0..n_threads {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut c = counter.lock().unwrap();
            *c += 1;
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    let result = *counter.lock().unwrap();
    result
}

// ─── 不変データ構造によるデータ共有 ──────────────────────────

/// イミュータブルな設定データをスレッド間で共有
#[derive(Debug, Clone)]
pub struct Config {
    pub max_connections: u32,
    pub timeout_ms: u64,
    pub host: String,
}

pub fn process_with_config(config: Config, tasks: Vec<u32>) -> Vec<u32> {
    let config = Arc::new(config);
    let mut handles = vec![];

    for task in tasks {
        let cfg = Arc::clone(&config);
        let handle = thread::spawn(move || {
            // 設定を参照しながら処理（不変なので安全）
            if task <= cfg.max_connections {
                task * 2
            } else {
                0
            }
        });
        handles.push(handle);
    }

    handles.into_iter().map(|h| h.join().unwrap()).collect()
}

// ─── fold によるスレッド結果の集約 ───────────────────────────

/// 各スレッドの部分結果を fold で集約するパターン
pub fn parallel_fold(chunks: Vec<Vec<i32>>) -> i32 {
    let handles: Vec<_> = chunks
        .into_iter()
        .map(|chunk| thread::spawn(move || chunk.into_iter().sum::<i32>()))
        .collect();

    handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .fold(0, |acc, x| acc + x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arc_shared_data() {
        let result = arc_shared_data();
        assert_eq!(result.len(), 3);
        // 各スレッドが data[0], data[1], data[2] を取得
        assert!(result.contains(&1));
        assert!(result.contains(&2));
        assert!(result.contains(&3));
    }

    #[test]
    fn test_channel_sum() {
        let result = channel_sum(vec![1, 2, 3, 4, 5]);
        assert_eq!(result, 15);
    }

    #[test]
    fn test_mutex_counter() {
        let result = mutex_counter(10);
        assert_eq!(result, 10);
    }

    #[test]
    fn test_process_with_config() {
        let config = Config {
            max_connections: 3,
            timeout_ms: 1000,
            host: "localhost".to_string(),
        };
        let mut result = process_with_config(config, vec![1, 2, 3, 4, 5]);
        result.sort();
        assert_eq!(result, vec![0, 0, 2, 4, 6]);
    }

    #[test]
    fn test_parallel_fold() {
        let chunks = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
        let result = parallel_fold(chunks);
        assert_eq!(result, 45);
    }
}
