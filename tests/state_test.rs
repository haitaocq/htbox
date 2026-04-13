use htbox::state::{ServiceState, State};

#[test]
fn test_state_default() {
    let state = State::default();
    assert!(state.services.is_empty());
}

#[test]
fn test_state_update_service() {
    let mut state = State::default();

    let service_state = ServiceState {
        name: "test-service".to_string(),
        service_type: "daemon".to_string(),
        running: true,
        pid: Some(12345),
        enabled: true,
        last_start: Some(1234567890),
        last_stop: None,
    };

    state.update_service("test-service", service_state.clone());

    assert_eq!(state.services.len(), 1);
    assert!(state.services.contains_key("test-service"));
}

#[test]
fn test_state_remove_service() {
    let mut state = State::default();

    let service_state = ServiceState {
        name: "test-service".to_string(),
        service_type: "daemon".to_string(),
        running: true,
        pid: Some(12345),
        enabled: true,
        last_start: Some(1234567890),
        last_stop: None,
    };

    state.update_service("test-service", service_state);
    state.remove_service("test-service");

    assert!(state.services.is_empty());
}

#[test]
fn test_state_get_service() {
    let mut state = State::default();

    let service_state = ServiceState {
        name: "test-service".to_string(),
        service_type: "daemon".to_string(),
        running: true,
        pid: Some(12345),
        enabled: true,
        last_start: Some(1234567890),
        last_stop: None,
    };

    state.update_service("test-service", service_state);

    let retrieved = state.services.get("test-service");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "test-service");
}

#[test]
fn test_state_get_nonexistent() {
    let state = State::default();
    let retrieved = state.services.get("nonexistent");
    assert!(retrieved.is_none());
}

#[test]
fn test_service_state_fields() {
    let state = ServiceState {
        name: "my-service".to_string(),
        service_type: "onetime".to_string(),
        running: false,
        pid: None,
        enabled: true,
        last_start: Some(1234567890),
        last_stop: Some(1234567900),
    };

    assert_eq!(state.name, "my-service");
    assert_eq!(state.service_type, "onetime");
    assert!(!state.running);
    assert!(state.pid.is_none());
    assert!(state.enabled);
    assert!(state.last_start.is_some());
    assert!(state.last_stop.is_some());
}

#[test]
fn test_service_state_clone() {
    let state1 = ServiceState {
        name: "test".to_string(),
        service_type: "daemon".to_string(),
        running: true,
        pid: Some(100),
        enabled: false,
        last_start: None,
        last_stop: None,
    };

    let state2 = state1.clone();

    assert_eq!(state1.name, state2.name);
    assert_eq!(state1.service_type, state2.service_type);
}
