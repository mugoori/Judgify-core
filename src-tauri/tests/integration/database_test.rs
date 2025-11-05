use judgify_core::database::{Database, ChatMessage, ChatSession, User};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_database_connection_integration() {
    // 데이터베이스 연결 테스트
    let db = Database::new("sqlite://judgify_test.db");
    let result = db.connect();

    // 연결 성공 확인
    assert!(result.is_ok());
    assert!(db.is_connected());
}

#[test]
fn test_database_migration_integration() {
    let db = Database::new("sqlite://judgify_test_migration.db");
    db.connect().ok();

    // 마이그레이션 실행
    let result = db.run_migrations();

    // 마이그레이션 성공 확인
    assert!(result.is_ok());

    // 테이블 존재 확인
    assert!(db.table_exists("chat_sessions"));
    assert!(db.table_exists("chat_messages"));
    assert!(db.table_exists("users"));
}

#[test]
fn test_database_save_message_integration() {
    let db = Database::new("sqlite://judgify_test_save.db");
    db.connect().ok();
    db.run_migrations().ok();

    // 메시지 저장
    let message = ChatMessage {
        id: uuid::Uuid::new_v4(),
        session_id: "test-session-1".to_string(),
        role: "user".to_string(),
        content: "Test message".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = db.save_message(&message);

    // 저장 성공 확인
    assert!(result.is_ok());

    // 저장된 메시지 조회
    let retrieved = db.get_message(message.id);
    assert!(retrieved.is_ok());
    let retrieved_msg = retrieved.unwrap();
    assert_eq!(retrieved_msg.content, "Test message");
}

#[test]
fn test_database_query_messages_integration() {
    let db = Database::new("sqlite://judgify_test_query.db");
    db.connect().ok();
    db.run_migrations().ok();

    let session_id = "test-session-2";

    // 여러 메시지 저장
    for i in 1..=5 {
        let message = ChatMessage {
            id: uuid::Uuid::new_v4(),
            session_id: session_id.to_string(),
            role: if i % 2 == 0 { "user" } else { "assistant" }.to_string(),
            content: format!("Message {}", i),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        db.save_message(&message).ok();
    }

    // 메시지 조회
    let messages = db.get_messages_by_session(session_id);

    // 검증
    assert!(messages.is_ok());
    let messages_vec = messages.unwrap();
    assert_eq!(messages_vec.len(), 5);
    assert!(messages_vec.iter().any(|msg| msg.content.contains("Message 1")));
}

#[test]
fn test_database_delete_message_integration() {
    let db = Database::new("sqlite://judgify_test_delete.db");
    db.connect().ok();
    db.run_migrations().ok();

    // 메시지 저장
    let message = ChatMessage {
        id: uuid::Uuid::new_v4(),
        session_id: "test-session-3".to_string(),
        role: "user".to_string(),
        content: "To be deleted".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    db.save_message(&message).ok();

    // 메시지 삭제
    let delete_result = db.delete_message(message.id);
    assert!(delete_result.is_ok());

    // 삭제 확인
    let retrieved = db.get_message(message.id);
    assert!(retrieved.is_err() || retrieved.unwrap().id != message.id);
}

#[test]
fn test_database_session_management_integration() {
    let db = Database::new("sqlite://judgify_test_session.db");
    db.connect().ok();
    db.run_migrations().ok();

    // 세션 생성
    let session = ChatSession {
        id: uuid::Uuid::new_v4(),
        session_id: "test-session-4".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        last_active: chrono::Utc::now().to_rfc3339(),
    };

    let result = db.create_session(&session);
    assert!(result.is_ok());

    // 세션 조회
    let retrieved = db.get_session(&session.session_id);
    assert!(retrieved.is_ok());
    assert_eq!(retrieved.unwrap().session_id, "test-session-4");
}

#[test]
fn test_database_transaction_rollback_integration() {
    let db = Database::new("sqlite://judgify_test_transaction.db");
    db.connect().ok();
    db.run_migrations().ok();

    // 트랜잭션 시작
    db.begin_transaction().ok();

    // 메시지 저장
    let message = ChatMessage {
        id: uuid::Uuid::new_v4(),
        session_id: "test-session-5".to_string(),
        role: "user".to_string(),
        content: "Rollback test".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    db.save_message(&message).ok();

    // 롤백
    db.rollback_transaction().ok();

    // 메시지가 저장되지 않았는지 확인
    let retrieved = db.get_message(message.id);
    assert!(retrieved.is_err());
}

#[test]
fn test_database_transaction_commit_integration() {
    let db = Database::new("sqlite://judgify_test_commit.db");
    db.connect().ok();
    db.run_migrations().ok();

    // 트랜잭션 시작
    db.begin_transaction().ok();

    // 메시지 저장
    let message = ChatMessage {
        id: uuid::Uuid::new_v4(),
        session_id: "test-session-6".to_string(),
        role: "user".to_string(),
        content: "Commit test".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    db.save_message(&message).ok();

    // 커밋
    db.commit_transaction().ok();

    // 메시지가 저장되었는지 확인
    let retrieved = db.get_message(message.id);
    assert!(retrieved.is_ok());
    assert_eq!(retrieved.unwrap().content, "Commit test");
}

#[test]
fn test_database_concurrent_writes_integration() {
    let db = Arc::new(Database::new("sqlite://judgify_test_concurrent.db"));
    db.connect().ok();
    db.run_migrations().ok();

    let mut handles = vec![];

    // 10개 스레드에서 동시 쓰기
    for i in 0..10 {
        let db_clone = Arc::clone(&db);
        let handle = thread::spawn(move || {
            let message = ChatMessage {
                id: uuid::Uuid::new_v4(),
                session_id: format!("concurrent-session-{}", i),
                role: "user".to_string(),
                content: format!("Concurrent message {}", i),
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            db_clone.save_message(&message)
        });
        handles.push(handle);
    }

    // 모든 스레드 완료 대기
    let mut success_count = 0;
    for handle in handles {
        if handle.join().unwrap().is_ok() {
            success_count += 1;
        }
    }

    // 모든 쓰기가 성공해야 함
    assert_eq!(success_count, 10);
}

#[test]
fn test_database_bulk_insert_integration() {
    let db = Database::new("sqlite://judgify_test_bulk.db");
    db.connect().ok();
    db.run_migrations().ok();

    let session_id = "test-session-7";

    // 100개 메시지 일괄 삽입
    let mut messages = Vec::new();
    for i in 1..=100 {
        messages.push(ChatMessage {
            id: uuid::Uuid::new_v4(),
            session_id: session_id.to_string(),
            role: if i % 2 == 0 { "user" } else { "assistant" }.to_string(),
            content: format!("Bulk message {}", i),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
    }

    let result = db.bulk_insert_messages(&messages);
    assert!(result.is_ok());

    // 삽입된 메시지 개수 확인
    let retrieved = db.get_messages_by_session(session_id).unwrap();
    assert_eq!(retrieved.len(), 100);
}

#[test]
fn test_database_search_messages_integration() {
    let db = Database::new("sqlite://judgify_test_search.db");
    db.connect().ok();
    db.run_migrations().ok();

    // 메시지 저장
    let messages = vec![
        ChatMessage {
            id: uuid::Uuid::new_v4(),
            session_id: "test-session-8".to_string(),
            role: "user".to_string(),
            content: "Hello world".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
        ChatMessage {
            id: uuid::Uuid::new_v4(),
            session_id: "test-session-8".to_string(),
            role: "assistant".to_string(),
            content: "Goodbye world".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
        ChatMessage {
            id: uuid::Uuid::new_v4(),
            session_id: "test-session-8".to_string(),
            role: "user".to_string(),
            content: "Test message".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    ];

    for msg in messages {
        db.save_message(&msg).ok();
    }

    // "world" 검색
    let search_results = db.search_messages("world");
    assert!(search_results.is_ok());
    let results = search_results.unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|msg| msg.content.contains("Hello")));
    assert!(results.iter().any(|msg| msg.content.contains("Goodbye")));
}

#[test]
fn test_database_pagination_integration() {
    let db = Database::new("sqlite://judgify_test_pagination.db");
    db.connect().ok();
    db.run_migrations().ok();

    let session_id = "test-session-9";

    // 50개 메시지 저장
    for i in 1..=50 {
        let message = ChatMessage {
            id: uuid::Uuid::new_v4(),
            session_id: session_id.to_string(),
            role: "user".to_string(),
            content: format!("Message {}", i),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        db.save_message(&message).ok();
    }

    // 페이지네이션 조회 (페이지 크기: 10)
    let page1 = db.get_messages_paginated(session_id, 1, 10);
    let page2 = db.get_messages_paginated(session_id, 2, 10);

    assert!(page1.is_ok());
    assert!(page2.is_ok());

    let page1_msgs = page1.unwrap();
    let page2_msgs = page2.unwrap();

    assert_eq!(page1_msgs.len(), 10);
    assert_eq!(page2_msgs.len(), 10);

    // 페이지 간 메시지가 중복되지 않는지 확인
    let page1_contents: Vec<_> = page1_msgs.iter().map(|m| &m.content).collect();
    let page2_contents: Vec<_> = page2_msgs.iter().map(|m| &m.content).collect();
    assert!(page1_contents.iter().all(|c| !page2_contents.contains(c)));
}

#[test]
fn test_database_vacuum_integration() {
    let db = Database::new("sqlite://judgify_test_vacuum.db");
    db.connect().ok();
    db.run_migrations().ok();

    // 메시지 저장 후 삭제 (공간 낭비)
    for i in 1..=100 {
        let message = ChatMessage {
            id: uuid::Uuid::new_v4(),
            session_id: "test-session-10".to_string(),
            role: "user".to_string(),
            content: format!("Temp message {}", i),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        let saved = db.save_message(&message).ok();
        if saved.is_some() {
            db.delete_message(message.id).ok();
        }
    }

    // VACUUM 실행
    let result = db.vacuum();
    assert!(result.is_ok());
}

#[test]
fn test_database_backup_restore_integration() {
    let db_original = Database::new("sqlite://judgify_test_backup.db");
    db_original.connect().ok();
    db_original.run_migrations().ok();

    // 원본 데이터 저장
    let message = ChatMessage {
        id: uuid::Uuid::new_v4(),
        session_id: "test-session-11".to_string(),
        role: "user".to_string(),
        content: "Backup test".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    db_original.save_message(&message).ok();

    // 백업
    let backup_result = db_original.backup("judgify_test_backup_copy.db");
    assert!(backup_result.is_ok());

    // 백업 파일에서 복원
    let db_restored = Database::new("sqlite://judgify_test_backup_copy.db");
    db_restored.connect().ok();

    // 복원된 데이터 확인
    let retrieved = db_restored.get_message(message.id);
    assert!(retrieved.is_ok());
    assert_eq!(retrieved.unwrap().content, "Backup test");
}
