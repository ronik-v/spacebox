use axum::{
    extract::{State, Json, Query, Path, Multipart},
    http::HeaderMap,
    routing::{post, delete, patch},
    Router,
};
use serde::Deserialize;
use utoipa::OpenApi;

use crate::{
    core::{
        api::{ApiResponse, ApiError},
        state::AppState,
        token::TokenExtractManager,
    },
    dto::storage::{FileDto, UserDirCreateResult, DirDto},
    repositories::users::UsersRepository,
    services::storage::StorageService,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::controllers::storage::upload_file,
        crate::controllers::storage::remove_file,
        crate::controllers::storage::create_directory,
        crate::controllers::storage::remove_directory,
        crate::controllers::storage::rename_directory
    ),
    components(
        schemas(
            UploadQuery,
            DirCreateRequest,
            DirRenameRequest,
            FileDto,
            UserDirCreateResult,
            DirDto,
            crate::core::api::ApiError
        )
    ),
    tags(
        (name = "storage", description = "Управление файлами и директориями в облачном хранилище")
    )
)]
pub struct StorageApiDoc;

pub fn storage_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/storage/upload", post(upload_file))
        .route("/api/v1/storage/file/{file_id}", delete(remove_file))
        .route("/api/v1/storage/dir", post(create_directory))
        .route(
            "/api/v1/storage/dir/{dir_id}",
            delete(remove_directory).patch(rename_directory),
        )
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct UploadForm {
    #[schema(format = Binary, content_media_type = "application/octet-stream")]
    pub file: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct UploadQuery {
    pub dir_id: i64,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct DirCreateRequest {
    pub name: String,
    pub parent_dir_id: Option<i64>
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct DirRenameRequest {
    pub new_name: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/storage/upload",
    tag = "storage",
    params(
        ("dir_id" = i64, Query, description = "ID директории (user_dir_id) для загрузки файла")
    ),
    request_body(
        content = UploadForm,
        content_type = "multipart/form-data",
        description = "Форма для загрузки файла"
    ),
    responses(
        (status = 200, description = "Файл успешно загружен", body = ApiResponse<FileDto>),
        (status = 400, description = "Ошибка загрузки", body = ApiResponse<String>),
        (status = 401, description = "Не авторизован", body = ApiError)
    ),
    security(
        ("bearerAuth" = [])
    )
)]
#[axum::debug_handler]
pub async fn upload_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<UploadQuery>,
    multipart: Multipart,
) -> ApiResponse<FileDto> {
    let token_manager = TokenExtractManager::new(headers);

    let token = match token_manager.check_token_error() {
        Ok(t) => t,
        Err(_) => {
            return ApiResponse::<FileDto>::Error {
                error: ApiError { message: "Invalid or expired token".to_string() },
            };
        }
    };

    let users_repo = UsersRepository::new(&state.db);

    let user_short = match users_repo.get_by_token(&token).await {
        Ok(u) => u,
        Err(_) => {
            return ApiResponse::<FileDto>::Error {
                error: ApiError { message: "Invalid or expired token".to_string() },
            };
        }
    };

    let storage_service = StorageService::new(user_short.id, &state.db, state.storage_root.clone());

    match storage_service.upload_file(multipart, "", query.dir_id).await {
        Ok(data) => ApiResponse::Success { data },
        Err(msg) => ApiResponse::<FileDto>::Error {
            error: ApiError { message: msg },
        },
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/storage/file/{file_id}",
    tag = "storage",
    params(
        ("file_id" = i64, Path, description = "ID файла для удаления")
    ),
    responses(
        (status = 200, description = "Файл успешно удален", body = ApiResponse<String>),
        (status = 400, description = "Ошибка удаления", body = ApiResponse<String>),
        (status = 401, description = "Не авторизован", body = ApiError)
    ),
    security(
        ("bearerAuth" = [])
    )
)]
#[axum::debug_handler]
pub async fn remove_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(file_id): Path<i64>,
) -> ApiResponse<String> {
    let token_manager = TokenExtractManager::new(headers);

    let token = match token_manager.check_token_error() {
        Ok(t) => t,
        Err(_) => {
            return ApiResponse::<String>::Error {
                error: ApiError { message: "Invalid or expired token".to_string() },
            };
        }
    };

    let users_repo = UsersRepository::new(&state.db);

    let user_short = match users_repo.get_by_token(&token).await {
        Ok(u) => u,
        Err(_) => {
            return ApiResponse::<String>::Error {
                error: ApiError { message: "Invalid or expired token".to_string() },
            };
        }
    };

    let storage_service = StorageService::new(user_short.id, &state.db, state.storage_root.clone());

    match storage_service.remove_file(file_id).await {
        Ok(_) => ApiResponse::Success { data: "deleted".to_string() },
        Err(msg) => ApiResponse::<String>::Error {
            error: ApiError { message: msg },
        },
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/storage/dir",
    tag = "storage",
    request_body(content = DirCreateRequest, description = "Имя новой директории (создается в корне пользователя)"),
    responses(
        (status = 200, description = "Директория успешно создана", body = ApiResponse<UserDirCreateResult>),
        (status = 400, description = "Ошибка создания", body = ApiResponse<String>),
        (status = 401, description = "Не авторизован", body = ApiError)
    ),
    security(
        ("bearerAuth" = [])
    )
)]
#[axum::debug_handler]
pub async fn create_directory(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<DirCreateRequest>,
) -> ApiResponse<UserDirCreateResult> {
    let token_manager = TokenExtractManager::new(headers);

    let token = match token_manager.check_token_error() {
        Ok(t) => t,
        Err(_) => {
            return ApiResponse::<UserDirCreateResult>::Error {
                error: ApiError { message: "Invalid or expired token".to_string() },
            };
        }
    };

    let users_repo = UsersRepository::new(&state.db);

    let user_short = match users_repo.get_by_token(&token).await {
        Ok(u) => u,
        Err(_) => {
            return ApiResponse::<UserDirCreateResult>::Error {
                error: ApiError { message: "Invalid or expired token".to_string() },
            };
        }
    };

    let storage_service = StorageService::new(user_short.id, &state.db, state.storage_root.clone());

    match storage_service.create_dir(&payload.name, &payload.parent_dir_id).await {
        Ok(data) => ApiResponse::Success { data },
        Err(msg) => ApiResponse::<UserDirCreateResult>::Error {
            error: ApiError { message: msg },
        },
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/storage/dir/{dir_id}",
    tag = "storage",
    params(
        ("dir_id" = i64, Path, description = "ID директории (user_dir_id) для удаления")
    ),
    responses(
        (status = 200, description = "Директория успешно удалена", body = ApiResponse<String>),
        (status = 400, description = "Ошибка удаления", body = ApiResponse<String>),
        (status = 401, description = "Не авторизован", body = ApiError)
    ),
    security(
        ("bearerAuth" = [])
    )
)]
#[axum::debug_handler]
pub async fn remove_directory(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dir_id): Path<i64>,
) -> ApiResponse<String> {
    let token_manager = TokenExtractManager::new(headers);

    let token = match token_manager.check_token_error() {
        Ok(t) => t,
        Err(_) => {
            return ApiResponse::<String>::Error {
                error: ApiError { message: "Invalid or expired token".to_string() },
            };
        }
    };

    let users_repo = UsersRepository::new(&state.db);

    let user_short = match users_repo.get_by_token(&token).await {
        Ok(u) => u,
        Err(_) => {
            return ApiResponse::<String>::Error {
                error: ApiError { message: "Invalid or expired token".to_string() },
            };
        }
    };

    let storage_service = StorageService::new(user_short.id, &state.db, state.storage_root.clone());

    match storage_service.remove_dir(dir_id).await {
        Ok(_) => ApiResponse::Success { data: "deleted".to_string() },
        Err(msg) => ApiResponse::<String>::Error {
            error: ApiError { message: msg },
        },
    }
}

