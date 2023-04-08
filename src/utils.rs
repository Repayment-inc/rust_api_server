use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::{
    async_trait,
    headers::{authorization::Bearer, Authorization},
    extract::{FromRequest, FromRequestParts, TypedHeader},
    http::{StatusCode, Request, request::Parts},
};

use jsonwebtoken::{decode, Validation};

use crate::{error::AppError, models::auth::Claims, KEYS};

// get 8 hours timestamp for jwt expiry
pub fn get_timestamp_8_hours_from_now() -> u64 {
    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    let eighthoursfromnow = since_the_epoch + Duration::from_secs(28800);
    eighthoursfromnow.as_secs()
    
}

// トークンを検証し、そこからデータを抽出 (一種のミドルウェア)。
// ハンドルでクレームを抽出しようとすると、最初にこのコードが実行される
#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts, _state: &S
    ) -> Result<Self, Self::Rejection> {
        // 認証ヘッダーからトークンを抽出する
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, _state)
                .await
                .map_err(|_| AppError::InvalidToken)?;
        let data = decode::<Claims>(bearer.token(), &KEYS.decoding, &Validation::default())
            .map_err(|_| AppError::InvalidToken)?;
        Ok(data.claims)
    }
}

