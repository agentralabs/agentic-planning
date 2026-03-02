use agentic_planning::{
    CreateCommitmentRequest, CreateDecisionRequest, CreateGoalRequest, DecisionReasoning,
    GoalFilter, PlanningEngine, Promise, Stakeholder, StakeholderId,
};
use std::cell::RefCell;
use std::ffi::{c_char, CStr, CString};
use std::path::PathBuf;

// ── R1: Error code enum ──────────────────────────────────────────────

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AplanResult {
    Ok = 0,
    NullPointer = 1,
    InvalidUtf8 = 2,
    EngineError = 3,
    NotFound = 4,
    ValidationError = 5,
    IoError = 6,
    SerializationError = 7,
}

// ── R8: Thread-local last-error storage ──────────────────────────────

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

fn set_last_error(msg: &str) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = CString::new(msg).ok();
    });
}

fn map_engine_error(err: &agentic_planning::Error) -> AplanResult {
    match err {
        agentic_planning::Error::GoalNotFound(_)
        | agentic_planning::Error::DecisionNotFound(_)
        | agentic_planning::Error::CommitmentNotFound(_)
        | agentic_planning::Error::FederationNotFound(_)
        | agentic_planning::Error::SoulNotFound(_)
        | agentic_planning::Error::PathNotFound(_)
        | agentic_planning::Error::DreamNotFound(_) => AplanResult::NotFound,
        agentic_planning::Error::InvalidTransition { .. }
        | agentic_planning::Error::CannotComplete(_)
        | agentic_planning::Error::AlreadyCrystallized
        | agentic_planning::Error::CannotRecrystallize(_)
        | agentic_planning::Error::CannotFulfill(_)
        | agentic_planning::Error::CannotBreak(_)
        | agentic_planning::Error::Validation(_) => AplanResult::ValidationError,
        agentic_planning::Error::InvalidFile | agentic_planning::Error::CorruptedFile(_) => {
            AplanResult::IoError
        }
        agentic_planning::Error::Io(_) => AplanResult::IoError,
        agentic_planning::Error::Serde(_) => AplanResult::SerializationError,
    }
}

fn handle_engine_error(err: agentic_planning::Error) -> AplanResult {
    let code = map_engine_error(&err);
    set_last_error(&err.to_string());
    code
}

// ── Handle type ──────────────────────────────────────────────────────

#[repr(C)]
pub struct AplanHandle {
    engine: PlanningEngine,
}

// ── Helper macros ────────────────────────────────────────────────────

macro_rules! check_null {
    ($ptr:expr) => {
        if $ptr.is_null() {
            set_last_error("null pointer argument");
            return std::ptr::null_mut();
        }
    };
    ($ptr:expr, result) => {
        if $ptr.is_null() {
            set_last_error("null pointer argument");
            return AplanResult::NullPointer;
        }
    };
}

macro_rules! cstr_to_string {
    ($ptr:expr) => {{
        let s = CStr::from_ptr($ptr).to_str();
        match s {
            Ok(v) => v.to_string(),
            Err(_) => {
                set_last_error("invalid UTF-8 string");
                return std::ptr::null_mut();
            }
        }
    }};
    ($ptr:expr, result) => {{
        let s = CStr::from_ptr($ptr).to_str();
        match s {
            Ok(v) => v.to_string(),
            Err(_) => {
                set_last_error("invalid UTF-8 string");
                return AplanResult::InvalidUtf8;
            }
        }
    }};
}

fn to_json_cstring<T: serde::Serialize>(value: &T) -> *mut c_char {
    match serde_json::to_string(value) {
        Ok(s) => CString::new(s).map(CString::into_raw).unwrap_or_else(|_| {
            set_last_error("serialization produced invalid C string");
            std::ptr::null_mut()
        }),
        Err(e) => {
            set_last_error(&format!("serialization error: {e}"));
            std::ptr::null_mut()
        }
    }
}

fn parse_uuid(s: &str) -> Option<uuid::Uuid> {
    uuid::Uuid::parse_str(s).ok()
}

// ── R2: Engine lifecycle ─────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn aplan_engine_new() -> *mut AplanHandle {
    let handle = AplanHandle {
        engine: PlanningEngine::in_memory(),
    };
    Box::into_raw(Box::new(handle))
}

#[no_mangle]
pub extern "C" fn aplan_engine_new_memory() -> *mut AplanHandle {
    aplan_engine_new()
}

