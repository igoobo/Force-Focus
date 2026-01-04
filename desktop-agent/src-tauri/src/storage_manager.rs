// LSN(Local Storage Manager) 모듈

/*
Table에 새로운 데이터 추가는 commands.rs에서 이루어짐
*/

use rusqlite::{Connection, OptionalExtension, Result};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager, Runtime}; // cache_event 함수에 필요한 use 문

// lib.rs
use crate::commands::InputStats;
use crate::ActiveSessionInfo;
use crate::LoggableEventData;

// 동기화할 이벤트 데이터 구조체 (public)
#[derive(Debug)]
pub struct CachedEvent {
    pub id: i64,
    pub session_id: String,
    pub timestamp: i64,
    pub app_name: String,
    pub window_title: String,
    pub activity_vector: String, // JSON String
}

// Local Storage Manager 구조체
pub struct StorageManager {
    conn: Mutex<Connection>,
}

// --- 1. 생성자 및 초기화 로직 ---
impl StorageManager {
    /// DB 연결 경로를 설정, 초기화
    pub fn new_from_path<R: Runtime>(app_handle: AppHandle<R>) -> Result<Self, String> {
        // OS 표준 데이터 경로
        let app_dir = app_handle.path().app_data_dir().map_err(|e| {
            format!(
                "Failed to get application data directory from Tauri API: {}",
                e
            )
        })?;

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
        Self::initialize_db_with_conn(&conn).map_err(|e| format!("DB Table init failed: {}", e))?;

        // 2. Mutex로 감싼 Connection을 저장
        Ok(StorageManager {
            conn: Mutex::new(conn),
        })
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
        Ok(StorageManager {
            conn: Mutex::new(conn),
        })
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
        )
        .map_err(|e| format!("Failed to create active_session table: {}", e))?;

        // --- 2. 캐시된 이벤트 데이터 테이블 ---
        conn.execute(
            "CREATE TABLE IF NOT EXISTS cached_events (
                id INTEGER PRIMARY KEY,
                session_id TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                app_name TEXT NOT NULL,       
                window_title TEXT NOT NULL,   
                activity_vector TEXT NOT NULL -- JSON
            )",
            [],
        )
        .map_err(|e| format!("Failed to create cached_events table: {}", e))?;

        // 3. 피드백 캐싱 테이블
        conn.execute(
            "CREATE TABLE IF NOT EXISTS cached_feedback (
                id INTEGER PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                event_id TEXT NOT NULL,
                feedback_type TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| format!("Failed to create cached_feedback table: {}", e))?;

        // 4. 인증 토큰 테이블
        conn.execute(
            "CREATE TABLE IF NOT EXISTS auth_token (
                id INTEGER PRIMARY KEY CHECK (id = 1), 
                access_token TEXT NOT NULL,
                refresh_token TEXT NOT NULL,
                user_email TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )
        .map_err(|e| format!("Failed to create auth_token table: {}", e))?;

        Ok(())
    }
}

