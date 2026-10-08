use http::StatusCode;

use crate::usecase::errors::UseCaseError;

pub fn map_uc_error(e: anyhow::Error, handler: &str) -> axum_anyhow::ApiError {
    let mut status_code = StatusCode::INTERNAL_SERVER_ERROR;
    let mut public_err = String::from("Internal error");
    let mut internal_err = format!("{e:?}"); // fallback: вся цепочка

    if let Some(uc) = e.chain().find_map(|c| c.downcast_ref::<UseCaseError>()) {
        match uc {
            UseCaseError::Transport {
                status_code: sc,
                public_err: pe,
                internal_err: ie,
            } => {
                status_code = *sc;
                public_err = pe.clone();

                if let Some(v) = ie {
                    internal_err = format!("{}; {}", internal_err, v.clone());
                    // } else {
                    //     internal_err.clear(); // ожидаемая бизнес-ошибка, без internal
                }
            }
            UseCaseError::UserNotFound => {
                status_code = StatusCode::NOT_FOUND;
                public_err = "user not found".into();
                // internal_err.clear();
            }
        }
    }

    log::error!("{}; {}", handler, internal_err);

    axum_anyhow::ApiError::builder()
        .status(status_code)
        .title(status_code.canonical_reason().unwrap_or("Error"))
        .detail(public_err)
        .build()
}