/// # Safety
/// `path` must be a valid null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn aplan_engine_new_file(path: *const c_char) -> *mut AplanHandle {
    check_null!(path);
    let path_str = cstr_to_string!(path);
    match PlanningEngine::load(PathBuf::from(&path_str)) {
        Ok(engine) => Box::into_raw(Box::new(AplanHandle { engine })),
        Err(e) => {
            set_last_error(&e.to_string());
            std::ptr::null_mut()
        }
    }
}

/// # Safety
/// Caller must pass a valid pointer previously returned by `aplan_engine_new`.
#[no_mangle]
pub unsafe extern "C" fn aplan_engine_free(handle: *mut AplanHandle) {
    if handle.is_null() {
        return;
    }
    drop(Box::from_raw(handle));
}

/// # Safety
/// `handle` must be a valid engine pointer.
#[no_mangle]
pub unsafe extern "C" fn aplan_engine_save(handle: *mut AplanHandle) -> AplanResult {
    check_null!(handle, result);
    let h = &mut *handle;
    match h.engine.save() {
        Ok(()) => AplanResult::Ok,
        Err(e) => handle_engine_error(e),
    }
}

/// # Safety
/// `handle` and `path` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn aplan_engine_load(
    handle: *mut AplanHandle,
    path: *const c_char,
) -> AplanResult {
    check_null!(handle, result);
    check_null!(path, result);
    let path_str = cstr_to_string!(path, result);
    match PlanningEngine::load(PathBuf::from(&path_str)) {
        Ok(engine) => {
            let h = &mut *handle;
            h.engine = engine;
            AplanResult::Ok
        }
        Err(e) => handle_engine_error(e),
    }
}

// ── R3: Goal operations ──────────────────────────────────────────────

/// # Safety
/// `handle`, `title`, and `intention` must be valid non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn aplan_goal_create(
    handle: *mut AplanHandle,
    title: *const c_char,
    intention: *const c_char,
) -> *mut c_char {
    check_null!(handle);
    check_null!(title);
    check_null!(intention);

    let title = cstr_to_string!(title);
    let intention = cstr_to_string!(intention);

    let h = &mut *handle;
    match h.engine.create_goal(CreateGoalRequest {
        title,
        intention,
        ..Default::default()
    }) {
        Ok(goal) => to_json_cstring(&goal),
        Err(e) => {
            set_last_error(&e.to_string());
            std::ptr::null_mut()
        }
    }
}

/// # Safety
/// `handle` and `id` must be valid non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn aplan_goal_get(
    handle: *mut AplanHandle,
    id: *const c_char,
) -> *mut c_char {
    check_null!(handle);
    check_null!(id);

    let id_str = cstr_to_string!(id);
    let uuid = match parse_uuid(&id_str) {
        Some(u) => u,
        None => {
            set_last_error("invalid UUID");
            return std::ptr::null_mut();
        }
    };

    let h = &*handle;
    match h.engine.get_goal(agentic_planning::GoalId(uuid)) {
        Some(goal) => to_json_cstring(goal),
        None => {
            set_last_error("goal not found");
            std::ptr::null_mut()
        }
    }
}

/// # Safety
/// `handle` must be a valid non-null pointer.
#[no_mangle]
pub unsafe extern "C" fn aplan_goal_list(handle: *mut AplanHandle) -> *mut c_char {
    check_null!(handle);
    let h = &*handle;
    let goals = h.engine.list_goals(GoalFilter::default());
    to_json_cstring(&goals)
}

/// # Safety
/// `handle` and `id` must be valid non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn aplan_goal_pause(
    handle: *mut AplanHandle,
    id: *const c_char,
    reason: *const c_char,
) -> AplanResult {
    check_null!(handle, result);
    check_null!(id, result);

    let id_str = cstr_to_string!(id, result);
    let uuid = match parse_uuid(&id_str) {
        Some(u) => u,
        None => {
            set_last_error("invalid UUID");
            return AplanResult::InvalidUtf8;
        }
    };

    let reason_opt = if reason.is_null() {
        None
    } else {
        Some(cstr_to_string!(reason, result))
    };

    let h = &mut *handle;
    match h
        .engine
        .pause_goal(agentic_planning::GoalId(uuid), reason_opt)
    {
        Ok(_) => AplanResult::Ok,
        Err(e) => handle_engine_error(e),
    }
}

