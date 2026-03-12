use std::path::PathBuf;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

pub const OP_MULTIPART: &str = "get_next_field_multipart";
pub const OP_FILE_CREATE: &str = "create_file";
pub const OP_CHUNK_READ: &str = "read_chunk";
pub const OP_WRITE_CHUNK: &str = "read_chunk";
pub const OP_FILE_OPEN: &str = "open_file";
pub const OP_REMOVE_FILE: &str = "remove_file";
pub const OP_CREATE_DIR: &str = "create_dir";
pub const OP_REMOVE_DIR: &str = "remove_dir";
pub const OP_RENAME_DIR: &str = "change_dir";

pub const ERR_MULTIPART: &str = "Ошибка при получении следующего поля multipart";
pub const ERR_FILE_CREATE: &str = "Ошибка создания файла";
pub const ERR_CHUNK_READ: &str = "Ошибка чтения чанка файла";
pub const ERR_WRITE_CHUNK: &str = "Ошибка записи чанка в файл";
pub const ERR_FILE_OPEN: &str = "Ошибка открытия файла";
pub const ERR_REMOVE_FILE: &str = "Ошибка удаления файла";
pub const ERR_CREATE_DIR: &str = "Ошибка создания директории";
pub const ERR_REMOVE_DIR: &str = "Ошибка удаления директории";
pub const ERR_RENAME_DIR: &str = "Ошибка переименования директории";

#[derive(Debug)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: i64,
    pub hash: String,
    pub mime_type: Option<String>
}

#[derive(Debug)]
pub struct FileData {
    pub path: PathBuf,
    pub data: ReaderStream<File>,
}

#[derive(Debug, Clone)]
pub struct DirInfo {
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct StorageError {
    pub operation: String,
    pub message: String,
    pub path: String
}