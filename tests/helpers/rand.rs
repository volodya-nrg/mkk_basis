use mkk_basis::adapter::db::models::{Task, TaskComment, TaskHistory, Team, TeamMember, User};
use mkk_basis::adapter::db::postgres::tables::tasks::Status as TaskStatuses;
use mkk_basis::transport::models::{
    RequestLogin, RequestRefreshToken, RequestRegister, RequestTask, RequestTeamCreate,
    RequestTeamInvite, RequestUser,
};
use rand::{Rng, RngExt, distr::Alphanumeric};
use uuid::Uuid;

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
pub fn email() -> String {
    format!("{}@{}.{}", str_limit(10), str_limit(10), str_limit(3))
}
pub fn bool() -> bool {
    rand::random()
}
pub fn int_range(min: usize, max: usize) -> usize {
    let mut rng = rand::rng();
    rng.random_range(min..=max) // включительно
}

pub fn request_register() -> RequestRegister {
    let pass = str();
    RequestRegister {
        email: email(),
        password: pass.clone(),
        password_confirm: pass,
        agreement: true,
        privacy_policy: true,
    }
}

pub fn request_login() -> RequestLogin {
    RequestLogin {
        email: email(),
        password: str(),
    }
}

pub fn request_team_create() -> RequestTeamCreate {
    RequestTeamCreate {
        name: str(),
        created_by: Uuid::new_v4(),
    }
}

pub fn request_team_invite() -> RequestTeamInvite {
    RequestTeamInvite {
        user_id: Uuid::new_v4(),
    }
}

pub fn request_task() -> RequestTask {
    RequestTask {
        name: str(),
        description: if bool() { Some(str()) } else { None },
        created_by: Uuid::new_v4(),
        team_id: Uuid::new_v4(),
        assignee_id: if bool() { Some(Uuid::new_v4()) } else { None },
        status: get_random_task_status(),
    }
}

pub fn request_user() -> RequestUser {
    RequestUser {
        name: if bool() { Some(str()) } else { None },
        email: email(),
        password: str(),
        email_code: if bool() { Some(str()) } else { None },
        role: if bool() { Some(str()) } else { None },
    }
}

pub fn request_refresh_token() -> RequestRefreshToken {
    RequestRefreshToken { token: str() }
}

pub fn user() -> User {
    User {
        user_id: Uuid::new_v4(),
        name: if bool() { Some(str()) } else { None },
        email: email(),
        password: str(),
        email_code: if bool() { Some(str()) } else { None },
        avatar: if bool() { Some(str()) } else { None },
        role: if bool() { Some(str()) } else { None },
        created_at: Default::default(),
        updated_at: Default::default(),
    }
}
pub fn team() -> Team {
    Team {
        team_id: Uuid::new_v4(),
        name: str(),
        created_by: Uuid::new_v4(),
        created_at: Default::default(),
        updated_at: Default::default(),
    }
}
pub fn team_member() -> TeamMember {
    TeamMember {
        team_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        created_at: Default::default(),
    }
}
pub fn task() -> Task {
    Task {
        task_id: Uuid::new_v4(),
        name: str(),
        description: if bool() { Some(str()) } else { None },
        created_by: Uuid::new_v4(),
        team_id: Uuid::new_v4(),
        assignee_id: if bool() { Some(Uuid::new_v4()) } else { None },
        status: get_random_task_status(),
        created_at: Default::default(),
        updated_at: Default::default(),
    }
}
pub fn task_history() -> TaskHistory {
    TaskHistory {
        task_history_id: Uuid::new_v4(),
        task_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        msg: str(),
        created_at: Default::default(),
    }
}
pub fn task_comment() -> TaskComment {
    TaskComment {
        task_comment_id: Uuid::new_v4(),
        task_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        msg: str(),
        created_at: Default::default(),
        updated_at: Default::default(),
    }
}

fn get_random_task_status() -> String {
    let statuses = [
        TaskStatuses::Start,
        TaskStatuses::Todo,
        TaskStatuses::Done,
        TaskStatuses::Cancelled,
    ];
    statuses[int_range(0, statuses.len() - 1)].to_string()
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
