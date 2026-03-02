#ifndef AGENTIC_PLANNING_H
#define AGENTIC_PLANNING_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ── Result codes ─────────────────────────────────────────── */

typedef enum {
    APLAN_OK = 0,
    APLAN_NULL_POINTER = 1,
    APLAN_INVALID_UTF8 = 2,
    APLAN_ENGINE_ERROR = 3,
    APLAN_NOT_FOUND = 4,
    APLAN_VALIDATION_ERROR = 5,
    APLAN_IO_ERROR = 6,
    APLAN_SERIALIZATION_ERROR = 7,
} AplanResult;

/* ── Opaque handle ────────────────────────────────────────── */

typedef struct AplanHandle AplanHandle;

/* ── Engine lifecycle ─────────────────────────────────────── */

AplanHandle* aplan_engine_new(void);
AplanHandle* aplan_engine_new_memory(void);
AplanHandle* aplan_engine_new_file(const char* path);
void aplan_engine_free(AplanHandle* handle);
AplanResult aplan_engine_save(AplanHandle* handle);
AplanResult aplan_engine_load(AplanHandle* handle, const char* path);

/* ── Goal operations ──────────────────────────────────────── */

char* aplan_goal_create(AplanHandle* handle, const char* title, const char* intention);
char* aplan_goal_get(AplanHandle* handle, const char* id);
char* aplan_goal_list(AplanHandle* handle);
AplanResult aplan_goal_pause(AplanHandle* handle, const char* id, const char* reason);
AplanResult aplan_goal_resume(AplanHandle* handle, const char* id);
AplanResult aplan_goal_abandon(AplanHandle* handle, const char* id);
AplanResult aplan_goal_complete(AplanHandle* handle, const char* id);

/* ── Decision operations ──────────────────────────────────── */

char* aplan_decision_create(AplanHandle* handle, const char* json);
char* aplan_decision_get(AplanHandle* handle, const char* id);
char* aplan_decision_list(AplanHandle* handle);
AplanResult aplan_decision_crystallize(AplanHandle* handle, const char* id, const char* chosen_path_id);

/* ── Commitment operations ────────────────────────────────── */

char* aplan_commitment_create(AplanHandle* handle, const char* json);
char* aplan_commitment_get(AplanHandle* handle, const char* id);
char* aplan_commitment_list(AplanHandle* handle);
AplanResult aplan_commitment_fulfill(AplanHandle* handle, const char* id);
AplanResult aplan_commitment_break(AplanHandle* handle, const char* id, const char* reason);

/* ── Dream operations ─────────────────────────────────────── */

char* aplan_dream_create(AplanHandle* handle, const char* goal_id);
char* aplan_dream_get(AplanHandle* handle, const char* id);
char* aplan_dream_list(AplanHandle* handle);

/* ── Query operations ─────────────────────────────────────── */

char* aplan_singularity_get(AplanHandle* handle);
char* aplan_blockers_scan(AplanHandle* handle);
char* aplan_echoes_listen(AplanHandle* handle);

/* ── Utility ──────────────────────────────────────────────── */

void aplan_string_free(char* s);
const char* aplan_version(void);
const char* aplan_last_error(void);

#ifdef __cplusplus
}
#endif

#endif /* AGENTIC_PLANNING_H */