/// # Safety
/// `handle` and `id` must be valid non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn aplan_goal_resume(
    handle: *mut AplanHandle,
    id: *const c_char,
) -> AplanResult {
    check_null!(handle, result);
    check_null!(id, result);

    let id_str = cstr_to_string!(id, result);
    let uuid = match parse_uuid(&id_str) {
        Some(u) => u,
        None => {
            set_last_error("invalid UUID");
            return AplanResult::InvalidUtf8;
        }
    };

    let h = &mut *handle;
    match h.engine.resume_goal(agentic_planning::GoalId(uuid)) {
        Ok(_) => AplanResult::Ok,
        Err(e) => handle_engine_error(e),
    }
}

/// # Safety
/// `handle` and `id` must be valid non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn aplan_goal_abandon(
    handle: *mut AplanHandle,
    id: *const c_char,
) -> AplanResult {
    check_null!(handle, result);
    check_null!(id, result);

    let id_str = cstr_to_string!(id, result);
    let uuid = match parse_uuid(&id_str) {
        Some(u) => u,
        None => {
            set_last_error("invalid UUID");
            return AplanResult::InvalidUtf8;
        }
    };

    let h = &mut *handle;
    match h.engine.abandon_goal(
        agentic_planning::GoalId(uuid),
        "abandoned via FFI".to_string(),
    ) {
        Ok(_) => AplanResult::Ok,
        Err(e) => handle_engine_error(e),
    }
}

/// # Safety
/// `handle` and `id` must be valid non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn aplan_goal_complete(
    handle: *mut AplanHandle,
    id: *const c_char,
) -> AplanResult {
    check_null!(handle, result);
    check_null!(id, result);

    let id_str = cstr_to_string!(id, result);
    let uuid = match parse_uuid(&id_str) {
        Some(u) => u,
        None => {
            set_last_error("invalid UUID");
            return AplanResult::InvalidUtf8;
        }
    };

    let h = &mut *handle;
    match h.engine.complete_goal(agentic_planning::GoalId(uuid), None) {
        Ok(_) => AplanResult::Ok,
        Err(e) => handle_engine_error(e),
    }
}

// ── R4: Decision operations ──────────────────────────────────────────

/// # Safety
/// `handle` and `json` must be valid non-null pointers.
/// `json` should contain at minimum `{"question":"..."}`.
#[no_mangle]
pub unsafe extern "C" fn aplan_decision_create(
    handle: *mut AplanHandle,
    json: *const c_char,
) -> *mut c_char {
    check_null!(handle);
    check_null!(json);

    let json_str = cstr_to_string!(json);
    let v: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(e) => {
            set_last_error(&format!("invalid JSON: {e}"));
            return std::ptr::null_mut();
        }
    };

    let question = v["question"].as_str().unwrap_or("").to_string();
    let context = v["context"].as_str().map(String::from);
    let request = CreateDecisionRequest {
        question,
        context,
        ..Default::default()
    };

    let h = &mut *handle;
    match h.engine.create_decision(request) {
        Ok(decision) => to_json_cstring(&decision),
        Err(e) => {
            set_last_error(&e.to_string());
            std::ptr::null_mut()
        }
    }
}

/// # Safety
/// `handle` and `id` must be valid non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn aplan_decision_get(
    handle: *mut AplanHandle,
    id: *const c_char,
) -> *mut c_char {
    check_null!(handle);
    check_null!(id);

    let id_str = cstr_to_string!(id);
    let uuid = match parse_uuid(&id_str) {
        Some(u) => u,
        None => {
            set_last_error("invalid UUID");
            return std::ptr::null_mut();
        }
    };

    let h = &*handle;
    match h.engine.get_decision(agentic_planning::DecisionId(uuid)) {
        Some(d) => to_json_cstring(d),
        None => {
            set_last_error("decision not found");
            std::ptr::null_mut()
        }
    }
}

/// # Safety
/// `handle` must be a valid non-null pointer.
#[no_mangle]
pub unsafe extern "C" fn aplan_decision_list(handle: *mut AplanHandle) -> *mut c_char {
    check_null!(handle);
    let h = &*handle;
    let decisions = h.engine.list_decisions();
    to_json_cstring(&decisions)
}

