use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

pub fn hash_password(password: String) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_ref(), &salt)
        .expect("Hashing password failed!")
        .to_string()
}

pub fn validate_password(password: String, password_hash: String) -> bool {
    let parsed_hash = PasswordHash::new(&password_hash).expect("Parsing password hash failed!");
    let argon2 = Argon2::default();
    argon2
        .verify_password(password.as_ref(), &parsed_hash)
        .is_ok()
}
