use utoipa::{OpenApi, Modify};
use utoipa::openapi::ComponentsBuilder;
use utoipa::openapi::security::{SecurityScheme, HttpBuilder, HttpAuthScheme};
use bcrypt::{hash, verify, DEFAULT_COST};
use rand::distr::{Distribution, Uniform};
use sqlx::PgPool;
use zeroize::Zeroizing;
use crate::repositories::users::UsersRepository;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let http = HttpBuilder::new()
            .scheme(HttpAuthScheme::Bearer)
            .description(Some("Формат: Bearer <токен>".to_string()))
            .build();
        let scheme = SecurityScheme::Http(http);

        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme("bearerAuth", scheme);
        } else {
            openapi.components = Some(
                ComponentsBuilder::new()
                    .security_scheme("bearerAuth", scheme)
                    .build(),
            );
        }
    }
}

pub struct Password {
    pub pass: String
}

impl Password {
    pub fn new(pass: String) -> Self { Self { pass } }

    pub fn hash(&self) -> String {
        hash(self.pass.clone(), DEFAULT_COST).unwrap()
    }

    pub fn is_correct(&self, stored_hash: &str) -> bool {
        verify(self.pass.clone(), stored_hash).unwrap_or(false)
    }
}


pub struct AppAdmin<'a> {
    db: &'a PgPool
}

impl<'a> AppAdmin<'a> {
    pub(crate) fn new(db: &'a PgPool) -> Self { Self { db } }

    fn generate_random_string(charset: &[u8], len: usize) -> String {
        let uniform = Uniform::new(0, charset.len()).expect("Charset must not be empty");
        let mut rng = rand::rng();

        (0..len)
            .map(|_| charset[uniform.sample(&mut rng)] as char)
            .collect()
    }

    fn generate_random_password(&self) -> String {
        const PASSWORD_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                          abcdefghijklmnopqrstuvwxyz\
                                          0123456789!@#$%^&*()-_=+[]{}";
        Self::generate_random_string(PASSWORD_CHARSET, 12)
    }

    fn generate_random_username(&self) -> String {
        const LOGIN_CHARSET: &[u8] = b"abcdefghjkmnpqrstuvwxyz23456789";
        Self::generate_random_string(LOGIN_CHARSET, 8)
    }

    pub async fn create(&self) {
        let repo = UsersRepository::new(self.db);

        let has_admin = match repo.has_admin().await {
            Ok(exists) => exists,
            Err(e) => {
                tracing::error!("[DB error] check if admin exists - {e}");
                return;
            }
        };

        if !has_admin {
            let admin_role_id = match repo.get_role_id_by_name("admin").await {
                Ok(id) => id,
                Err(e) => {
                    tracing::error!("[DB error] get admin role id - {e}");
                    return;
                }
            };

            let admin_name = self.generate_random_username();
            let admin_raw_password = self.generate_random_password();
            let pass_manager = Password::new(admin_raw_password.clone());
            let admin_password_hash = pass_manager.hash();

            match repo.add(&admin_name, &admin_password_hash).await {
                Ok(Some(user)) => {
                    if let Err(e) = repo.add_role(user.id, admin_role_id).await {
                        tracing::error!("[DB error] add role for base admin - {e}");
                        return;
                    }
                    println!("[SUCCESS] Admin was created...");
                    println!("[ADMIN] Creds | LOGIN = {admin_name} | PASSWORD = {admin_raw_password}");
                }
                Ok(None) => {
                    tracing::error!("[FAIL] Admin user was not returned after insert");
                }
                Err(e) => {
                    tracing::error!("[FAIL] Create base app admin error: {e}");
                }
            }
        }
    }
}