#[utoipa::path(
    patch,
    path = "/api/v1/storage/dir/{dir_id}",
    tag = "storage",
    params(
        ("dir_id" = i64, Path, description = "ID директории (user_dir_id) для переименования")
    ),
    request_body(content = DirRenameRequest, description = "Новое имя директории"),
    responses(
        (status = 200, description = "Директория успешно переименована", body = ApiResponse<DirDto>),
        (status = 400, description = "Ошибка переименования", body = ApiResponse<String>),
        (status = 401, description = "Не авторизован", body = ApiError)
    ),
    security(
        ("bearerAuth" = [])
    )
)]
#[axum::debug_handler]
pub async fn rename_directory(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dir_id): Path<i64>,
    Json(payload): Json<DirRenameRequest>,
) -> ApiResponse<DirDto> {
    let token_manager = TokenExtractManager::new(headers);

    let token = match token_manager.check_token_error() {
        Ok(t) => t,
        Err(_) => {
            return ApiResponse::<DirDto>::Error {
                error: ApiError { message: "Invalid or expired token".to_string() },
            };
        }
    };

    let users_repo = UsersRepository::new(&state.db);

    let user_short = match users_repo.get_by_token(&token).await {
        Ok(u) => u,
        Err(_) => {
            return ApiResponse::<DirDto>::Error {
                error: ApiError { message: "Invalid or expired token".to_string() },
            };
        }
    };

    let storage_service = StorageService::new(user_short.id, &state.db, state.storage_root.clone());

    match storage_service.rename_dir(dir_id, &payload.new_name).await {
        Ok(data) => ApiResponse::Success { data },
        Err(msg) => ApiResponse::<DirDto>::Error {
            error: ApiError { message: msg },
        },
    }
}