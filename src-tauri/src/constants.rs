pub const DIR_CONVERSATION: &str = "conversations";
pub const DIR_ASSISTANTS: &str = "assistants";
pub const DIR_THREADS: &str = "threads";
pub const DIR_OPEN_AI_FILES: &str = "files";
pub const DIR_OPEN_AI_VECTORS: &str = "vectors";

pub const OPENAI_MAXIMUM_CONTENT_SIZE_BYTES: u64 = 26214400;
// 112674483
// pub const OPENAI_MAXIMUM_CONTENT_SIZE_BYTES: u64 = 15000000;

pub static PATH_LOGGING: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    if cfg!(debug_assertions) {
        std::env::current_exe()
            .unwrap()
            .with_file_name("log")
            .to_string_lossy()
            .to_string()
    } else {
        if cfg!(target_os = "macos") {
            //systemのログ置き場が、書き込み可能かチェックする（管理者権限でないと書き込めない）
            let dir_log_admin = "/Library/Logs/ai_client";
            if std::fs::create_dir_all(dir_log_admin)
                .ok()
                .and_then(|_| {
                    std::fs::write(std::path::Path::new(dir_log_admin).join("tmp"), "test").ok()
                })
                .is_some()
            {
                dir_log_admin.to_string()
            } else {
                //書き込み不可ならユーザーディレクトリに作成
                let dir_log_home = dirs::home_dir()
                    .unwrap()
                    .join("Library")
                    .join("Logs")
                    .join("ai_client");
                std::fs::create_dir_all(dir_log_home.as_path()).expect("can't create log dir");
                dir_log_home.to_string_lossy().to_string()
            }
        } else if cfg!(target_os = "linux") {
            "/var/log/ai_client".to_string()
        } else if cfg!(windows) {
            "C:\\ProgramData\\ai_client".to_string()
        } else {
            panic!("not supported os")
        }
    }
});
