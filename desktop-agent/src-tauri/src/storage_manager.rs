// LSN(Local Storage Manager) 모듈

use rusqlite::{Connection, Result, OptionalExtension};
use std::sync::{Arc, Mutex};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Runtime, Manager};
use std::time::{SystemTime, UNIX_EPOCH}; // cache_event 함수에 필요한 use 문

// lib.rs에서 ActiveSessionInfo
use crate::ActiveSessionInfo; 



// Local Storage Manager 구조체
pub struct StorageManager {
    conn: Mutex<Connection>,
}

// --- 1. 생성자 및 초기화 로직 ---
impl StorageManager {
    /// DB 연결 경로를 설정, 초기화
    pub fn new_from_path<R: Runtime>(app_handle: AppHandle<R>) -> Result<Self, String> {
        
        // OS 표준 데이터 경로
        let app_dir = app_handle.path().app_data_dir()
            .map_err(|e| format!("Failed to get application data directory from Tauri API: {}", e))?;

        let db_path = app_dir.join("local.db");

        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create app directory: {}", e))?;
            }
        }

        // 1.  Connection을 열고 초기화
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open DB connection: {}", e))?;
        // 에러 타입을 String으로 변환
        Self::initialize_db_with_conn(&conn)
            .map_err(|e| format!("DB Table init failed: {}", e))?;
        
        // 2. Mutex로 감싼 Connection을 저장
        Ok(StorageManager { conn: Mutex::new(conn) })
    }

    /// (테스트용) 인메모리 DB로 LSN을 생성
    #[cfg(test)]
    fn new_in_memory() -> Result<Self, String> {
        // 1. 인메모리 Connection 열기
        let conn = Connection::open_in_memory()
            .map_err(|e| format!("Failed to open in-memory DB: {}", e))?;
        
        // 2. 초기화
        Self::initialize_db_with_conn(&conn)
             .map_err(|e| format!("DB (in-memory) init failed: {}", e))?;

        // 3. Mutex로 감싼 Connection을 저장
        Ok(StorageManager { conn: Mutex::new(conn) })
    }

    fn initialize_db_with_conn(conn: &Connection) -> Result<(), String> {
        // --- 1. 활성 세션 정보 테이블 ---
        conn.execute(
            "CREATE TABLE IF NOT EXISTS active_session (
                session_id TEXT PRIMARY KEY,
                task_id TEXT NULL,
                start_time_s INTEGER NOT NULL
            )",
            [],
        ).map_err(|e| format!("Failed to create active_session table: {}", e))?;

        // --- 2. 캐시된 이벤트 데이터 테이블 ---
        conn.execute(
            "CREATE TABLE IF NOT EXISTS cached_events (
                id INTEGER PRIMARY KEY,
                session_id TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                app_name TEXT NOT NULL,
                window_title TEXT NOT NULL
            )",
            [],
        ).map_err(|e| format!("Failed to create cached_events table: {}", e))?;
        
        Ok(())
    }
}


// --- LSN 핵심 CRUD 함수  ---
impl StorageManager {
    /// 활성 세션 정보를 로컬 DB에 저장 (세션 시작 시 호출)
    pub fn save_active_session(&self, info: &crate::ActiveSessionInfo) -> Result<(), String>{
        // self.conn.lock()을 사용하여 공유된 연결에 접근
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        
        conn.execute("DELETE FROM active_session", []).map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO active_session (session_id, task_id, start_time_s) VALUES (?1, ?2, ?3)",
            rusqlite::params![
                info.session_id,
                info.task_id,
                info.start_time_s
            ],
        ).map_err(|e| e.to_string())?;
        
        Ok(())
    }
    
    /// 활성 세션 정보를 로컬 DB에서 읽기. (앱 시작 시 호출)
    pub fn load_active_session(&self) -> Result<Option<crate::ActiveSessionInfo>, String> {
        //  self.conn.lock()을 사용.
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn.prepare("SELECT session_id, task_id, start_time_s FROM active_session LIMIT 1")
            .map_err(|e| format!("SQL prepare error: {}", e))?;

        let row_result = stmt.query_row([], |row| {
            Ok(crate::ActiveSessionInfo {
                session_id: row.get(0)?,
                task_id: row.get(1)?, 
                start_time_s: row.get(2)?,
            })
        }).optional(); // 쿼리 결과가 없을 경우 None을 반환하도록 설정

        match row_result {
            Ok(Some(info)) => Ok(Some(info)),
            Ok(None) => Ok(None),
            Err(e) => Err(format!("Failed to load active session: {}", e)),
        }
    }
    
    /// 활성 세션 정보를 로컬 DB에서 삭제 (세션 종료 시 호출)
     pub fn delete_active_session(&self) -> Result<(), String> {
        // self.conn.lock()을 사용
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM active_session", []).map_err(|e| e.to_string())?;
        Ok(())
    }

    // 이벤트를 로컬 DB에 캐싱
    pub fn cache_event(&self, session_id: &str, app_name: &str, window_title: &str) -> Result<(), String> {
        // self.conn.lock()을 사용
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now_s = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();
        
        conn.execute(
            "INSERT INTO cached_events (session_id, timestamp, app_name, window_title) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                session_id,
                now_s,
                app_name,
                window_title
            ],
        ).map_err(|e| e.to_string())?;
        
        Ok(())
    }
}
// --- 유닛 테스트 모듈 ---
#[cfg(test)]
mod tests {
    use super::*;
    // 테스트용 lib.rs의 ActiveSessionInfo
    use crate::ActiveSessionInfo; 

    fn setup_test_db() -> StorageManager {
        StorageManager::new_in_memory().expect("Failed to create in-memory DB")
    }

    #[test]
    fn test_save_and_load_session() {
        let storage = setup_test_db();
        
        let info = ActiveSessionInfo {
            session_id: "test-session-123".to_string(),
            task_id: Some("test-task-456".to_string()),
            start_time_s: 123456789,
        };
        
        // 1. 저장 테스트
        storage.save_active_session(&info).expect("Failed to save session");
        
        // 2. 로드 테스트
        let loaded_info = storage.load_active_session()
            .expect("Failed to load session")
            .expect("Session not found after saving");
        
        assert_eq!(loaded_info.session_id, info.session_id);
        assert_eq!(loaded_info.task_id, info.task_id);
        assert_eq!(loaded_info.start_time_s, info.start_time_s);
    }

    #[test]
    fn test_delete_session() {
         let storage = setup_test_db();
        
        let info = ActiveSessionInfo {
            session_id: "test-session-123".to_string(),
            task_id: None,
            start_time_s: 123456789,
        };

        // 1. 저장
        storage.save_active_session(&info).expect("Failed to save session");
        let loaded_info = storage.load_active_session().unwrap();
        assert!(loaded_info.is_some()); // 저장 확인
        
        // 2. 삭제
        storage.delete_active_session().expect("Failed to delete session");

        // 3. 로드 확인 (None이어야 함)
        let loaded_info_after_delete = storage.load_active_session().unwrap();
        assert!(loaded_info_after_delete.is_none());
    }

    #[test]
    fn test_cache_event() {
        let storage = setup_test_db();
        
        storage.cache_event("session-1", "chrome.exe", "YouTube")
            .expect("Failed to cache event 1");
        storage.cache_event("session-1", "code.exe", "lib.rs")
            .expect("Failed to cache event 2");

        //  conn.lock()을 통해 Connection에 접근
        let conn = storage.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM cached_events WHERE session_id = 'session-1'").unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        
        assert_eq!(count, 2);
    }
}