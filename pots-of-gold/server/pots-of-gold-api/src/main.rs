use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder, http::header};
use rand::Rng;
use serde::{Deserialize, Serialize};
use shuttle_actix_web::ShuttleActixWeb;

#[derive(Serialize, Deserialize)]
struct GameRequest1 {
    potscount: u32,
}

#[derive(Serialize, Deserialize)]
struct GameResponse1 {
    pots: Vec<u32>,
}

#[derive(Serialize, Deserialize)]
struct GameRequest2 {
    pots: Vec<u32>,
    level: String,
}

#[derive(Serialize, Deserialize)]
struct GameResponse2 {
    chosen_index: usize,
}

async fn start_game(req: web::Json<GameRequest1>) -> impl Responder {
    let mut rng = rand::thread_rng();
    let pots: Vec<u32> = (0..req.potscount).map(|_| rng.gen_range(1..=9)).collect();

    HttpResponse::Ok().json(GameResponse1 {
        pots,
    })
}

fn optimal_move(pots: &[u32], level: &str) -> usize {
    let n = pots.len();
    if n == 1 { return 0; }
    let mut dp = vec![vec![0; n]; n];
    let mut sum = vec![0; n + 1];

    for i in 0..n {
        sum[i + 1] = sum[i] + pots[i];
    }

    for length in 1..=n {
        for left in 0..=n - length {
            let right = left + length - 1;
            if left == right {
                dp[left][right] = pots[left];
            } else {
                let pick_left = pots[left] + (sum[right + 1] - sum[left + 1] - dp[left + 1][right]);
                let pick_right = pots[right] + (sum[right] - sum[left] - dp[left][right - 1]);
                dp[left][right] = pick_left.max(pick_right);
            }
        }
    }

    let good_resp = if dp[1][n - 1] <= dp[0][n - 2] { 0 } else { n - 1 };
    let bad_resp = if good_resp == 0 { n - 1 } else { 0 };

    match level {
        "EASY" => bad_resp,
        "MEDIUM" => {
            let mut rng = rand::thread_rng();
            if rng.gen_bool(0.5) { good_resp } else { bad_resp }
        },
        _ => good_resp,
    }
}

async fn optimal_move_handler(req: web::Json<GameRequest2>) -> impl Responder {
    let chosen_index = optimal_move(&req.pots, &req.level);
    HttpResponse::Ok().json(GameResponse2 { chosen_index })
}

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut web::ServiceConfig) + Send + Clone + 'static> {
    let config = move |cfg: &mut web::ServiceConfig| {
        let cors = Cors::default()
            .allowed_origin("https://pots-of-gold.vercel.app")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
            .allowed_header(header::CONTENT_TYPE)
            .max_age(3600);

        cfg.service(
            web::scope("/api")
                .wrap(cors)
                .service(web::resource("/start-game").route(web::post().to(start_game)))
                .service(web::resource("/optimal-move").route(web::post().to(optimal_move_handler)))
        );
    };

    Ok(config.into())
}
