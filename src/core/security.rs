use utoipa::{OpenApi, Modify};
use utoipa::openapi::ComponentsBuilder;
use utoipa::openapi::security::{SecurityScheme, HttpBuilder, HttpAuthScheme};
use bcrypt::{hash, verify, DEFAULT_COST};

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