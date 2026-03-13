use utoipa::{OpenApi, Modify};
use utoipa::openapi::ComponentsBuilder;
use utoipa::openapi::security::{SecurityScheme, HttpBuilder, HttpAuthScheme};

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