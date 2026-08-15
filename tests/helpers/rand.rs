use rand::{Rng, RngExt, distr::Alphanumeric}; // distr::Alphabetic,

pub fn private_key(len: usize) -> Vec<u8> {
    let mut key = vec![0u8; len];
    rand::rng().fill_bytes(&mut key);
    key
}
pub fn str() -> String {
    str_limit(20)
}
pub fn str_limit(len: usize) -> String {
    // rand::rng() - вызовется в своем потоке. У него локальный итератор.
    // Нельзя его создать в одном потоке "let x = rand::rng()", а потом этот x (генератор) вызывать в другом потоке.
    rand::rng()
        .sample_iter(Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use tokio::sync::mpsc as TokioMPSC;
    use tokio::task;

    #[test]
    fn check_random_via_os_thread() {
        const LIMIT: usize = 100;
        let (tx, rx) = mpsc::channel();
        // let (tx, rx) = mpsc::sync_channel(10);
        let mut handles = Vec::with_capacity(LIMIT);
        for _ in 0..LIMIT {
            // clone отправит данные в оригинальный канал (tx), после уничтожится
            let tx_clone = tx.clone();
            handles.push(std::thread::spawn(move || tx_clone.send(str()).unwrap()))
        }
        drop(tx); // закрываем оригинальный отправитель
        // Ждем завершения всех потоков. При буферизированном нужно наоборот, чтоб освобождать буфер.
        for handle in handles {
            handle.join().unwrap(); // ждем завершения конкретного потока
        }
        let mut rcv: Vec<String> = (0..LIMIT).map(|_| "".to_string()).collect(); // обязательно нужно создать данные
        // считываем данные. rx.iter().collect::<Vec<String>>()
        for (i, v) in rx.iter().enumerate() {
            rcv[i] = v;
        }
        assert_eq!(LIMIT, rcv.len())
    }

    #[tokio::test]
    async fn check_random_via_tokio_thread() {
        const LIMIT: usize = 100;
        // let (tx, mut rx) = TokioMPSC::channel(32);
        let (tx, mut rx) = TokioMPSC::unbounded_channel();
        let mut handles = Vec::with_capacity(LIMIT);
        for _ in 0..LIMIT {
            let tx_clone = tx.clone();
            let handle = task::spawn(async move { tx_clone.send(str()).unwrap() });
            handles.push(handle);
        }
        drop(tx);
        let mut rcv: Vec<String> = (0..LIMIT).map(|_| "".to_string()).collect(); // обязательно нужно создать данные
        let mut i = 0;
        while let Some(v) = rx.recv().await {
            rcv[i] = v;
            i = i + 1;
        }
        for handle in handles {
            handle.await.unwrap()
        }
        assert_eq!(LIMIT, rcv.len())
    }
}