/// # Safety
/// `handle`, `id`, and `chosen_path_id` must be valid non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn aplan_decision_crystallize(
    handle: *mut AplanHandle,
    id: *const c_char,
    chosen_path_id: *const c_char,
) -> AplanResult {
    check_null!(handle, result);
    check_null!(id, result);
    check_null!(chosen_path_id, result);

    let id_str = cstr_to_string!(id, result);
    let path_str = cstr_to_string!(chosen_path_id, result);

    let dec_uuid = match parse_uuid(&id_str) {
        Some(u) => u,
        None => {
            set_last_error("invalid decision UUID");
            return AplanResult::InvalidUtf8;
        }
    };
    let path_uuid = match parse_uuid(&path_str) {
        Some(u) => u,
        None => {
            set_last_error("invalid path UUID");
            return AplanResult::InvalidUtf8;
        }
    };

    let h = &mut *handle;
    let reasoning = DecisionReasoning {
        rationale: "crystallized via FFI".to_string(),
        confidence: 0.8,
        factors_considered: vec![],
        weights: std::collections::HashMap::new(),
    };

    match h.engine.crystallize(
        agentic_planning::DecisionId(dec_uuid),
        agentic_planning::PathId(path_uuid),
        reasoning,
    ) {
        Ok(_) => AplanResult::Ok,
        Err(e) => handle_engine_error(e),
    }
}

// ── R5: Commitment operations ────────────────────────────────────────

/// # Safety
/// `handle` and `json` must be valid non-null pointers.
/// `json` must be a valid JSON string for CreateCommitmentRequest.
#[no_mangle]
pub unsafe extern "C" fn aplan_commitment_create(
    handle: *mut AplanHandle,
    json: *const c_char,
) -> *mut c_char {
    check_null!(handle);
    check_null!(json);

    let json_str = cstr_to_string!(json);
    let v: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(e) => {
            set_last_error(&format!("invalid JSON: {e}"));
            return std::ptr::null_mut();
        }
    };

    let promise = Promise {
        description: v["promise"]["description"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        deliverables: v["promise"]["deliverables"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        conditions: v["promise"]["conditions"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
    };
    let stakeholder = Stakeholder {
        id: StakeholderId(uuid::Uuid::new_v4()),
        name: v["stakeholder"]["name"].as_str().unwrap_or("").to_string(),
        role: v["stakeholder"]["role"].as_str().unwrap_or("").to_string(),
        importance: v["stakeholder"]["importance"].as_f64().unwrap_or(0.5),
    };
    let request = CreateCommitmentRequest {
        promise,
        stakeholder,
        due: None,
        goal: None,
    };

    let h = &mut *handle;
    match h.engine.create_commitment(request) {
        Ok(commitment) => to_json_cstring(&commitment),
        Err(e) => {
            set_last_error(&e.to_string());
            std::ptr::null_mut()
        }
    }
}

/// # Safety
/// `handle` and `id` must be valid non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn aplan_commitment_get(
    handle: *mut AplanHandle,
    id: *const c_char,
) -> *mut c_char {
    check_null!(handle);
    check_null!(id);

    let id_str = cstr_to_string!(id);
    let uuid = match parse_uuid(&id_str) {
        Some(u) => u,
        None => {
            set_last_error("invalid UUID");
            return std::ptr::null_mut();
        }
    };

    let h = &*handle;
    match h
        .engine
        .get_commitment(agentic_planning::CommitmentId(uuid))
    {
        Some(c) => to_json_cstring(c),
        None => {
            set_last_error("commitment not found");
            std::ptr::null_mut()
        }
    }
}

/// # Safety
/// `handle` must be a valid non-null pointer.
#[no_mangle]
pub unsafe extern "C" fn aplan_commitment_list(handle: *mut AplanHandle) -> *mut c_char {
    check_null!(handle);
    let h = &*handle;
    let commitments = h.engine.list_commitments();
    to_json_cstring(&commitments)
}

/// # Safety
/// `handle` and `id` must be valid non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn aplan_commitment_fulfill(
    handle: *mut AplanHandle,
    id: *const c_char,
) -> AplanResult {
    check_null!(handle, result);
    check_null!(id, result);

    let id_str = cstr_to_string!(id, result);
    let uuid = match parse_uuid(&id_str) {
        Some(u) => u,
        None => {
            set_last_error("invalid UUID");
            return AplanResult::InvalidUtf8;
        }
    };

    let h = &mut *handle;
    match h.engine.fulfill_commitment(
        agentic_planning::CommitmentId(uuid),
        "fulfilled via FFI".to_string(),
    ) {
        Ok(_) => AplanResult::Ok,
        Err(e) => handle_engine_error(e),
    }
}

