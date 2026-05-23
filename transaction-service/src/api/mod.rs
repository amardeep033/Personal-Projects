use actix_web::web;
use tracing::info;

use crate::{
    api::{
        create_account::create_account, create_transaction::create_transaction,
        get_account::get_account, health::health,
    },
    constants::{ROUTE_ACCOUNTS, ROUTE_ACCOUNT_BY_ID, ROUTE_HEALTH, ROUTE_TRANSACTIONS},
};

pub mod create_account;
pub mod create_transaction;
pub mod get_account;
pub mod health;

pub fn handlers(cfg: &mut web::ServiceConfig) {
    info!("registering public routes");
    cfg.service(web::resource(ROUTE_HEALTH).route(web::get().to(health)));
}

pub fn protected_handlers(cfg: &mut web::ServiceConfig) {
    info!("registering protected routes");
    cfg.route(ROUTE_ACCOUNTS, web::post().to(create_account))
        .route(ROUTE_ACCOUNT_BY_ID, web::get().to(get_account))
        .route(ROUTE_TRANSACTIONS, web::post().to(create_transaction));
}
