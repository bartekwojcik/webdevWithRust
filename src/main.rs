use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{Error, ErrorKind};
use std::str::FromStr;
use warp::filters::query;
use warp::{
    filters::cors::CorsForbidden, http::Method, http::StatusCode, reject::Reject, Filter,
    Rejection, Reply,
};

#[derive(Debug, Serialize)]
pub struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct QuestionId(String);

#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    token: String,
}

#[derive(Debug, Serialize)]
struct Claims {
    sub: String,
    exp: usize,
}

impl Question {
    fn new(id: QuestionId, title: String, content: String, tags: Option<Vec<String>>) -> Self {
        Question {
            id,
            title,
            content,
            tags,
        }
    }
}

impl FromStr for QuestionId {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.is_empty() {
            false => Ok(QuestionId(s.to_string())),
            true => Err(Error::new(ErrorKind::InvalidInput, "no id provided")),
        }
    }
}

#[derive(Debug)]
struct InvalidId;
impl Reject for InvalidId {}

#[derive(Debug)]
struct InvalidCredentials;
impl Reject for InvalidCredentials {}

async fn get_questions() -> Result<impl Reply, Rejection> {
    let question = Question::new(
        QuestionId::from_str("1").expect("no id provided"),
        "First question".to_string(),
        "conttent".to_string(),
        Some(vec!["faq".to_string()]),
    );

    match question.id.0.parse::<i32>() {
        Err(_) => Err(warp::reject::custom(InvalidId)),
        Ok(_) => Ok(warp::reply::json(&question)),
    }
}

async fn login_handler(login: LoginRequest) -> Result<impl Reply, Rejection> {
    // In a real application, you would verify credentials against a database
    // For this example, we'll accept any username with password "password123"
    if login.password != "password123" {
        return Err(warp::reject::custom(InvalidCredentials));
    }

    // Create the JWT claims
    let claims = Claims {
        sub: login.username,
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    // Create the token
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("your-secret-key".as_ref()),
    )
    .map_err(|_| warp::reject::custom(InvalidCredentials))?;

    Ok(warp::reply::json(&LoginResponse { token }))
}

//todo dont know whats here
// async fn login_handler(body: Value) -> Result<impl Reply, Rejection> {
//     // Print the raw incoming body for debugging
//     println!("Received body: {}", body);

//     // Manually extract username and password
//     let username = body["username"].as_str().ok_or_else(|| {
//         println!("Failed to extract username");
//         warp::reject::custom(InvalidCredentials)
//     })?;

//     let password = body["password"].as_str().ok_or_else(|| {
//         println!("Failed to extract password");
//         warp::reject::custom(InvalidCredentials)
//     })?;

//     // Create LoginRequest manually
//     let login = LoginRequest {
//         username: username.to_string(),
//         password: password.to_string(),
//     };

//     // Rest of your existing authentication logic
//     if login.password != "password123" {
//         return Err(warp::reject::custom(InvalidCredentials));
//     }

//     // Create the JWT claims
//     let claims = Claims {
//         sub: login.username,
//         exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
//     };

//     // Create the token
//     let token = encode(
//         &Header::default(),
//         &claims,
//         &EncodingKey::from_secret("your-secret-key".as_ref()),
//     )
//     .map_err(|e| {
//         println!("Token encoding error: {:?}", e);
//         warp::reject::custom(InvalidCredentials)
//     })?;

//     Ok(warp::reply::json(&LoginResponse { token }))
// }

async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(error) = r.find::<CorsForbidden>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else if let Some(InvalidId) = r.find() {
        Ok(warp::reply::with_status(
            "no valid id".to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(InvalidCredentials) = r.find() {
        Ok(warp::reply::with_status(
            "invalid credentials".to_string(),
            StatusCode::UNAUTHORIZED,
        ))
    } else {
        eprintln!("unhandled rejection: {:?}", r);
        Ok(warp::reply::with_status(
            "route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}

#[tokio::main]
async fn main() {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["Content-Type", "Accept"])
        .allow_methods(&[Method::PUT, Method::DELETE, Method::GET, Method::POST]);

    let get_items = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and_then(get_questions);

    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and_then(login_handler);

    let routes = get_items.or(login).with(cors).recover(return_error);
    // let routes = get_items.or(login).recover(return_error);

    println!("Server started at http://127.0.0.1:3030");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
