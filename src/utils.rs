use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

pub fn hash_password<'a>(password: &'a [u8]) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password, &salt)
        .expect("Hashing password failed!")
        .to_string()
}

pub fn validate_password<'a>(password: &'a [u8], password_hash: &'a str) -> bool {
    let parsed_hash = PasswordHash::new(password_hash).expect("Parsing password hash failed!");
    let argon2 = Argon2::default();
    argon2.verify_password(password, &parsed_hash).is_ok()
}