/// # Safety
/// `handle`, `id`, and `reason` must be valid non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn aplan_commitment_break(
    handle: *mut AplanHandle,
    id: *const c_char,
    reason: *const c_char,
) -> AplanResult {
    check_null!(handle, result);
    check_null!(id, result);
    check_null!(reason, result);

    let id_str = cstr_to_string!(id, result);
    let reason_str = cstr_to_string!(reason, result);

    let uuid = match parse_uuid(&id_str) {
        Some(u) => u,
        None => {
            set_last_error("invalid UUID");
            return AplanResult::InvalidUtf8;
        }
    };

    let h = &mut *handle;
    match h
        .engine
        .break_commitment(agentic_planning::CommitmentId(uuid), reason_str)
    {
        Ok(_) => AplanResult::Ok,
        Err(e) => handle_engine_error(e),
    }
}

// ── R6: Dream operations ─────────────────────────────────────────────

/// # Safety
/// `handle` and `goal_id` must be valid non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn aplan_dream_create(
    handle: *mut AplanHandle,
    goal_id: *const c_char,
) -> *mut c_char {
    check_null!(handle);
    check_null!(goal_id);

    let id_str = cstr_to_string!(goal_id);
    let uuid = match parse_uuid(&id_str) {
        Some(u) => u,
        None => {
            set_last_error("invalid UUID");
            return std::ptr::null_mut();
        }
    };

    let h = &mut *handle;
    match h.engine.dream_goal(agentic_planning::GoalId(uuid)) {
        Ok(dream) => to_json_cstring(&dream),
        Err(e) => {
            set_last_error(&e.to_string());
            std::ptr::null_mut()
        }
    }
}

/// # Safety
/// `handle` and `id` must be valid non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn aplan_dream_get(
    handle: *mut AplanHandle,
    id: *const c_char,
) -> *mut c_char {
    check_null!(handle);
    check_null!(id);

    let id_str = cstr_to_string!(id);
    let uuid = match parse_uuid(&id_str) {
        Some(u) => u,
        None => {
            set_last_error("invalid UUID");
            return std::ptr::null_mut();
        }
    };

    let h = &*handle;
    match h.engine.get_dream(agentic_planning::DreamId(uuid)) {
        Some(d) => to_json_cstring(d),
        None => {
            set_last_error("dream not found");
            std::ptr::null_mut()
        }
    }
}

/// # Safety
/// `handle` must be a valid non-null pointer.
#[no_mangle]
pub unsafe extern "C" fn aplan_dream_list(handle: *mut AplanHandle) -> *mut c_char {
    check_null!(handle);
    let h = &*handle;
    let dreams = h.engine.list_dreams();
    to_json_cstring(&dreams)
}

// ── R7: Query operations ─────────────────────────────────────────────

/// # Safety
/// `handle` must be a valid non-null pointer.
#[no_mangle]
pub unsafe extern "C" fn aplan_singularity_get(handle: *mut AplanHandle) -> *mut c_char {
    check_null!(handle);
    let h = &*handle;
    let singularity = h.engine.get_intention_singularity();
    to_json_cstring(&singularity)
}

/// # Safety
/// `handle` must be a valid non-null pointer.
#[no_mangle]
pub unsafe extern "C" fn aplan_blockers_scan(handle: *mut AplanHandle) -> *mut c_char {
    check_null!(handle);
    let h = &*handle;
    let blockers = h.engine.scan_blocker_prophecy();
    to_json_cstring(&blockers)
}

/// # Safety
/// `handle` must be a valid non-null pointer.
#[no_mangle]
pub unsafe extern "C" fn aplan_echoes_listen(handle: *mut AplanHandle) -> *mut c_char {
    check_null!(handle);
    let h = &*handle;
    let echoes = h.engine.listen_progress_echoes();
    to_json_cstring(&echoes)
}

// ── R8: Utility ──────────────────────────────────────────────────────

/// # Safety
/// Caller must pass a pointer returned by any `aplan_*` function that returns `*mut c_char`.
#[no_mangle]
pub unsafe extern "C" fn aplan_string_free(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    drop(CString::from_raw(s));
}