// --- LSN 핵심 CRUD 함수  ---
impl StorageManager {
    /// 활성 세션 정보를 로컬 DB에 저장 (세션 시작 시 호출)
    pub fn save_active_session(&self, info: &crate::ActiveSessionInfo) -> Result<(), String> {
        // self.conn.lock()을 사용하여 공유된 연결에 접근
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute("DELETE FROM active_session", [])
            .map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO active_session (session_id, task_id, start_time_s) VALUES (?1, ?2, ?3)",
            rusqlite::params![info.session_id, info.task_id, info.start_time_s],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// 활성 세션 정보를 로컬 DB에서 읽기. (앱 시작 시 호출)
    pub fn load_active_session(&self) -> Result<Option<crate::ActiveSessionInfo>, String> {
        //  self.conn.lock()을 사용.
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare("SELECT session_id, task_id, start_time_s FROM active_session LIMIT 1")
            .map_err(|e| format!("SQL prepare error: {}", e))?;

        let row_result = stmt
            .query_row([], |row| {
                Ok(crate::ActiveSessionInfo {
                    session_id: row.get(0)?,
                    task_id: row.get(1)?,
                    start_time_s: row.get(2)?,
                })
            })
            .optional(); // 쿼리 결과가 없을 경우 None을 반환하도록 설정

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
        conn.execute("DELETE FROM active_session", [])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // 이벤트를 로컬 DB에 캐싱
    pub fn cache_event(
        &self,
        session_id: &str,
        app_name: &str,
        window_title: &str,
        activity_vector_json: &str, // JSON 문자열을 직접 받음
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now_s = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();

        // 스키마에 맞게 INSERT
        conn.execute(
            "INSERT INTO cached_events (session_id, timestamp, app_name, window_title, activity_vector) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                session_id,
                now_s,
                app_name,
                window_title,
                activity_vector_json // JSON 문자열 저장
            ],
        ).map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn cache_feedback(&self, event_id: &str, feedback_type: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now_s = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();

        conn.execute(
            "INSERT INTO cached_feedback (timestamp, event_id, feedback_type) VALUES (?1, ?2, ?3)",
            rusqlite::params![now_s, event_id, feedback_type],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    // 동기화를 위해 전송되지 않은 이벤트 조회
    // limit: 한 번에 가져올 개수 (예: 50개)
    pub fn get_unsynced_events(&self, limit: u32) -> Result<Vec<CachedEvent>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        
        // 오래된 순서(ASC)로 조회하여 순차적 전송 보장
        let mut stmt = conn.prepare(
            "SELECT id, session_id, timestamp, app_name, window_title, activity_vector 
             FROM cached_events 
             ORDER BY timestamp ASC 
             LIMIT ?1"
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map([limit], |row| {
            Ok(CachedEvent {
                id: row.get(0)?,
                session_id: row.get(1)?,
                timestamp: row.get(2)?,
                app_name: row.get(3)?,
                window_title: row.get(4)?,
                activity_vector: row.get(5)?,
            })
        }).map_err(|e| e.to_string())?;

        let mut events = Vec::new();
        for row in rows {
            events.push(row.map_err(|e| e.to_string())?);
        }
        
        Ok(events)
    }

    // 전송 완료된 이벤트 삭제 (Batch Delete)
    // ids: 삭제할 이벤트의 ID 목록
    pub fn delete_events_by_ids(&self, ids: &[i64]) -> Result<(), String> {
        let mut conn = self.conn.lock().map_err(|e| e.to_string())?;
        
        // 트랜잭션 시작 (중간에 실패하면 롤백)
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        
        for id in ids {
            tx.execute("DELETE FROM cached_events WHERE id = ?1", [id])
                .map_err(|e| e.to_string())?;
        }
        
        tx.commit().map_err(|e| e.to_string())?;
        
        Ok(())
    }

    pub fn save_auth_token(&self, access: &str, refresh: &str, email: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        conn.execute(
            "INSERT OR REPLACE INTO auth_token (id, access_token, refresh_token, user_email, updated_at) 
             VALUES (1, ?1, ?2, ?3, ?4)",
            rusqlite::params![access, refresh, email, now],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_auth_token(&self) -> Result<Option<(String, String, String)>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT access_token, refresh_token, user_email FROM auth_token WHERE id = 1")
            .map_err(|e| e.to_string())?;

        let result = stmt
            .query_row([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .optional()
            .map_err(|e| e.to_string())?;

        Ok(result)
    }

    pub fn delete_auth_token(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM auth_token WHERE id = 1", [])
            .map_err(|e| e.to_string())?;
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
        storage
            .save_active_session(&info)
            .expect("Failed to save session");

        // 2. 로드 테스트
        let loaded_info = storage
            .load_active_session()
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
        storage
            .save_active_session(&info)
            .expect("Failed to save session");
        let loaded_info = storage.load_active_session().unwrap();
        assert!(loaded_info.is_some()); // 저장 확인

        // 2. 삭제
        storage
            .delete_active_session()
            .expect("Failed to delete session");

        // 3. 로드 확인 (None이어야 함)
        let loaded_info_after_delete = storage.load_active_session().unwrap();
        assert!(loaded_info_after_delete.is_none());
    }

    #[test]
    fn test_cache_event() {
        let storage = setup_test_db();

        // 테스트용 목업 데이터 (JSON 문자열) 생성
        let mock_stats_1 = InputStats {
            meaningful_input_events: 10,
            last_meaningful_input_timestamp_ms: 1234567890,
            last_mouse_move_timestamp_ms: 1234567899,
            start_monitoring_timestamp_ms: 0,
        };
        // commands.rs의 헬퍼 함수를 직접 테스트
        let json_1 = mock_stats_1.to_activity_vector_json();

        let mock_stats_2 = InputStats {
            meaningful_input_events: 20,
            last_meaningful_input_timestamp_ms: 1234567999,
            last_mouse_move_timestamp_ms: 1234567990,
            start_monitoring_timestamp_ms: 0,
        };
        let json_2 = mock_stats_2.to_activity_vector_json();

        // 변경된 cache_event 시그니처 호출
        storage
            .cache_event("session-1", "chrome.exe", "YouTube", &json_1)
            .expect("Failed to cache event 1");
        storage
            .cache_event("session-1", "code.exe", "lib.rs", &json_2)
            .expect("Failed to cache event 2");

        let conn = storage.conn.lock().unwrap();
        // 스키마(activity_vector)에서 데이터 검증
        let mut stmt = conn
            .prepare(
                "SELECT COUNT(*), activity_vector FROM cached_events WHERE app_name = 'chrome.exe'",
            )
            .unwrap();
        let (count, vector_str): (i64, String) = stmt
            .query_row([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap();

        assert_eq!(count, 1);
        assert!(vector_str.contains("meaningful_input_events\":10")); // JSON 내용 검증
    }
}
