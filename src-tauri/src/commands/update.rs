use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub available: bool,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub release_notes: Option<String>,
}

/// 업데이트 확인
#[tauri::command]
pub async fn check_for_updates(app: tauri::AppHandle) -> Result<UpdateInfo, String> {
    let current_version = app.package_info().version.to_string();

    // Tauri updater를 사용한 업데이트 체크
    match app.updater().check().await {
        Ok(update) => {
            if update.is_update_available() {
                Ok(UpdateInfo {
                    available: true,
                    current_version,
                    latest_version: Some(update.latest_version().to_string()),
                    release_notes: update.body().map(|b| b.to_string()),
                })
            } else {
                Ok(UpdateInfo {
                    available: false,
                    current_version,
                    latest_version: None,
                    release_notes: None,
                })
            }
        }
        Err(e) => Err(format!("업데이트 확인 실패: {}", e)),
    }
}

/// 업데이트 다운로드 및 설치
#[tauri::command]
pub async fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    match app.updater().check().await {
        Ok(update) => {
            if update.is_update_available() {
                // 업데이트 다운로드 및 설치
                update
                    .download_and_install()
                    .await
                    .map_err(|e| format!("업데이트 설치 실패: {}", e))?;

                Ok(())
            } else {
                Err("설치할 업데이트가 없습니다.".to_string())
            }
        }
        Err(e) => Err(format!("업데이트 확인 실패: {}", e)),
    }
}

/// 현재 버전 정보 가져오기
#[tauri::command]
pub fn get_app_version(app: tauri::AppHandle) -> String {
    app.package_info().version.to_string()
}
