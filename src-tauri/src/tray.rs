use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem, WindowEvent,
};

/// System Tray 메뉴 생성
pub fn create_tray() -> SystemTray {
    let open = CustomMenuItem::new("open".to_string(), "열기");
    let settings = CustomMenuItem::new("settings".to_string(), "설정");
    let separator = SystemTrayMenuItem::Separator;
    let quit = CustomMenuItem::new("quit".to_string(), "종료");

    let tray_menu = SystemTrayMenu::new()
        .add_item(open)
        .add_item(settings)
        .add_native_item(separator)
        .add_item(quit);

    SystemTray::new().with_menu(tray_menu)
}

/// System Tray 이벤트 핸들러
pub fn handle_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::LeftClick {
            position: _,
            size: _,
            ..
        } => {
            // 트레이 아이콘 왼쪽 클릭 → 메인 창 표시
            if let Some(window) = app.get_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "open" => {
                // "열기" 메뉴 클릭 → 메인 창 표시
                if let Some(window) = app.get_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "settings" => {
                // "설정" 메뉴 클릭 → 설정 페이지로 이동
                if let Some(window) = app.get_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    // Frontend에서 /settings 경로로 이동하도록 이벤트 전송
                    let _ = window.emit("navigate-to-settings", ());
                }
            }
            "quit" => {
                // "종료" 메뉴 클릭 → 앱 종료
                std::process::exit(0);
            }
            _ => {}
        },
        _ => {}
    }
}

/// 창 닫기 이벤트 핸들러 - 트레이로 최소화 (백그라운드 실행)
pub fn handle_window_close(window: &tauri::Window, event: &tauri::WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        // 창 닫기 요청 → 숨기기만 하고 종료 방지
        api.prevent_close();

        // 창을 숨김 (트레이로 최소화)
        let _ = window.hide();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_tray() {
        // System Tray 생성 테스트
        let tray = create_tray();
        // Tauri SystemTray는 실제 앱 컨텍스트 없이 테스트 불가
        // 단순 생성 성공 여부만 확인
        assert!(std::mem::size_of_val(&tray) > 0);
    }
}