/// Returns the library version as a static string.
#[no_mangle]
pub extern "C" fn aplan_version() -> *const c_char {
    static VERSION: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();
    VERSION.as_ptr() as *const c_char
}

/// Returns the last error message from the current thread, or null if no error.
/// The returned pointer is valid until the next FFI call on the same thread.
#[no_mangle]
pub extern "C" fn aplan_last_error() -> *const c_char {
    LAST_ERROR.with(|e| {
        e.borrow()
            .as_ref()
            .map(|s| s.as_ptr())
            .unwrap_or(std::ptr::null())
    })
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    fn new_engine() -> *mut AplanHandle {
        aplan_engine_new()
    }

    fn c(s: &str) -> CString {
        CString::new(s).unwrap()
    }

    #[test]
    fn test_engine_lifecycle() {
        let h = new_engine();
        assert!(!h.is_null());
        let h2 = aplan_engine_new_memory();
        assert!(!h2.is_null());
        unsafe {
            aplan_engine_free(h);
            aplan_engine_free(h2);
        }
    }

    #[test]
    fn test_null_safety() {
        unsafe {
            aplan_engine_free(std::ptr::null_mut());
            assert_eq!(
                aplan_engine_save(std::ptr::null_mut()),
                AplanResult::NullPointer
            );
            assert!(
                aplan_goal_create(std::ptr::null_mut(), std::ptr::null(), std::ptr::null())
                    .is_null()
            );
            assert!(aplan_goal_get(std::ptr::null_mut(), std::ptr::null()).is_null());
            assert!(aplan_goal_list(std::ptr::null_mut()).is_null());
        }
    }

    #[test]
    fn test_goal_crud() {
        let h = new_engine();
        let title = c("Build a rocket");
        let intention = c("Reach orbit");
        unsafe {
            let json = aplan_goal_create(h, title.as_ptr(), intention.as_ptr());
            assert!(!json.is_null());
            let json_str = CStr::from_ptr(json).to_str().unwrap();
            let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
            let goal_id = parsed["id"].as_str().unwrap().to_string();
            aplan_string_free(json);

            // Get the goal
            let id_c = c(&goal_id);
            let got = aplan_goal_get(h, id_c.as_ptr());
            assert!(!got.is_null());
            aplan_string_free(got);

            // List goals
            let list = aplan_goal_list(h);
            assert!(!list.is_null());
            let list_str = CStr::from_ptr(list).to_str().unwrap();
            let arr: serde_json::Value = serde_json::from_str(list_str).unwrap();
            assert!(!arr.as_array().unwrap().is_empty());
            aplan_string_free(list);

            aplan_engine_free(h);
        }
    }

    #[test]
    fn test_goal_state_transitions() {
        let h = new_engine();
        let title = c("State test");
        let intention = c("Test transitions");
        unsafe {
            let json = aplan_goal_create(h, title.as_ptr(), intention.as_ptr());
            let parsed: serde_json::Value =
                serde_json::from_str(CStr::from_ptr(json).to_str().unwrap()).unwrap();
            let goal_id = parsed["id"].as_str().unwrap().to_string();
            aplan_string_free(json);

            // Activate first (Draft -> Active)
            let h_ref = &mut *h;
            let gid = agentic_planning::GoalId(uuid::Uuid::parse_str(&goal_id).unwrap());
            let _ = h_ref.engine.activate_goal(gid);

            let id_c = c(&goal_id);

            // Pause
            let reason = c("need a break");
            assert_eq!(
                aplan_goal_pause(h, id_c.as_ptr(), reason.as_ptr()),
                AplanResult::Ok
            );

            // Resume
            assert_eq!(aplan_goal_resume(h, id_c.as_ptr()), AplanResult::Ok);

            // Complete
            assert_eq!(aplan_goal_complete(h, id_c.as_ptr()), AplanResult::Ok);

            aplan_engine_free(h);
        }
    }

    #[test]
    fn test_decision_operations() {
        let h = new_engine();
        unsafe {
            let json_str = c(
                r#"{"question":"Which framework?","context":"Building new app","affected_goals":[]}"#,
            );
            let result = aplan_decision_create(h, json_str.as_ptr());
            assert!(!result.is_null());
            let parsed: serde_json::Value =
                serde_json::from_str(CStr::from_ptr(result).to_str().unwrap()).unwrap();
            let dec_id = parsed["id"].as_str().unwrap().to_string();
            aplan_string_free(result);

            // Get decision
            let id_c = c(&dec_id);
            let got = aplan_decision_get(h, id_c.as_ptr());
            assert!(!got.is_null());
            aplan_string_free(got);

            // List decisions
            let list = aplan_decision_list(h);
            assert!(!list.is_null());
            aplan_string_free(list);

            aplan_engine_free(h);
        }
    }

    #[test]
    fn test_commitment_operations() {
        let h = new_engine();
        unsafe {
            let json_str = c(
                r#"{"promise":{"description":"Ship v1","deliverables":[],"conditions":[]},"stakeholder":{"name":"CEO","role":"executive","importance":0.9}}"#,
            );
            let result = aplan_commitment_create(h, json_str.as_ptr());
            assert!(!result.is_null());
            let parsed: serde_json::Value =
                serde_json::from_str(CStr::from_ptr(result).to_str().unwrap()).unwrap();
            let com_id = parsed["id"].as_str().unwrap().to_string();
            aplan_string_free(result);

            // Get commitment
            let id_c = c(&com_id);
            let got = aplan_commitment_get(h, id_c.as_ptr());
            assert!(!got.is_null());
            aplan_string_free(got);

            // List commitments
            let list = aplan_commitment_list(h);
            assert!(!list.is_null());
            aplan_string_free(list);

            // Fulfill
            assert_eq!(aplan_commitment_fulfill(h, id_c.as_ptr()), AplanResult::Ok);

            aplan_engine_free(h);
        }
    }

    #[test]
    fn test_query_operations() {
        let h = new_engine();
        unsafe {
            let singularity = aplan_singularity_get(h);
            assert!(!singularity.is_null());
            aplan_string_free(singularity);

            let blockers = aplan_blockers_scan(h);
            assert!(!blockers.is_null());
            aplan_string_free(blockers);

            let echoes = aplan_echoes_listen(h);
            assert!(!echoes.is_null());
            aplan_string_free(echoes);

            aplan_engine_free(h);
        }
    }

    #[test]
    fn test_version() {
        let ver = aplan_version();
        assert!(!ver.is_null());
        let s = unsafe { CStr::from_ptr(ver) }.to_str().unwrap();
        assert!(!s.is_empty());
    }

    #[test]
    fn test_last_error() {
        // No error initially
        let err = aplan_last_error();
        assert!(err.is_null());

        // Trigger an error
        unsafe {
            let h = new_engine();
            let bad_id = c("not-a-uuid");
            let _ = aplan_goal_get(h, bad_id.as_ptr());
            let err = aplan_last_error();
            assert!(!err.is_null());
            let msg = CStr::from_ptr(err).to_str().unwrap();
            assert!(msg.contains("UUID") || msg.contains("invalid"));
            aplan_engine_free(h);
        }
    }

    #[test]
    fn test_dream_operations() {
        let h = new_engine();
        let title = c("Dream test goal");
        let intention = c("Test dreaming");
        unsafe {
            // Create a goal first
            let json = aplan_goal_create(h, title.as_ptr(), intention.as_ptr());
            let parsed: serde_json::Value =
                serde_json::from_str(CStr::from_ptr(json).to_str().unwrap()).unwrap();
            let goal_id = parsed["id"].as_str().unwrap().to_string();
            aplan_string_free(json);

            // Activate the goal (dream requires active goal)
            let h_ref = &mut *h;
            let gid = agentic_planning::GoalId(uuid::Uuid::parse_str(&goal_id).unwrap());
            let _ = h_ref.engine.activate_goal(gid);

            // Dream about it
            let id_c = c(&goal_id);
            let dream = aplan_dream_create(h, id_c.as_ptr());
            assert!(!dream.is_null());
            let dream_parsed: serde_json::Value =
                serde_json::from_str(CStr::from_ptr(dream).to_str().unwrap()).unwrap();
            let dream_id = dream_parsed["id"].as_str().unwrap().to_string();
            aplan_string_free(dream);

            // Get dream
            let did_c = c(&dream_id);
            let got = aplan_dream_get(h, did_c.as_ptr());
            assert!(!got.is_null());
            aplan_string_free(got);

            // List dreams
            let list = aplan_dream_list(h);
            assert!(!list.is_null());
            aplan_string_free(list);

            aplan_engine_free(h);
        }
    }
}
