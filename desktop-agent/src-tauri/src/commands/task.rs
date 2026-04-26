// 파일 위치: src-tauri/src/commands/task.rs
// backend_comm.rs에서 분리된 Task 관련 Tauri 커맨드 (U-3 해결)

use tauri::{command, State};
use crate::{StorageManagerArcMutex, Task};

/// Task / LSN 데이터 연동
#[command]
pub fn get_tasks(
    storage_manager_mutex: State<'_, StorageManagerArcMutex>,
) -> Result<Vec<Task>, String> {
    let storage_manager = storage_manager_mutex.lock().map_err(|e| e.to_string())?;

    let user_id = match storage_manager
        .load_auth_token()
        .map_err(|e| e.to_string())?
    {
        Some((_, _, _, uid)) => uid,
        None => return Ok(vec![]),
    };

    let local_tasks = storage_manager
        .get_tasks_by_user(&user_id)
        .map_err(|e| e.to_string())?;

    println!(
        "get_tasks: Found {} tasks for user {}",
        local_tasks.len(),
        user_id
    );

    let tasks: Vec<Task> = local_tasks
        .into_iter()
        .map(|t| Task {
            id: t.id,
            user_id: t.user_id,
            task_name: t.task_name,
            description: t.description.unwrap_or_default(),
            due_date: "".to_string(),
            status: t.status,
            target_executable: t.target_executable.unwrap_or_default(),
            target_arguments: t
                .target_arguments
                .map(|s| s.split_whitespace().map(|s| s.to_string()).collect())
                .unwrap_or_default(),
            created_at: "".to_string(),
            updated_at: "".to_string(),
        })
        .collect();

    Ok(tasks)
}
