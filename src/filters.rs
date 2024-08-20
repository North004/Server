use crate::model::{User};
use serde::{Deserialize,Serialize};
use chrono::{DateTime,Utc};

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Clone)]
pub struct FilterdUser {
    pub id: uuid::Uuid,
    pub username: String,
}



pub fn filter_user(user: &User) -> FilterdUser {
    let filterd_user = FilterdUser {
        id: user.id,
        username: user.username.clone()
    };
    filterd_user
}