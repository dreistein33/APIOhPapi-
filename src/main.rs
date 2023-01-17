use actix_web::{get, HttpResponse, HttpServer, Responder, App, web, post};
use actix_web::http::header::ContentType;

use serde::{Serialize, Deserialize};
use serde_json;

use std::{fs, str::FromStr};
use std::io::{BufWriter};
use std::char;

use hex_literal::hex;
use sha3::{Digest, Sha3_256};

static FILE: &'static str = "dzejson.json";

enum InputError {
    EmptyFieldsError(serde_json::Value),
    InvalidCharacterError(serde_json::Value),
    InvalidLengthError(serde_json::Value),
    UsernameTakenError(serde_json::Value)
}

trait InputErrorMethods {
    fn extract_value(&self) -> &serde_json::Value;
}

impl InputErrorMethods for InputError {
    fn extract_value(&self) -> &serde_json::Value {
        match &self {
            Self::EmptyFieldsError(e) => e,
            Self::InvalidCharacterError(e) => e,
            Self::InvalidLengthError(e) => e,
            Self::UsernameTakenError(e) => e
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
    #[serde(default)]
    username: String,
    #[serde(default)]
    password: String,
}

trait UserValidation {
    fn check_pwd(&self, hashed_pwd: String) -> bool;
}

trait UserHashFunctions {
    fn hash_password(&mut self);
}

impl UserHashFunctions for User {
    fn hash_password(&mut self) {
        let mut sha256 = Sha3_256::new();

        sha256.update(&self.password);

        let user_password_hash = format!("{:X}", sha256.finalize());

        self.password = user_password_hash;
    }
}

impl UserValidation for User {
    fn check_pwd(&self, hashed_pwd: String) -> bool {
        if self.password.eq(&hashed_pwd) {
            return true;
        } else {
            return false;
        }
    }
}

fn define_error_when_registering(user: &User) -> Result<(), InputError> {

    if user.username.is_empty() || user.password.is_empty() {
        return Err(InputError::EmptyFieldsError(serde_json::json!({"e": "Cannot pass empty username nor password."})));
    }

    if !user.username.chars().all(char::is_alphanumeric) {
        return Err(InputError::InvalidCharacterError(serde_json::json!({"e": "Special characters used in username."})));
    }

    if user.username.len() < 5 || user.password.len() < 5 {
        return Err(InputError::InvalidLengthError(serde_json::json!({"e": "Either password and username must be longer than 5 characters."})));
    }

    match check_if_username_exists(user.username.clone()) {
        Ok(_) => return Err(InputError::UsernameTakenError(serde_json::json!({"e": "Username is taken."}))),
        Err(_error) => () 
    }

    Ok(())
}

fn define_error_when_loging_in() {
    todo!();
}

fn add_user_to_file(users: Vec<User>) -> std::io::Result<()> {
    let file = fs::File::options()
        .write(true)
        .truncate(true)
        .open(FILE)?;

    let writer = BufWriter::new(file);

    serde_json::to_writer(writer, &users)?;

    Ok(())

}

fn create_vector_out_of_json_file() -> Vec<User> {
    let content = fs::read_to_string(FILE).unwrap();

    let content_to_value = serde_json::Value::from_str(&content).unwrap();

    let mut  user_vector: Vec<User> = Vec::new();

    if let Some(array) = content_to_value.as_array() {
        for credential in array {
            let user = User {
                username: credential["username"].as_str().unwrap().to_owned(),
                password: credential["password"].as_str().unwrap().to_owned()
            };
            user_vector.push(user);
        }
    }

    user_vector
}

// Check if username exists and if so return that specific user of User type
fn check_if_username_exists(username: String) -> Result<User, String> {
    let mut taken = false;

    let user_info_str = fs::read_to_string(FILE).unwrap();

    let user_info_json = serde_json::Value::from_str(&user_info_str).unwrap();

    if let Some(array) = user_info_json.as_array() {
        for credential in array {
            if credential["username"] == username {
                let user_found= User {
                    username: credential["username"].as_str().unwrap().to_owned(),
                    password: credential["password"].as_str().unwrap().to_owned()
                };
                return Ok(user_found);
            }
        }
    }

    Err("No user found".to_string())
    
}

#[get("/users")]
async fn get_user_info() -> impl Responder {
    let user_info = fs::read_to_string(FILE).unwrap();

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(user_info)
}

#[post("/login")]
async fn login_user(user: web::Json<User>) -> impl Responder {
    let mut unpacked_user = user.into_inner();

    unpacked_user.hash_password();

    let mut password_matching = false;

    match check_if_username_exists(unpacked_user.username) {
        Ok(user_found) => {
            println!("{:?}", user_found.password);
            println!("{:?}", unpacked_user.password);
            if user_found.password == unpacked_user.password {
                password_matching = true;
            }
        },
        Err(error) => ()
    }

    if password_matching {
        return HttpResponse::Ok()
            .content_type(ContentType::json())
            .body("Succes")
    }

    return HttpResponse::InternalServerError()
        .body("Bad data")

    
}

#[post("/register")]
async fn register_user(user: web::Json<User>) -> impl Responder{

    let mut unpacked_user = user.into_inner();

    unpacked_user.hash_password();

    let mut create_user = false;

    match define_error_when_registering(&unpacked_user) {
        Ok(_) => create_user = true,
        Err(error_msg) => return HttpResponse::InternalServerError()
            .content_type(ContentType::json())
            .body(error_msg.extract_value().to_string())
    }

    if create_user {
        let mut user_vector = create_vector_out_of_json_file();
        user_vector.push(unpacked_user);
        add_user_to_file(user_vector).unwrap();
    }
  
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body("Success!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    HttpServer::new(move ||
        App::new()
            .service(get_user_info)
            .service(register_user)
            .service(login_user)
        )
        .bind(("127.0.0.1", 3333))?
        .run()
        .await

}