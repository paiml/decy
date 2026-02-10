//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C851-C875: Task Scheduling & RTOS Patterns -- schedulers, timers,
//! task state machines, priority protocols, and real-time system
//! infrastructure code.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world task scheduling and RTOS patterns
//! commonly found in FreeRTOS, Zephyr, VxWorks, and similar real-time
//! operating systems -- all expressed as valid C99.
//!
//! Organization:
//! - C851-C855: Core schedulers (round-robin, priority, RMS, EDF, timer wheel)
//! - C856-C860: Task management (watchdog, state machine, priority inheritance/ceiling, time-slicing)
//! - C861-C865: Scheduling policies (cooperative, deadline-monotonic, sporadic, aperiodic, MLFQ)
//! - C866-C870: Advanced schedulers (lottery, stride, gang, RTC, interval timer)
//! - C871-C875: System monitoring (CPU load, stack overflow, power mgmt, interrupt priority, DMA)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C851-C855: Core Schedulers
// ============================================================================

/// C851: Round-robin scheduler with circular task queue
#[test]
fn c851_round_robin_scheduler() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_TASKS 16
#define TASK_READY   0
#define TASK_RUNNING 1
#define TASK_BLOCKED 2

typedef struct {
    int task_id;
    int state;
    int time_remaining;
    int quantum;
    uint32_t stack_ptr;
} sched_rr_task_t;

typedef struct {
    sched_rr_task_t tasks[MAX_TASKS];
    int task_count;
    int current_task;
    int tick_count;
    int quantum_default;
} sched_rr_scheduler_t;

void sched_rr_init(sched_rr_scheduler_t *sched, int quantum) {
    sched->task_count = 0;
    sched->current_task = -1;
    sched->tick_count = 0;
    sched->quantum_default = quantum;
}

int sched_rr_add_task(sched_rr_scheduler_t *sched, int task_id, uint32_t sp) {
    if (sched->task_count >= MAX_TASKS) return -1;
    int idx = sched->task_count;
    sched->tasks[idx].task_id = task_id;
    sched->tasks[idx].state = TASK_READY;
    sched->tasks[idx].time_remaining = sched->quantum_default;
    sched->tasks[idx].quantum = sched->quantum_default;
    sched->tasks[idx].stack_ptr = sp;
    sched->task_count++;
    return idx;
}

int sched_rr_next(sched_rr_scheduler_t *sched) {
    if (sched->task_count == 0) return -1;
    int start = (sched->current_task + 1) % sched->task_count;
    int i;
    for (i = 0; i < sched->task_count; i++) {
        int idx = (start + i) % sched->task_count;
        if (sched->tasks[idx].state == TASK_READY) {
            if (sched->current_task >= 0 && sched->current_task < sched->task_count) {
                sched->tasks[sched->current_task].state = TASK_READY;
            }
            sched->current_task = idx;
            sched->tasks[idx].state = TASK_RUNNING;
            sched->tasks[idx].time_remaining = sched->tasks[idx].quantum;
            return idx;
        }
    }
    return -1;
}

int sched_rr_tick(sched_rr_scheduler_t *sched) {
    sched->tick_count++;
    if (sched->current_task < 0) return sched_rr_next(sched);
    sched->tasks[sched->current_task].time_remaining--;
    if (sched->tasks[sched->current_task].time_remaining <= 0) {
        return sched_rr_next(sched);
    }
    return sched->current_task;
}

int sched_rr_test(void) {
    sched_rr_scheduler_t sched;
    sched_rr_init(&sched, 5);
    sched_rr_add_task(&sched, 100, 0x1000);
    sched_rr_add_task(&sched, 101, 0x2000);
    sched_rr_add_task(&sched, 102, 0x3000);
    int first = sched_rr_next(&sched);
    if (first != 0) return -1;
    int t;
    for (t = 0; t < 5; t++) sched_rr_tick(&sched);
    if (sched.current_task != 1) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C851: Round-robin scheduler should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C851: Output should not be empty");
    assert!(code.contains("fn sched_rr_init"), "C851: Should contain sched_rr_init");
    assert!(code.contains("fn sched_rr_next"), "C851: Should contain sched_rr_next");
    assert!(code.contains("fn sched_rr_tick"), "C851: Should contain sched_rr_tick");
}

/// C852: Priority-based preemptive scheduler
#[test]
fn c852_priority_scheduler() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_PRIO_TASKS 32
#define PRIO_LEVELS 8

typedef struct {
    int task_id;
    int priority;
    int state;
    uint32_t stack_ptr;
    int cpu_time;
} sched_prio_task_t;

typedef struct {
    sched_prio_task_t tasks[MAX_PRIO_TASKS];
    int task_count;
    int current_task;
    int ready_bitmap;
} sched_prio_scheduler_t;

void sched_prio_init(sched_prio_scheduler_t *sched) {
    sched->task_count = 0;
    sched->current_task = -1;
    sched->ready_bitmap = 0;
}

int sched_prio_add(sched_prio_scheduler_t *sched, int task_id, int priority) {
    if (sched->task_count >= MAX_PRIO_TASKS) return -1;
    if (priority < 0 || priority >= PRIO_LEVELS) return -1;
    int idx = sched->task_count;
    sched->tasks[idx].task_id = task_id;
    sched->tasks[idx].priority = priority;
    sched->tasks[idx].state = 1;
    sched->tasks[idx].stack_ptr = 0;
    sched->tasks[idx].cpu_time = 0;
    sched->ready_bitmap |= (1 << priority);
    sched->task_count++;
    return idx;
}

int sched_prio_find_highest(sched_prio_scheduler_t *sched) {
    int best = -1;
    int best_prio = -1;
    int i;
    for (i = 0; i < sched->task_count; i++) {
        if (sched->tasks[i].state == 1 && sched->tasks[i].priority > best_prio) {
            best = i;
            best_prio = sched->tasks[i].priority;
        }
    }
    return best;
}

int sched_prio_schedule(sched_prio_scheduler_t *sched) {
    int highest = sched_prio_find_highest(sched);
    if (highest < 0) return -1;
    if (sched->current_task >= 0) {
        sched->tasks[sched->current_task].state = 1;
    }
    sched->current_task = highest;
    sched->tasks[highest].state = 2;
    sched->tasks[highest].cpu_time++;
    return highest;
}

int sched_prio_test(void) {
    sched_prio_scheduler_t sched;
    sched_prio_init(&sched);
    sched_prio_add(&sched, 1, 3);
    sched_prio_add(&sched, 2, 7);
    sched_prio_add(&sched, 3, 1);
    int picked = sched_prio_schedule(&sched);
    if (picked != 1) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C852: Priority scheduler should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C852: Output should not be empty");
    assert!(code.contains("fn sched_prio_init"), "C852: Should contain sched_prio_init");
    assert!(code.contains("fn sched_prio_schedule"), "C852: Should contain sched_prio_schedule");
}

/// C853: Rate-monotonic scheduling (RMS)
#[test]
fn c853_rate_monotonic_scheduler() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_RMS_TASKS 16

typedef struct {
    int task_id;
    uint32_t period;
    uint32_t wcet;
    uint32_t deadline;
    uint32_t next_release;
    int active;
} sched_rms_task_t;

typedef struct {
    sched_rms_task_t tasks[MAX_RMS_TASKS];
    int task_count;
    uint32_t current_time;
    int current_task;
} sched_rms_scheduler_t;

void sched_rms_init(sched_rms_scheduler_t *sched) {
    sched->task_count = 0;
    sched->current_time = 0;
    sched->current_task = -1;
}

int sched_rms_add(sched_rms_scheduler_t *sched, int id, uint32_t period, uint32_t wcet) {
    if (sched->task_count >= MAX_RMS_TASKS) return -1;
    int idx = sched->task_count;
    sched->tasks[idx].task_id = id;
    sched->tasks[idx].period = period;
    sched->tasks[idx].wcet = wcet;
    sched->tasks[idx].deadline = period;
    sched->tasks[idx].next_release = 0;
    sched->tasks[idx].active = 1;
    sched->task_count++;
    return idx;
}

int sched_rms_select(sched_rms_scheduler_t *sched) {
    int best = -1;
    uint32_t best_period = 0xFFFFFFFF;
    int i;
    for (i = 0; i < sched->task_count; i++) {
        if (sched->tasks[i].active && sched->tasks[i].next_release <= sched->current_time) {
            if (sched->tasks[i].period < best_period) {
                best = i;
                best_period = sched->tasks[i].period;
            }
        }
    }
    return best;
}

int sched_rms_tick(sched_rms_scheduler_t *sched) {
    sched->current_time++;
    sched->current_task = sched_rms_select(sched);
    return sched->current_task;
}

int sched_rms_utilization_check(sched_rms_scheduler_t *sched) {
    uint32_t util_num = 0;
    uint32_t util_den = 1;
    int i;
    for (i = 0; i < sched->task_count; i++) {
        util_num += (sched->tasks[i].wcet * 100) / sched->tasks[i].period;
    }
    if (util_num <= 69) return 1;
    return 0;
}

int sched_rms_test(void) {
    sched_rms_scheduler_t sched;
    sched_rms_init(&sched);
    sched_rms_add(&sched, 1, 10, 2);
    sched_rms_add(&sched, 2, 20, 3);
    int ok = sched_rms_utilization_check(&sched);
    if (!ok) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C853: RMS scheduler should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C853: Output should not be empty");
    assert!(code.contains("fn sched_rms_init"), "C853: Should contain sched_rms_init");
    assert!(code.contains("fn sched_rms_select"), "C853: Should contain sched_rms_select");
}

/// C854: Earliest deadline first (EDF) scheduler
#[test]
fn c854_earliest_deadline_first() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_EDF_TASKS 16

typedef struct {
    int task_id;
    uint32_t period;
    uint32_t wcet;
    uint32_t abs_deadline;
    uint32_t remaining;
    int active;
} sched_edf_task_t;

typedef struct {
    sched_edf_task_t tasks[MAX_EDF_TASKS];
    int task_count;
    uint32_t current_time;
    int current_task;
    int deadline_misses;
} sched_edf_scheduler_t;

void sched_edf_init(sched_edf_scheduler_t *sched) {
    sched->task_count = 0;
    sched->current_time = 0;
    sched->current_task = -1;
    sched->deadline_misses = 0;
}

int sched_edf_add(sched_edf_scheduler_t *sched, int id, uint32_t period, uint32_t wcet) {
    if (sched->task_count >= MAX_EDF_TASKS) return -1;
    int idx = sched->task_count;
    sched->tasks[idx].task_id = id;
    sched->tasks[idx].period = period;
    sched->tasks[idx].wcet = wcet;
    sched->tasks[idx].abs_deadline = period;
    sched->tasks[idx].remaining = wcet;
    sched->tasks[idx].active = 1;
    sched->task_count++;
    return idx;
}

int sched_edf_select(sched_edf_scheduler_t *sched) {
    int best = -1;
    uint32_t earliest = 0xFFFFFFFF;
    int i;
    for (i = 0; i < sched->task_count; i++) {
        if (sched->tasks[i].active && sched->tasks[i].remaining > 0) {
            if (sched->tasks[i].abs_deadline < earliest) {
                best = i;
                earliest = sched->tasks[i].abs_deadline;
            }
        }
    }
    return best;
}

void sched_edf_check_deadlines(sched_edf_scheduler_t *sched) {
    int i;
    for (i = 0; i < sched->task_count; i++) {
        if (sched->tasks[i].remaining > 0 && sched->current_time >= sched->tasks[i].abs_deadline) {
            sched->deadline_misses++;
            sched->tasks[i].remaining = 0;
        }
    }
}

int sched_edf_tick(sched_edf_scheduler_t *sched) {
    sched->current_time++;
    sched_edf_check_deadlines(sched);
    int sel = sched_edf_select(sched);
    if (sel >= 0) {
        sched->tasks[sel].remaining--;
        if (sched->tasks[sel].remaining == 0) {
            sched->tasks[sel].abs_deadline += sched->tasks[sel].period;
            sched->tasks[sel].remaining = sched->tasks[sel].wcet;
        }
    }
    sched->current_task = sel;
    return sel;
}

int sched_edf_test(void) {
    sched_edf_scheduler_t sched;
    sched_edf_init(&sched);
    sched_edf_add(&sched, 1, 5, 1);
    sched_edf_add(&sched, 2, 10, 3);
    int r = sched_edf_tick(&sched);
    if (r < 0) return -1;
    if (sched.deadline_misses != 0) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C854: EDF scheduler should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C854: Output should not be empty");
    assert!(code.contains("fn sched_edf_init"), "C854: Should contain sched_edf_init");
    assert!(code.contains("fn sched_edf_select"), "C854: Should contain sched_edf_select");
}

/// C855: Hierarchical timer wheel
#[test]
fn c855_timer_wheel() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define WHEEL_SIZE 64
#define MAX_TIMERS 128

typedef struct {
    int timer_id;
    uint32_t expires;
    int active;
    int callback_id;
    int slot;
} sched_tw_timer_t;

typedef struct {
    sched_tw_timer_t timers[MAX_TIMERS];
    int timer_count;
    int slots[WHEEL_SIZE];
    uint32_t current_tick;
    int current_slot;
    int fired_count;
} sched_tw_wheel_t;

void sched_tw_init(sched_tw_wheel_t *tw) {
    tw->timer_count = 0;
    tw->current_tick = 0;
    tw->current_slot = 0;
    tw->fired_count = 0;
    int i;
    for (i = 0; i < WHEEL_SIZE; i++) {
        tw->slots[i] = -1;
    }
}

int sched_tw_add(sched_tw_wheel_t *tw, int timer_id, uint32_t ticks, int cb_id) {
    if (tw->timer_count >= MAX_TIMERS) return -1;
    int idx = tw->timer_count;
    uint32_t expires = tw->current_tick + ticks;
    int slot = (int)(expires % WHEEL_SIZE);
    tw->timers[idx].timer_id = timer_id;
    tw->timers[idx].expires = expires;
    tw->timers[idx].active = 1;
    tw->timers[idx].callback_id = cb_id;
    tw->timers[idx].slot = slot;
    tw->slots[slot] = idx;
    tw->timer_count++;
    return idx;
}

int sched_tw_cancel(sched_tw_wheel_t *tw, int idx) {
    if (idx < 0 || idx >= tw->timer_count) return -1;
    tw->timers[idx].active = 0;
    return 0;
}

int sched_tw_advance(sched_tw_wheel_t *tw) {
    tw->current_tick++;
    tw->current_slot = (int)(tw->current_tick % WHEEL_SIZE);
    int fired = 0;
    int i;
    for (i = 0; i < tw->timer_count; i++) {
        if (tw->timers[i].active && tw->timers[i].expires == tw->current_tick) {
            tw->timers[i].active = 0;
            fired++;
            tw->fired_count++;
        }
    }
    return fired;
}

int sched_tw_pending(sched_tw_wheel_t *tw) {
    int count = 0;
    int i;
    for (i = 0; i < tw->timer_count; i++) {
        if (tw->timers[i].active) count++;
    }
    return count;
}

int sched_tw_test(void) {
    sched_tw_wheel_t tw;
    sched_tw_init(&tw);
    sched_tw_add(&tw, 1, 5, 100);
    sched_tw_add(&tw, 2, 10, 200);
    if (sched_tw_pending(&tw) != 2) return -1;
    int i;
    for (i = 0; i < 5; i++) sched_tw_advance(&tw);
    if (tw.fired_count != 1) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C855: Timer wheel should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C855: Output should not be empty");
    assert!(code.contains("fn sched_tw_init"), "C855: Should contain sched_tw_init");
    assert!(code.contains("fn sched_tw_advance"), "C855: Should contain sched_tw_advance");
}

// ============================================================================
// C856-C860: Task Management
// ============================================================================

/// C856: Watchdog timer with timeout monitoring
#[test]
fn c856_watchdog_timer() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_WATCHDOGS 8
#define WD_ACTIVE   1
#define WD_EXPIRED  2
#define WD_DISABLED 0

typedef struct {
    int wd_id;
    uint32_t timeout;
    uint32_t counter;
    int state;
    int bark_count;
} sched_wd_timer_t;

typedef struct {
    sched_wd_timer_t watchdogs[MAX_WATCHDOGS];
    int wd_count;
    uint32_t system_tick;
    int total_barks;
} sched_wd_manager_t;

void sched_wd_init(sched_wd_manager_t *mgr) {
    mgr->wd_count = 0;
    mgr->system_tick = 0;
    mgr->total_barks = 0;
}

int sched_wd_create(sched_wd_manager_t *mgr, int id, uint32_t timeout) {
    if (mgr->wd_count >= MAX_WATCHDOGS) return -1;
    int idx = mgr->wd_count;
    mgr->watchdogs[idx].wd_id = id;
    mgr->watchdogs[idx].timeout = timeout;
    mgr->watchdogs[idx].counter = timeout;
    mgr->watchdogs[idx].state = WD_ACTIVE;
    mgr->watchdogs[idx].bark_count = 0;
    mgr->wd_count++;
    return idx;
}

void sched_wd_kick(sched_wd_manager_t *mgr, int idx) {
    if (idx >= 0 && idx < mgr->wd_count) {
        mgr->watchdogs[idx].counter = mgr->watchdogs[idx].timeout;
        mgr->watchdogs[idx].state = WD_ACTIVE;
    }
}

int sched_wd_tick(sched_wd_manager_t *mgr) {
    mgr->system_tick++;
    int expired = 0;
    int i;
    for (i = 0; i < mgr->wd_count; i++) {
        if (mgr->watchdogs[i].state == WD_ACTIVE) {
            if (mgr->watchdogs[i].counter > 0) {
                mgr->watchdogs[i].counter--;
            }
            if (mgr->watchdogs[i].counter == 0) {
                mgr->watchdogs[i].state = WD_EXPIRED;
                mgr->watchdogs[i].bark_count++;
                mgr->total_barks++;
                expired++;
            }
        }
    }
    return expired;
}

int sched_wd_test(void) {
    sched_wd_manager_t mgr;
    sched_wd_init(&mgr);
    int w = sched_wd_create(&mgr, 1, 3);
    int i;
    for (i = 0; i < 3; i++) sched_wd_tick(&mgr);
    if (mgr.watchdogs[w].state != WD_EXPIRED) return -1;
    sched_wd_kick(&mgr, w);
    if (mgr.watchdogs[w].state != WD_ACTIVE) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C856: Watchdog timer should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C856: Output should not be empty");
    assert!(code.contains("fn sched_wd_init"), "C856: Should contain sched_wd_init");
    assert!(code.contains("fn sched_wd_tick"), "C856: Should contain sched_wd_tick");
}

/// C857: Task state machine (ready/running/blocked/suspended)
#[test]
fn c857_task_state_machine() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_TSM_TASKS 16
#define TSM_READY     0
#define TSM_RUNNING   1
#define TSM_BLOCKED   2
#define TSM_SUSPENDED 3
#define TSM_TERMINATED 4

typedef struct {
    int task_id;
    int state;
    int prev_state;
    uint32_t block_reason;
    uint32_t transitions;
} sched_tsm_task_t;

typedef struct {
    sched_tsm_task_t tasks[MAX_TSM_TASKS];
    int task_count;
    int running_task;
} sched_tsm_t;

void sched_tsm_init(sched_tsm_t *tsm) {
    tsm->task_count = 0;
    tsm->running_task = -1;
}

int sched_tsm_create(sched_tsm_t *tsm, int id) {
    if (tsm->task_count >= MAX_TSM_TASKS) return -1;
    int idx = tsm->task_count;
    tsm->tasks[idx].task_id = id;
    tsm->tasks[idx].state = TSM_READY;
    tsm->tasks[idx].prev_state = TSM_READY;
    tsm->tasks[idx].block_reason = 0;
    tsm->tasks[idx].transitions = 0;
    tsm->task_count++;
    return idx;
}

int sched_tsm_transition(sched_tsm_t *tsm, int idx, int new_state) {
    if (idx < 0 || idx >= tsm->task_count) return -1;
    int cur = tsm->tasks[idx].state;
    if (cur == TSM_TERMINATED) return -2;
    if (cur == TSM_READY && new_state == TSM_RUNNING) {
        tsm->tasks[idx].prev_state = cur;
        tsm->tasks[idx].state = TSM_RUNNING;
        tsm->running_task = idx;
    } else if (cur == TSM_RUNNING && new_state == TSM_READY) {
        tsm->tasks[idx].prev_state = cur;
        tsm->tasks[idx].state = TSM_READY;
        tsm->running_task = -1;
    } else if (cur == TSM_RUNNING && new_state == TSM_BLOCKED) {
        tsm->tasks[idx].prev_state = cur;
        tsm->tasks[idx].state = TSM_BLOCKED;
        tsm->running_task = -1;
    } else if (cur == TSM_BLOCKED && new_state == TSM_READY) {
        tsm->tasks[idx].prev_state = cur;
        tsm->tasks[idx].state = TSM_READY;
    } else if (new_state == TSM_SUSPENDED) {
        tsm->tasks[idx].prev_state = cur;
        tsm->tasks[idx].state = TSM_SUSPENDED;
        if (tsm->running_task == idx) tsm->running_task = -1;
    } else if (cur == TSM_SUSPENDED && new_state == TSM_READY) {
        tsm->tasks[idx].prev_state = cur;
        tsm->tasks[idx].state = TSM_READY;
    } else {
        return -3;
    }
    tsm->tasks[idx].transitions++;
    return 0;
}

int sched_tsm_count_state(sched_tsm_t *tsm, int state) {
    int count = 0;
    int i;
    for (i = 0; i < tsm->task_count; i++) {
        if (tsm->tasks[i].state == state) count++;
    }
    return count;
}

int sched_tsm_test(void) {
    sched_tsm_t tsm;
    sched_tsm_init(&tsm);
    int t0 = sched_tsm_create(&tsm, 10);
    int t1 = sched_tsm_create(&tsm, 20);
    sched_tsm_transition(&tsm, t0, TSM_RUNNING);
    if (tsm.tasks[t0].state != TSM_RUNNING) return -1;
    sched_tsm_transition(&tsm, t0, TSM_BLOCKED);
    if (tsm.tasks[t0].state != TSM_BLOCKED) return -2;
    sched_tsm_transition(&tsm, t0, TSM_READY);
    if (sched_tsm_count_state(&tsm, TSM_READY) != 2) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C857: Task state machine should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C857: Output should not be empty");
    assert!(code.contains("fn sched_tsm_init"), "C857: Should contain sched_tsm_init");
    assert!(code.contains("fn sched_tsm_transition"), "C857: Should contain sched_tsm_transition");
}

/// C858: Priority inheritance protocol
#[test]
fn c858_priority_inheritance() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_PI_TASKS 8
#define MAX_PI_MUTEXES 4

typedef struct {
    int task_id;
    int base_priority;
    int effective_priority;
    int holding_mutex;
    int blocked_on;
} sched_pi_task_t;

typedef struct {
    int mutex_id;
    int owner;
    int locked;
} sched_pi_mutex_t;

typedef struct {
    sched_pi_task_t tasks[MAX_PI_TASKS];
    sched_pi_mutex_t mutexes[MAX_PI_MUTEXES];
    int task_count;
    int mutex_count;
} sched_pi_system_t;

void sched_pi_init(sched_pi_system_t *sys) {
    sys->task_count = 0;
    sys->mutex_count = 0;
}

int sched_pi_add_task(sched_pi_system_t *sys, int id, int prio) {
    if (sys->task_count >= MAX_PI_TASKS) return -1;
    int idx = sys->task_count;
    sys->tasks[idx].task_id = id;
    sys->tasks[idx].base_priority = prio;
    sys->tasks[idx].effective_priority = prio;
    sys->tasks[idx].holding_mutex = -1;
    sys->tasks[idx].blocked_on = -1;
    sys->task_count++;
    return idx;
}

int sched_pi_add_mutex(sched_pi_system_t *sys, int id) {
    if (sys->mutex_count >= MAX_PI_MUTEXES) return -1;
    int idx = sys->mutex_count;
    sys->mutexes[idx].mutex_id = id;
    sys->mutexes[idx].owner = -1;
    sys->mutexes[idx].locked = 0;
    sys->mutex_count++;
    return idx;
}

int sched_pi_lock(sched_pi_system_t *sys, int task_idx, int mutex_idx) {
    if (sys->mutexes[mutex_idx].locked == 0) {
        sys->mutexes[mutex_idx].locked = 1;
        sys->mutexes[mutex_idx].owner = task_idx;
        sys->tasks[task_idx].holding_mutex = mutex_idx;
        return 0;
    }
    int owner = sys->mutexes[mutex_idx].owner;
    sys->tasks[task_idx].blocked_on = mutex_idx;
    if (sys->tasks[task_idx].effective_priority > sys->tasks[owner].effective_priority) {
        sys->tasks[owner].effective_priority = sys->tasks[task_idx].effective_priority;
    }
    return 1;
}

int sched_pi_unlock(sched_pi_system_t *sys, int task_idx, int mutex_idx) {
    if (sys->mutexes[mutex_idx].owner != task_idx) return -1;
    sys->mutexes[mutex_idx].locked = 0;
    sys->mutexes[mutex_idx].owner = -1;
    sys->tasks[task_idx].holding_mutex = -1;
    sys->tasks[task_idx].effective_priority = sys->tasks[task_idx].base_priority;
    return 0;
}

int sched_pi_test(void) {
    sched_pi_system_t sys;
    sched_pi_init(&sys);
    int low = sched_pi_add_task(&sys, 1, 1);
    int high = sched_pi_add_task(&sys, 2, 5);
    int m = sched_pi_add_mutex(&sys, 1);
    sched_pi_lock(&sys, low, m);
    sched_pi_lock(&sys, high, m);
    if (sys.tasks[low].effective_priority != 5) return -1;
    sched_pi_unlock(&sys, low, m);
    if (sys.tasks[low].effective_priority != 1) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C858: Priority inheritance should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C858: Output should not be empty");
    assert!(code.contains("fn sched_pi_init"), "C858: Should contain sched_pi_init");
    assert!(code.contains("fn sched_pi_lock"), "C858: Should contain sched_pi_lock");
}

/// C859: Priority ceiling protocol
#[test]
fn c859_priority_ceiling() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_PC_TASKS 8
#define MAX_PC_RESOURCES 4

typedef struct {
    int resource_id;
    int ceiling;
    int owner;
    int locked;
} sched_pc_resource_t;

typedef struct {
    int task_id;
    int priority;
    int active_priority;
} sched_pc_task_t;

typedef struct {
    sched_pc_task_t tasks[MAX_PC_TASKS];
    sched_pc_resource_t resources[MAX_PC_RESOURCES];
    int task_count;
    int res_count;
    int system_ceiling;
} sched_pc_system_t;

void sched_pc_init(sched_pc_system_t *sys) {
    sys->task_count = 0;
    sys->res_count = 0;
    sys->system_ceiling = 0;
}

int sched_pc_add_task(sched_pc_system_t *sys, int id, int prio) {
    if (sys->task_count >= MAX_PC_TASKS) return -1;
    int idx = sys->task_count;
    sys->tasks[idx].task_id = id;
    sys->tasks[idx].priority = prio;
    sys->tasks[idx].active_priority = prio;
    sys->task_count++;
    return idx;
}

int sched_pc_add_resource(sched_pc_system_t *sys, int id, int ceiling) {
    if (sys->res_count >= MAX_PC_RESOURCES) return -1;
    int idx = sys->res_count;
    sys->resources[idx].resource_id = id;
    sys->resources[idx].ceiling = ceiling;
    sys->resources[idx].owner = -1;
    sys->resources[idx].locked = 0;
    sys->res_count++;
    return idx;
}

int sched_pc_lock(sched_pc_system_t *sys, int task_idx, int res_idx) {
    if (sys->resources[res_idx].locked) return -1;
    if (sys->tasks[task_idx].active_priority <= sys->system_ceiling &&
        sys->resources[res_idx].owner != task_idx) {
        return -2;
    }
    sys->resources[res_idx].locked = 1;
    sys->resources[res_idx].owner = task_idx;
    if (sys->resources[res_idx].ceiling > sys->system_ceiling) {
        sys->system_ceiling = sys->resources[res_idx].ceiling;
    }
    sys->tasks[task_idx].active_priority = sys->resources[res_idx].ceiling;
    return 0;
}

void sched_pc_unlock(sched_pc_system_t *sys, int task_idx, int res_idx) {
    sys->resources[res_idx].locked = 0;
    sys->resources[res_idx].owner = -1;
    sys->tasks[task_idx].active_priority = sys->tasks[task_idx].priority;
    sys->system_ceiling = 0;
    int i;
    for (i = 0; i < sys->res_count; i++) {
        if (sys->resources[i].locked && sys->resources[i].ceiling > sys->system_ceiling) {
            sys->system_ceiling = sys->resources[i].ceiling;
        }
    }
}

int sched_pc_test(void) {
    sched_pc_system_t sys;
    sched_pc_init(&sys);
    sched_pc_add_task(&sys, 1, 3);
    int r = sched_pc_add_resource(&sys, 1, 5);
    sched_pc_lock(&sys, 0, r);
    if (sys.tasks[0].active_priority != 5) return -1;
    sched_pc_unlock(&sys, 0, r);
    if (sys.tasks[0].active_priority != 3) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C859: Priority ceiling should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C859: Output should not be empty");
    assert!(code.contains("fn sched_pc_init"), "C859: Should contain sched_pc_init");
    assert!(code.contains("fn sched_pc_lock"), "C859: Should contain sched_pc_lock");
}

/// C860: Time-slicing scheduler with preemption
#[test]
fn c860_time_slicing_scheduler() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_TS_TASKS 16

typedef struct {
    int task_id;
    int priority;
    int state;
    uint32_t slice_remaining;
    uint32_t total_runtime;
    uint32_t preempt_count;
} sched_ts_task_t;

typedef struct {
    sched_ts_task_t tasks[MAX_TS_TASKS];
    int task_count;
    int current;
    uint32_t default_slice;
    uint32_t tick;
} sched_ts_scheduler_t;

void sched_ts_init(sched_ts_scheduler_t *sched, uint32_t slice) {
    sched->task_count = 0;
    sched->current = -1;
    sched->default_slice = slice;
    sched->tick = 0;
}

int sched_ts_add(sched_ts_scheduler_t *sched, int id, int prio) {
    if (sched->task_count >= MAX_TS_TASKS) return -1;
    int idx = sched->task_count;
    sched->tasks[idx].task_id = id;
    sched->tasks[idx].priority = prio;
    sched->tasks[idx].state = 1;
    sched->tasks[idx].slice_remaining = sched->default_slice;
    sched->tasks[idx].total_runtime = 0;
    sched->tasks[idx].preempt_count = 0;
    sched->task_count++;
    return idx;
}

int sched_ts_pick_next(sched_ts_scheduler_t *sched) {
    int best = -1;
    int best_prio = -1;
    int i;
    for (i = 0; i < sched->task_count; i++) {
        if (sched->tasks[i].state == 1 && sched->tasks[i].priority > best_prio) {
            best = i;
            best_prio = sched->tasks[i].priority;
        }
    }
    return best;
}

int sched_ts_tick(sched_ts_scheduler_t *sched) {
    sched->tick++;
    if (sched->current >= 0) {
        sched->tasks[sched->current].total_runtime++;
        sched->tasks[sched->current].slice_remaining--;
        if (sched->tasks[sched->current].slice_remaining == 0) {
            sched->tasks[sched->current].preempt_count++;
            sched->tasks[sched->current].slice_remaining = sched->default_slice;
            sched->tasks[sched->current].state = 1;
            int next = sched_ts_pick_next(sched);
            sched->current = next;
            if (next >= 0) sched->tasks[next].state = 2;
            return next;
        }
    } else {
        int next = sched_ts_pick_next(sched);
        sched->current = next;
        if (next >= 0) sched->tasks[next].state = 2;
        return next;
    }
    return sched->current;
}

int sched_ts_test(void) {
    sched_ts_scheduler_t sched;
    sched_ts_init(&sched, 3);
    sched_ts_add(&sched, 1, 5);
    sched_ts_add(&sched, 2, 5);
    int r = sched_ts_tick(&sched);
    if (r < 0) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C860: Time-slicing scheduler should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C860: Output should not be empty");
    assert!(code.contains("fn sched_ts_init"), "C860: Should contain sched_ts_init");
    assert!(code.contains("fn sched_ts_tick"), "C860: Should contain sched_ts_tick");
}

// ============================================================================
// C861-C865: Scheduling Policies
// ============================================================================

/// C861: Cooperative scheduler (yield-based)
#[test]
fn c861_cooperative_scheduler() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_COOP_TASKS 16
#define COOP_READY   0
#define COOP_RUNNING 1
#define COOP_DONE    2

typedef struct {
    int task_id;
    int state;
    uint32_t progress;
    uint32_t work_total;
    int yield_count;
} sched_coop_task_t;

typedef struct {
    sched_coop_task_t tasks[MAX_COOP_TASKS];
    int task_count;
    int current;
    int completed;
} sched_coop_scheduler_t;

void sched_coop_init(sched_coop_scheduler_t *sched) {
    sched->task_count = 0;
    sched->current = -1;
    sched->completed = 0;
}

int sched_coop_add(sched_coop_scheduler_t *sched, int id, uint32_t work) {
    if (sched->task_count >= MAX_COOP_TASKS) return -1;
    int idx = sched->task_count;
    sched->tasks[idx].task_id = id;
    sched->tasks[idx].state = COOP_READY;
    sched->tasks[idx].progress = 0;
    sched->tasks[idx].work_total = work;
    sched->tasks[idx].yield_count = 0;
    sched->task_count++;
    return idx;
}

int sched_coop_yield(sched_coop_scheduler_t *sched) {
    if (sched->current >= 0) {
        sched->tasks[sched->current].yield_count++;
        sched->tasks[sched->current].state = COOP_READY;
    }
    int start = (sched->current + 1) % sched->task_count;
    int i;
    for (i = 0; i < sched->task_count; i++) {
        int idx = (start + i) % sched->task_count;
        if (sched->tasks[idx].state == COOP_READY) {
            sched->current = idx;
            sched->tasks[idx].state = COOP_RUNNING;
            return idx;
        }
    }
    sched->current = -1;
    return -1;
}

int sched_coop_run_step(sched_coop_scheduler_t *sched) {
    if (sched->current < 0) return -1;
    int c = sched->current;
    sched->tasks[c].progress++;
    if (sched->tasks[c].progress >= sched->tasks[c].work_total) {
        sched->tasks[c].state = COOP_DONE;
        sched->completed++;
        return sched_coop_yield(sched);
    }
    return c;
}

int sched_coop_test(void) {
    sched_coop_scheduler_t sched;
    sched_coop_init(&sched);
    sched_coop_add(&sched, 1, 3);
    sched_coop_add(&sched, 2, 2);
    sched_coop_yield(&sched);
    if (sched.current != 0) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C861: Cooperative scheduler should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C861: Output should not be empty");
    assert!(code.contains("fn sched_coop_init"), "C861: Should contain sched_coop_init");
    assert!(code.contains("fn sched_coop_yield"), "C861: Should contain sched_coop_yield");
}

/// C862: Deadline-monotonic scheduler
#[test]
fn c862_deadline_monotonic_scheduler() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_DM_TASKS 16

typedef struct {
    int task_id;
    uint32_t period;
    uint32_t relative_deadline;
    uint32_t wcet;
    uint32_t next_release;
    uint32_t abs_deadline;
    int active;
} sched_dm_task_t;

typedef struct {
    sched_dm_task_t tasks[MAX_DM_TASKS];
    int task_count;
    uint32_t current_time;
    int current_task;
} sched_dm_scheduler_t;

void sched_dm_init(sched_dm_scheduler_t *sched) {
    sched->task_count = 0;
    sched->current_time = 0;
    sched->current_task = -1;
}

int sched_dm_add(sched_dm_scheduler_t *sched, int id, uint32_t period,
                 uint32_t deadline, uint32_t wcet) {
    if (sched->task_count >= MAX_DM_TASKS) return -1;
    int idx = sched->task_count;
    sched->tasks[idx].task_id = id;
    sched->tasks[idx].period = period;
    sched->tasks[idx].relative_deadline = deadline;
    sched->tasks[idx].wcet = wcet;
    sched->tasks[idx].next_release = 0;
    sched->tasks[idx].abs_deadline = deadline;
    sched->tasks[idx].active = 1;
    sched->task_count++;
    return idx;
}

int sched_dm_select(sched_dm_scheduler_t *sched) {
    int best = -1;
    uint32_t shortest_dl = 0xFFFFFFFF;
    int i;
    for (i = 0; i < sched->task_count; i++) {
        if (sched->tasks[i].active && sched->tasks[i].next_release <= sched->current_time) {
            if (sched->tasks[i].relative_deadline < shortest_dl) {
                best = i;
                shortest_dl = sched->tasks[i].relative_deadline;
            }
        }
    }
    return best;
}

int sched_dm_tick(sched_dm_scheduler_t *sched) {
    sched->current_time++;
    sched->current_task = sched_dm_select(sched);
    return sched->current_task;
}

int sched_dm_test(void) {
    sched_dm_scheduler_t sched;
    sched_dm_init(&sched);
    sched_dm_add(&sched, 1, 10, 8, 2);
    sched_dm_add(&sched, 2, 20, 15, 4);
    int sel = sched_dm_select(&sched);
    if (sel != 0) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C862: Deadline-monotonic scheduler should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C862: Output should not be empty");
    assert!(code.contains("fn sched_dm_init"), "C862: Should contain sched_dm_init");
    assert!(code.contains("fn sched_dm_select"), "C862: Should contain sched_dm_select");
}

/// C863: Sporadic server for aperiodic tasks
#[test]
fn c863_sporadic_server() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_SS_REQUESTS 32

typedef struct {
    int request_id;
    uint32_t arrival;
    uint32_t exec_time;
    int served;
} sched_ss_request_t;

typedef struct {
    int server_priority;
    uint32_t budget;
    uint32_t max_budget;
    uint32_t replenish_period;
    uint32_t next_replenish;
    sched_ss_request_t queue[MAX_SS_REQUESTS];
    int queue_head;
    int queue_tail;
    int queue_size;
    uint32_t current_time;
    int served_count;
} sched_ss_server_t;

void sched_ss_init(sched_ss_server_t *srv, int prio, uint32_t budget, uint32_t period) {
    srv->server_priority = prio;
    srv->budget = budget;
    srv->max_budget = budget;
    srv->replenish_period = period;
    srv->next_replenish = period;
    srv->queue_head = 0;
    srv->queue_tail = 0;
    srv->queue_size = 0;
    srv->current_time = 0;
    srv->served_count = 0;
}

int sched_ss_enqueue(sched_ss_server_t *srv, int id, uint32_t exec_time) {
    if (srv->queue_size >= MAX_SS_REQUESTS) return -1;
    srv->queue[srv->queue_tail].request_id = id;
    srv->queue[srv->queue_tail].arrival = srv->current_time;
    srv->queue[srv->queue_tail].exec_time = exec_time;
    srv->queue[srv->queue_tail].served = 0;
    srv->queue_tail = (srv->queue_tail + 1) % MAX_SS_REQUESTS;
    srv->queue_size++;
    return 0;
}

int sched_ss_serve(sched_ss_server_t *srv) {
    if (srv->queue_size == 0 || srv->budget == 0) return -1;
    int head = srv->queue_head;
    uint32_t needed = srv->queue[head].exec_time;
    if (needed <= srv->budget) {
        srv->budget -= needed;
        srv->queue[head].served = 1;
        srv->queue_head = (srv->queue_head + 1) % MAX_SS_REQUESTS;
        srv->queue_size--;
        srv->served_count++;
        return 0;
    }
    return -2;
}

void sched_ss_tick(sched_ss_server_t *srv) {
    srv->current_time++;
    if (srv->current_time >= srv->next_replenish) {
        srv->budget = srv->max_budget;
        srv->next_replenish += srv->replenish_period;
    }
}

int sched_ss_test(void) {
    sched_ss_server_t srv;
    sched_ss_init(&srv, 5, 10, 20);
    sched_ss_enqueue(&srv, 1, 3);
    sched_ss_enqueue(&srv, 2, 5);
    sched_ss_serve(&srv);
    if (srv.served_count != 1) return -1;
    if (srv.budget != 7) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C863: Sporadic server should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C863: Output should not be empty");
    assert!(code.contains("fn sched_ss_init"), "C863: Should contain sched_ss_init");
    assert!(code.contains("fn sched_ss_serve"), "C863: Should contain sched_ss_serve");
}

/// C864: Aperiodic task handler with background server
#[test]
fn c864_aperiodic_task_handler() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_APERIODIC 16

typedef struct {
    int task_id;
    uint32_t arrival_time;
    uint32_t exec_time;
    uint32_t remaining;
    int completed;
} sched_ap_task_t;

typedef struct {
    sched_ap_task_t tasks[MAX_APERIODIC];
    int task_count;
    uint32_t current_time;
    int active_task;
    int completed_count;
    uint32_t total_response_time;
} sched_ap_handler_t;

void sched_ap_init(sched_ap_handler_t *h) {
    h->task_count = 0;
    h->current_time = 0;
    h->active_task = -1;
    h->completed_count = 0;
    h->total_response_time = 0;
}

int sched_ap_submit(sched_ap_handler_t *h, int id, uint32_t exec_time) {
    if (h->task_count >= MAX_APERIODIC) return -1;
    int idx = h->task_count;
    h->tasks[idx].task_id = id;
    h->tasks[idx].arrival_time = h->current_time;
    h->tasks[idx].exec_time = exec_time;
    h->tasks[idx].remaining = exec_time;
    h->tasks[idx].completed = 0;
    h->task_count++;
    return idx;
}

int sched_ap_select_fifo(sched_ap_handler_t *h) {
    int i;
    for (i = 0; i < h->task_count; i++) {
        if (!h->tasks[i].completed && h->tasks[i].remaining > 0) {
            return i;
        }
    }
    return -1;
}

int sched_ap_tick(sched_ap_handler_t *h) {
    h->current_time++;
    if (h->active_task < 0) {
        h->active_task = sched_ap_select_fifo(h);
    }
    if (h->active_task >= 0) {
        h->tasks[h->active_task].remaining--;
        if (h->tasks[h->active_task].remaining == 0) {
            h->tasks[h->active_task].completed = 1;
            h->total_response_time += h->current_time - h->tasks[h->active_task].arrival_time;
            h->completed_count++;
            h->active_task = sched_ap_select_fifo(h);
        }
    }
    return h->active_task;
}

int sched_ap_test(void) {
    sched_ap_handler_t h;
    sched_ap_init(&h);
    sched_ap_submit(&h, 1, 3);
    sched_ap_submit(&h, 2, 2);
    int i;
    for (i = 0; i < 5; i++) sched_ap_tick(&h);
    if (h.completed_count != 2) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C864: Aperiodic task handler should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C864: Output should not be empty");
    assert!(code.contains("fn sched_ap_init"), "C864: Should contain sched_ap_init");
    assert!(code.contains("fn sched_ap_tick"), "C864: Should contain sched_ap_tick");
}

/// C865: Multilevel feedback queue (MLFQ)
#[test]
fn c865_multilevel_feedback_queue() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MLFQ_LEVELS 4
#define MLFQ_MAX_TASKS 32

typedef struct {
    int task_id;
    int current_level;
    uint32_t allotment_remaining;
    uint32_t total_runtime;
    int active;
} sched_mlfq_task_t;

typedef struct {
    sched_mlfq_task_t tasks[MLFQ_MAX_TASKS];
    int task_count;
    uint32_t level_quantum[MLFQ_LEVELS];
    uint32_t level_allotment[MLFQ_LEVELS];
    int current_task;
    uint32_t boost_interval;
    uint32_t ticks_since_boost;
} sched_mlfq_scheduler_t;

void sched_mlfq_init(sched_mlfq_scheduler_t *sched, uint32_t boost_interval) {
    sched->task_count = 0;
    sched->current_task = -1;
    sched->boost_interval = boost_interval;
    sched->ticks_since_boost = 0;
    int i;
    for (i = 0; i < MLFQ_LEVELS; i++) {
        sched->level_quantum[i] = (uint32_t)((i + 1) * 2);
        sched->level_allotment[i] = (uint32_t)((i + 1) * 10);
    }
}

int sched_mlfq_add(sched_mlfq_scheduler_t *sched, int id) {
    if (sched->task_count >= MLFQ_MAX_TASKS) return -1;
    int idx = sched->task_count;
    sched->tasks[idx].task_id = id;
    sched->tasks[idx].current_level = 0;
    sched->tasks[idx].allotment_remaining = sched->level_allotment[0];
    sched->tasks[idx].total_runtime = 0;
    sched->tasks[idx].active = 1;
    sched->task_count++;
    return idx;
}

void sched_mlfq_boost(sched_mlfq_scheduler_t *sched) {
    int i;
    for (i = 0; i < sched->task_count; i++) {
        if (sched->tasks[i].active) {
            sched->tasks[i].current_level = 0;
            sched->tasks[i].allotment_remaining = sched->level_allotment[0];
        }
    }
    sched->ticks_since_boost = 0;
}

int sched_mlfq_select(sched_mlfq_scheduler_t *sched) {
    int level;
    for (level = 0; level < MLFQ_LEVELS; level++) {
        int i;
        for (i = 0; i < sched->task_count; i++) {
            if (sched->tasks[i].active && sched->tasks[i].current_level == level) {
                return i;
            }
        }
    }
    return -1;
}

int sched_mlfq_tick(sched_mlfq_scheduler_t *sched) {
    sched->ticks_since_boost++;
    if (sched->ticks_since_boost >= sched->boost_interval) {
        sched_mlfq_boost(sched);
    }
    int sel = sched_mlfq_select(sched);
    if (sel >= 0) {
        sched->tasks[sel].total_runtime++;
        sched->tasks[sel].allotment_remaining--;
        if (sched->tasks[sel].allotment_remaining == 0) {
            if (sched->tasks[sel].current_level < MLFQ_LEVELS - 1) {
                sched->tasks[sel].current_level++;
            }
            sched->tasks[sel].allotment_remaining =
                sched->level_allotment[sched->tasks[sel].current_level];
        }
    }
    sched->current_task = sel;
    return sel;
}

int sched_mlfq_test(void) {
    sched_mlfq_scheduler_t sched;
    sched_mlfq_init(&sched, 100);
    sched_mlfq_add(&sched, 1);
    sched_mlfq_add(&sched, 2);
    int sel = sched_mlfq_tick(&sched);
    if (sel < 0) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C865: MLFQ should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C865: Output should not be empty");
    assert!(code.contains("fn sched_mlfq_init"), "C865: Should contain sched_mlfq_init");
    assert!(code.contains("fn sched_mlfq_tick"), "C865: Should contain sched_mlfq_tick");
}

// ============================================================================
// C866-C870: Advanced Schedulers
// ============================================================================

/// C866: Lottery scheduler with ticket-based allocation
#[test]
fn c866_lottery_scheduler() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_LOT_TASKS 16

typedef struct {
    int task_id;
    int tickets;
    int wins;
    int active;
} sched_lot_task_t;

typedef struct {
    sched_lot_task_t tasks[MAX_LOT_TASKS];
    int task_count;
    int total_tickets;
    int current;
    uint32_t seed;
} sched_lot_scheduler_t;

void sched_lot_init(sched_lot_scheduler_t *sched, uint32_t seed) {
    sched->task_count = 0;
    sched->total_tickets = 0;
    sched->current = -1;
    sched->seed = seed;
}

uint32_t sched_lot_rand(sched_lot_scheduler_t *sched) {
    sched->seed = sched->seed * 1103515245 + 12345;
    return (sched->seed >> 16) & 0x7FFF;
}

int sched_lot_add(sched_lot_scheduler_t *sched, int id, int tickets) {
    if (sched->task_count >= MAX_LOT_TASKS) return -1;
    int idx = sched->task_count;
    sched->tasks[idx].task_id = id;
    sched->tasks[idx].tickets = tickets;
    sched->tasks[idx].wins = 0;
    sched->tasks[idx].active = 1;
    sched->total_tickets += tickets;
    sched->task_count++;
    return idx;
}

int sched_lot_draw(sched_lot_scheduler_t *sched) {
    if (sched->total_tickets == 0) return -1;
    uint32_t winner = sched_lot_rand(sched) % (uint32_t)sched->total_tickets;
    uint32_t counter = 0;
    int i;
    for (i = 0; i < sched->task_count; i++) {
        if (!sched->tasks[i].active) continue;
        counter += (uint32_t)sched->tasks[i].tickets;
        if (counter > winner) {
            sched->tasks[i].wins++;
            sched->current = i;
            return i;
        }
    }
    return -1;
}

int sched_lot_test(void) {
    sched_lot_scheduler_t sched;
    sched_lot_init(&sched, 42);
    sched_lot_add(&sched, 1, 100);
    sched_lot_add(&sched, 2, 50);
    int w = sched_lot_draw(&sched);
    if (w < 0) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C866: Lottery scheduler should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C866: Output should not be empty");
    assert!(code.contains("fn sched_lot_init"), "C866: Should contain sched_lot_init");
    assert!(code.contains("fn sched_lot_draw"), "C866: Should contain sched_lot_draw");
}

/// C867: Stride scheduler with pass/stride values
#[test]
fn c867_stride_scheduler() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_STRIDE_TASKS 16
#define STRIDE_LARGE 100000

typedef struct {
    int task_id;
    int tickets;
    uint32_t stride;
    uint32_t pass;
    uint32_t runtime;
    int active;
} sched_stride_task_t;

typedef struct {
    sched_stride_task_t tasks[MAX_STRIDE_TASKS];
    int task_count;
    int current;
    uint32_t global_pass;
} sched_stride_scheduler_t;

void sched_stride_init(sched_stride_scheduler_t *sched) {
    sched->task_count = 0;
    sched->current = -1;
    sched->global_pass = 0;
}

int sched_stride_add(sched_stride_scheduler_t *sched, int id, int tickets) {
    if (sched->task_count >= MAX_STRIDE_TASKS || tickets <= 0) return -1;
    int idx = sched->task_count;
    sched->tasks[idx].task_id = id;
    sched->tasks[idx].tickets = tickets;
    sched->tasks[idx].stride = STRIDE_LARGE / (uint32_t)tickets;
    sched->tasks[idx].pass = sched->global_pass;
    sched->tasks[idx].runtime = 0;
    sched->tasks[idx].active = 1;
    sched->task_count++;
    return idx;
}

int sched_stride_select(sched_stride_scheduler_t *sched) {
    int best = -1;
    uint32_t min_pass = 0xFFFFFFFF;
    int i;
    for (i = 0; i < sched->task_count; i++) {
        if (sched->tasks[i].active && sched->tasks[i].pass < min_pass) {
            best = i;
            min_pass = sched->tasks[i].pass;
        }
    }
    return best;
}

int sched_stride_tick(sched_stride_scheduler_t *sched) {
    int sel = sched_stride_select(sched);
    if (sel < 0) return -1;
    sched->tasks[sel].pass += sched->tasks[sel].stride;
    sched->tasks[sel].runtime++;
    sched->current = sel;
    sched->global_pass++;
    return sel;
}

int sched_stride_test(void) {
    sched_stride_scheduler_t sched;
    sched_stride_init(&sched);
    sched_stride_add(&sched, 1, 100);
    sched_stride_add(&sched, 2, 50);
    int s = sched_stride_tick(&sched);
    if (s < 0) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C867: Stride scheduler should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C867: Output should not be empty");
    assert!(code.contains("fn sched_stride_init"), "C867: Should contain sched_stride_init");
    assert!(code.contains("fn sched_stride_tick"), "C867: Should contain sched_stride_tick");
}

/// C868: Gang scheduler with CPU affinity
#[test]
fn c868_gang_scheduler() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_GANG_TASKS 16
#define MAX_CPUS 4

typedef struct {
    int task_id;
    int group_id;
    int cpu_affinity;
    int state;
    uint32_t runtime;
} sched_gang_task_t;

typedef struct {
    sched_gang_task_t tasks[MAX_GANG_TASKS];
    int task_count;
    int cpu_assignment[MAX_CPUS];
    int active_group;
    uint32_t tick;
} sched_gang_scheduler_t;

void sched_gang_init(sched_gang_scheduler_t *sched) {
    sched->task_count = 0;
    sched->active_group = -1;
    sched->tick = 0;
    int i;
    for (i = 0; i < MAX_CPUS; i++) {
        sched->cpu_assignment[i] = -1;
    }
}

int sched_gang_add(sched_gang_scheduler_t *sched, int id, int group, int cpu) {
    if (sched->task_count >= MAX_GANG_TASKS) return -1;
    if (cpu < 0 || cpu >= MAX_CPUS) return -1;
    int idx = sched->task_count;
    sched->tasks[idx].task_id = id;
    sched->tasks[idx].group_id = group;
    sched->tasks[idx].cpu_affinity = cpu;
    sched->tasks[idx].state = 0;
    sched->tasks[idx].runtime = 0;
    sched->task_count++;
    return idx;
}

int sched_gang_schedule_group(sched_gang_scheduler_t *sched, int group) {
    int i;
    for (i = 0; i < MAX_CPUS; i++) {
        sched->cpu_assignment[i] = -1;
    }
    int scheduled = 0;
    for (i = 0; i < sched->task_count; i++) {
        if (sched->tasks[i].group_id == group) {
            int cpu = sched->tasks[i].cpu_affinity;
            if (sched->cpu_assignment[cpu] == -1) {
                sched->cpu_assignment[cpu] = i;
                sched->tasks[i].state = 1;
                scheduled++;
            }
        }
    }
    sched->active_group = group;
    return scheduled;
}

int sched_gang_tick(sched_gang_scheduler_t *sched) {
    sched->tick++;
    int active = 0;
    int i;
    for (i = 0; i < MAX_CPUS; i++) {
        if (sched->cpu_assignment[i] >= 0) {
            sched->tasks[sched->cpu_assignment[i]].runtime++;
            active++;
        }
    }
    return active;
}

int sched_gang_test(void) {
    sched_gang_scheduler_t sched;
    sched_gang_init(&sched);
    sched_gang_add(&sched, 1, 0, 0);
    sched_gang_add(&sched, 2, 0, 1);
    sched_gang_add(&sched, 3, 1, 0);
    int n = sched_gang_schedule_group(&sched, 0);
    if (n != 2) return -1;
    sched_gang_tick(&sched);
    if (sched.tasks[0].runtime != 1) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C868: Gang scheduler should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C868: Output should not be empty");
    assert!(code.contains("fn sched_gang_init"), "C868: Should contain sched_gang_init");
    assert!(code.contains("fn sched_gang_schedule_group"), "C868: Should contain sched_gang_schedule_group");
}

/// C869: Real-time clock with tick counter
#[test]
fn c869_realtime_clock() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

typedef struct {
    uint64_t ticks;
    uint32_t ticks_per_second;
    uint32_t seconds;
    uint32_t minutes;
    uint32_t hours;
    uint32_t days;
    uint32_t sub_tick;
} sched_rtc_t;

void sched_rtc_init(sched_rtc_t *rtc, uint32_t ticks_per_sec) {
    rtc->ticks = 0;
    rtc->ticks_per_second = ticks_per_sec;
    rtc->seconds = 0;
    rtc->minutes = 0;
    rtc->hours = 0;
    rtc->days = 0;
    rtc->sub_tick = 0;
}

void sched_rtc_tick(sched_rtc_t *rtc) {
    rtc->ticks++;
    rtc->sub_tick++;
    if (rtc->sub_tick >= rtc->ticks_per_second) {
        rtc->sub_tick = 0;
        rtc->seconds++;
        if (rtc->seconds >= 60) {
            rtc->seconds = 0;
            rtc->minutes++;
            if (rtc->minutes >= 60) {
                rtc->minutes = 0;
                rtc->hours++;
                if (rtc->hours >= 24) {
                    rtc->hours = 0;
                    rtc->days++;
                }
            }
        }
    }
}

uint32_t sched_rtc_elapsed_ms(sched_rtc_t *rtc) {
    uint32_t total_secs = rtc->days * 86400 + rtc->hours * 3600 +
                          rtc->minutes * 60 + rtc->seconds;
    uint32_t ms = total_secs * 1000;
    if (rtc->ticks_per_second > 0) {
        ms += (rtc->sub_tick * 1000) / rtc->ticks_per_second;
    }
    return ms;
}

int sched_rtc_test(void) {
    sched_rtc_t rtc;
    sched_rtc_init(&rtc, 1000);
    int i;
    for (i = 0; i < 1000; i++) sched_rtc_tick(&rtc);
    if (rtc.seconds != 1) return -1;
    if (rtc.sub_tick != 0) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C869: Real-time clock should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C869: Output should not be empty");
    assert!(code.contains("fn sched_rtc_init"), "C869: Should contain sched_rtc_init");
    assert!(code.contains("fn sched_rtc_tick"), "C869: Should contain sched_rtc_tick");
}

/// C870: Interval timer manager with multiple timers
#[test]
fn c870_interval_timer_manager() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_ITIMERS 16

typedef struct {
    int timer_id;
    uint32_t interval;
    uint32_t remaining;
    int repeating;
    int active;
    int fire_count;
} sched_itimer_t;

typedef struct {
    sched_itimer_t timers[MAX_ITIMERS];
    int timer_count;
    uint32_t tick;
    int total_fires;
} sched_itimer_mgr_t;

void sched_itimer_init(sched_itimer_mgr_t *mgr) {
    mgr->timer_count = 0;
    mgr->tick = 0;
    mgr->total_fires = 0;
}

int sched_itimer_create(sched_itimer_mgr_t *mgr, int id, uint32_t interval, int repeating) {
    if (mgr->timer_count >= MAX_ITIMERS) return -1;
    int idx = mgr->timer_count;
    mgr->timers[idx].timer_id = id;
    mgr->timers[idx].interval = interval;
    mgr->timers[idx].remaining = interval;
    mgr->timers[idx].repeating = repeating;
    mgr->timers[idx].active = 1;
    mgr->timers[idx].fire_count = 0;
    mgr->timer_count++;
    return idx;
}

int sched_itimer_stop(sched_itimer_mgr_t *mgr, int idx) {
    if (idx < 0 || idx >= mgr->timer_count) return -1;
    mgr->timers[idx].active = 0;
    return 0;
}

int sched_itimer_tick(sched_itimer_mgr_t *mgr) {
    mgr->tick++;
    int fired = 0;
    int i;
    for (i = 0; i < mgr->timer_count; i++) {
        if (!mgr->timers[i].active) continue;
        mgr->timers[i].remaining--;
        if (mgr->timers[i].remaining == 0) {
            mgr->timers[i].fire_count++;
            mgr->total_fires++;
            fired++;
            if (mgr->timers[i].repeating) {
                mgr->timers[i].remaining = mgr->timers[i].interval;
            } else {
                mgr->timers[i].active = 0;
            }
        }
    }
    return fired;
}

int sched_itimer_test(void) {
    sched_itimer_mgr_t mgr;
    sched_itimer_init(&mgr);
    sched_itimer_create(&mgr, 1, 5, 1);
    sched_itimer_create(&mgr, 2, 10, 0);
    int i;
    for (i = 0; i < 10; i++) sched_itimer_tick(&mgr);
    if (mgr.timers[0].fire_count != 2) return -1;
    if (mgr.timers[1].fire_count != 1) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C870: Interval timer manager should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C870: Output should not be empty");
    assert!(code.contains("fn sched_itimer_init"), "C870: Should contain sched_itimer_init");
    assert!(code.contains("fn sched_itimer_tick"), "C870: Should contain sched_itimer_tick");
}

// ============================================================================
// C871-C875: System Monitoring
// ============================================================================

/// C871: CPU load monitor with sliding window
#[test]
fn c871_cpu_load_monitor() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define LOAD_WINDOW 16

typedef struct {
    uint32_t samples[LOAD_WINDOW];
    int head;
    int count;
    uint32_t total_idle;
    uint32_t total_busy;
    uint32_t tick;
} sched_cpuload_t;

void sched_cpuload_init(sched_cpuload_t *mon) {
    int i;
    for (i = 0; i < LOAD_WINDOW; i++) {
        mon->samples[i] = 0;
    }
    mon->head = 0;
    mon->count = 0;
    mon->total_idle = 0;
    mon->total_busy = 0;
    mon->tick = 0;
}

void sched_cpuload_record(sched_cpuload_t *mon, uint32_t busy_pct) {
    if (busy_pct > 100) busy_pct = 100;
    mon->samples[mon->head] = busy_pct;
    mon->head = (mon->head + 1) % LOAD_WINDOW;
    if (mon->count < LOAD_WINDOW) mon->count++;
    mon->total_busy += busy_pct;
    mon->total_idle += (100 - busy_pct);
    mon->tick++;
}

uint32_t sched_cpuload_average(sched_cpuload_t *mon) {
    if (mon->count == 0) return 0;
    uint32_t sum = 0;
    int i;
    for (i = 0; i < mon->count; i++) {
        sum += mon->samples[i];
    }
    return sum / (uint32_t)mon->count;
}

uint32_t sched_cpuload_peak(sched_cpuload_t *mon) {
    uint32_t peak = 0;
    int i;
    for (i = 0; i < mon->count; i++) {
        if (mon->samples[i] > peak) peak = mon->samples[i];
    }
    return peak;
}

int sched_cpuload_is_overloaded(sched_cpuload_t *mon, uint32_t threshold) {
    return sched_cpuload_average(mon) > threshold;
}

int sched_cpuload_test(void) {
    sched_cpuload_t mon;
    sched_cpuload_init(&mon);
    sched_cpuload_record(&mon, 50);
    sched_cpuload_record(&mon, 80);
    sched_cpuload_record(&mon, 70);
    uint32_t avg = sched_cpuload_average(&mon);
    if (avg != 66) return -1;
    if (sched_cpuload_peak(&mon) != 80) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C871: CPU load monitor should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C871: Output should not be empty");
    assert!(code.contains("fn sched_cpuload_init"), "C871: Should contain sched_cpuload_init");
    assert!(code.contains("fn sched_cpuload_average"), "C871: Should contain sched_cpuload_average");
}

/// C872: Stack overflow detector with canary and watermark
#[test]
fn c872_stack_overflow_detector() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define MAX_STACK_TASKS 8
#define STACK_SIZE 256
#define CANARY_VALUE 0xDEADBEEF

typedef struct {
    int task_id;
    uint32_t stack[STACK_SIZE];
    uint32_t stack_size;
    uint32_t watermark;
    uint32_t canary;
    int overflow_detected;
} sched_stackmon_task_t;

typedef struct {
    sched_stackmon_task_t tasks[MAX_STACK_TASKS];
    int task_count;
    int overflow_count;
} sched_stackmon_t;

void sched_stackmon_init(sched_stackmon_t *mon) {
    mon->task_count = 0;
    mon->overflow_count = 0;
}

int sched_stackmon_add(sched_stackmon_t *mon, int id) {
    if (mon->task_count >= MAX_STACK_TASKS) return -1;
    int idx = mon->task_count;
    mon->tasks[idx].task_id = id;
    mon->tasks[idx].stack_size = STACK_SIZE;
    mon->tasks[idx].watermark = STACK_SIZE;
    mon->tasks[idx].canary = CANARY_VALUE;
    mon->tasks[idx].overflow_detected = 0;
    int i;
    for (i = 0; i < STACK_SIZE; i++) {
        mon->tasks[idx].stack[i] = CANARY_VALUE;
    }
    mon->task_count++;
    return idx;
}

int sched_stackmon_check_canary(sched_stackmon_t *mon, int idx) {
    if (idx < 0 || idx >= mon->task_count) return -1;
    if (mon->tasks[idx].canary != CANARY_VALUE) {
        mon->tasks[idx].overflow_detected = 1;
        mon->overflow_count++;
        return 1;
    }
    return 0;
}

uint32_t sched_stackmon_compute_watermark(sched_stackmon_t *mon, int idx) {
    if (idx < 0 || idx >= mon->task_count) return 0;
    uint32_t used = 0;
    int i;
    for (i = 0; i < (int)mon->tasks[idx].stack_size; i++) {
        if (mon->tasks[idx].stack[i] != CANARY_VALUE) {
            used = (uint32_t)(mon->tasks[idx].stack_size - (uint32_t)i);
            break;
        }
    }
    mon->tasks[idx].watermark = mon->tasks[idx].stack_size - used;
    return used;
}

int sched_stackmon_test(void) {
    sched_stackmon_t mon;
    sched_stackmon_init(&mon);
    int t = sched_stackmon_add(&mon, 1);
    if (sched_stackmon_check_canary(&mon, t) != 0) return -1;
    mon.tasks[t].stack[0] = 0x12345678;
    uint32_t used = sched_stackmon_compute_watermark(&mon, t);
    if (used != STACK_SIZE) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C872: Stack overflow detector should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C872: Output should not be empty");
    assert!(code.contains("fn sched_stackmon_init"), "C872: Should contain sched_stackmon_init");
    assert!(code.contains("fn sched_stackmon_check_canary"), "C872: Should contain sched_stackmon_check_canary");
}

/// C873: Power management with sleep states
#[test]
fn c873_power_management() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define PM_ACTIVE  0
#define PM_IDLE    1
#define PM_SLEEP   2
#define PM_DEEP    3
#define PM_STATES  4

typedef struct {
    int current_state;
    uint32_t time_in_state[PM_STATES];
    uint32_t transition_count;
    uint32_t idle_threshold;
    uint32_t sleep_threshold;
    uint32_t deep_threshold;
    uint32_t idle_ticks;
    uint32_t tick;
} sched_pm_t;

void sched_pm_init(sched_pm_t *pm, uint32_t idle_th, uint32_t sleep_th, uint32_t deep_th) {
    pm->current_state = PM_ACTIVE;
    pm->transition_count = 0;
    pm->idle_threshold = idle_th;
    pm->sleep_threshold = sleep_th;
    pm->deep_threshold = deep_th;
    pm->idle_ticks = 0;
    pm->tick = 0;
    int i;
    for (i = 0; i < PM_STATES; i++) {
        pm->time_in_state[i] = 0;
    }
}

void sched_pm_set_state(sched_pm_t *pm, int new_state) {
    if (new_state != pm->current_state && new_state >= 0 && new_state < PM_STATES) {
        pm->current_state = new_state;
        pm->transition_count++;
    }
}

void sched_pm_activity(sched_pm_t *pm) {
    pm->idle_ticks = 0;
    sched_pm_set_state(pm, PM_ACTIVE);
}

void sched_pm_tick(sched_pm_t *pm) {
    pm->tick++;
    pm->time_in_state[pm->current_state]++;
    pm->idle_ticks++;
    if (pm->idle_ticks >= pm->deep_threshold) {
        sched_pm_set_state(pm, PM_DEEP);
    } else if (pm->idle_ticks >= pm->sleep_threshold) {
        sched_pm_set_state(pm, PM_SLEEP);
    } else if (pm->idle_ticks >= pm->idle_threshold) {
        sched_pm_set_state(pm, PM_IDLE);
    }
}

int sched_pm_get_power_level(sched_pm_t *pm) {
    if (pm->current_state == PM_ACTIVE) return 100;
    if (pm->current_state == PM_IDLE) return 50;
    if (pm->current_state == PM_SLEEP) return 10;
    return 1;
}

int sched_pm_test(void) {
    sched_pm_t pm;
    sched_pm_init(&pm, 5, 20, 50);
    if (pm.current_state != PM_ACTIVE) return -1;
    int i;
    for (i = 0; i < 6; i++) sched_pm_tick(&pm);
    if (pm.current_state != PM_IDLE) return -2;
    sched_pm_activity(&pm);
    if (pm.current_state != PM_ACTIVE) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C873: Power management should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C873: Output should not be empty");
    assert!(code.contains("fn sched_pm_init"), "C873: Should contain sched_pm_init");
    assert!(code.contains("fn sched_pm_tick"), "C873: Should contain sched_pm_tick");
}

/// C874: Interrupt priority controller with nesting
#[test]
fn c874_interrupt_priority_controller() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_IRQS 32
#define MAX_NEST_DEPTH 8

typedef struct {
    int irq_number;
    int priority;
    int enabled;
    int pending;
    uint32_t trigger_count;
} sched_irq_entry_t;

typedef struct {
    sched_irq_entry_t irqs[MAX_IRQS];
    int irq_count;
    int nest_stack[MAX_NEST_DEPTH];
    int nest_depth;
    int current_priority;
    int global_enable;
    uint32_t total_interrupts;
} sched_irqctrl_t;

void sched_irqctrl_init(sched_irqctrl_t *ctrl) {
    ctrl->irq_count = 0;
    ctrl->nest_depth = 0;
    ctrl->current_priority = -1;
    ctrl->global_enable = 1;
    ctrl->total_interrupts = 0;
}

int sched_irqctrl_register(sched_irqctrl_t *ctrl, int irq, int priority) {
    if (ctrl->irq_count >= MAX_IRQS) return -1;
    int idx = ctrl->irq_count;
    ctrl->irqs[idx].irq_number = irq;
    ctrl->irqs[idx].priority = priority;
    ctrl->irqs[idx].enabled = 1;
    ctrl->irqs[idx].pending = 0;
    ctrl->irqs[idx].trigger_count = 0;
    ctrl->irq_count++;
    return idx;
}

int sched_irqctrl_trigger(sched_irqctrl_t *ctrl, int idx) {
    if (idx < 0 || idx >= ctrl->irq_count) return -1;
    if (!ctrl->global_enable || !ctrl->irqs[idx].enabled) return -2;
    if (ctrl->irqs[idx].priority <= ctrl->current_priority) {
        ctrl->irqs[idx].pending = 1;
        return 0;
    }
    if (ctrl->nest_depth >= MAX_NEST_DEPTH) return -3;
    ctrl->nest_stack[ctrl->nest_depth] = ctrl->current_priority;
    ctrl->nest_depth++;
    ctrl->current_priority = ctrl->irqs[idx].priority;
    ctrl->irqs[idx].trigger_count++;
    ctrl->total_interrupts++;
    return 1;
}

int sched_irqctrl_eoi(sched_irqctrl_t *ctrl) {
    if (ctrl->nest_depth <= 0) return -1;
    ctrl->nest_depth--;
    ctrl->current_priority = ctrl->nest_stack[ctrl->nest_depth];
    return ctrl->nest_depth;
}

int sched_irqctrl_test(void) {
    sched_irqctrl_t ctrl;
    sched_irqctrl_init(&ctrl);
    int lo = sched_irqctrl_register(&ctrl, 0, 1);
    int hi = sched_irqctrl_register(&ctrl, 1, 5);
    int r = sched_irqctrl_trigger(&ctrl, lo);
    if (r != 1) return -1;
    r = sched_irqctrl_trigger(&ctrl, hi);
    if (r != 1) return -2;
    if (ctrl.nest_depth != 2) return -3;
    sched_irqctrl_eoi(&ctrl);
    if (ctrl.nest_depth != 1) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C874: Interrupt priority controller should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C874: Output should not be empty");
    assert!(code.contains("fn sched_irqctrl_init"), "C874: Should contain sched_irqctrl_init");
    assert!(code.contains("fn sched_irqctrl_trigger"), "C874: Should contain sched_irqctrl_trigger");
}

/// C875: DMA transfer scheduler with channel management
#[test]
fn c875_dma_transfer_scheduler() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_DMA_CHANNELS 8
#define MAX_DMA_QUEUE 16

#define DMA_IDLE     0
#define DMA_ACTIVE   1
#define DMA_COMPLETE 2
#define DMA_ERROR    3

typedef struct {
    uint32_t src_addr;
    uint32_t dst_addr;
    uint32_t length;
    int priority;
    int channel;
    int status;
} sched_dma_request_t;

typedef struct {
    int channel_id;
    int busy;
    uint32_t bytes_transferred;
    int current_request;
} sched_dma_channel_t;

typedef struct {
    sched_dma_channel_t channels[MAX_DMA_CHANNELS];
    int channel_count;
    sched_dma_request_t queue[MAX_DMA_QUEUE];
    int queue_count;
    uint32_t total_transfers;
    uint32_t total_bytes;
} sched_dma_t;

void sched_dma_init(sched_dma_t *dma, int num_channels) {
    if (num_channels > MAX_DMA_CHANNELS) num_channels = MAX_DMA_CHANNELS;
    dma->channel_count = num_channels;
    dma->queue_count = 0;
    dma->total_transfers = 0;
    dma->total_bytes = 0;
    int i;
    for (i = 0; i < num_channels; i++) {
        dma->channels[i].channel_id = i;
        dma->channels[i].busy = 0;
        dma->channels[i].bytes_transferred = 0;
        dma->channels[i].current_request = -1;
    }
}

int sched_dma_submit(sched_dma_t *dma, uint32_t src, uint32_t dst,
                     uint32_t len, int prio) {
    if (dma->queue_count >= MAX_DMA_QUEUE) return -1;
    int idx = dma->queue_count;
    dma->queue[idx].src_addr = src;
    dma->queue[idx].dst_addr = dst;
    dma->queue[idx].length = len;
    dma->queue[idx].priority = prio;
    dma->queue[idx].channel = -1;
    dma->queue[idx].status = DMA_IDLE;
    dma->queue_count++;
    return idx;
}

int sched_dma_find_free_channel(sched_dma_t *dma) {
    int i;
    for (i = 0; i < dma->channel_count; i++) {
        if (!dma->channels[i].busy) return i;
    }
    return -1;
}

int sched_dma_dispatch(sched_dma_t *dma) {
    int dispatched = 0;
    int i;
    for (i = 0; i < dma->queue_count; i++) {
        if (dma->queue[i].status != DMA_IDLE) continue;
        int ch = sched_dma_find_free_channel(dma);
        if (ch < 0) break;
        dma->queue[i].channel = ch;
        dma->queue[i].status = DMA_ACTIVE;
        dma->channels[ch].busy = 1;
        dma->channels[ch].current_request = i;
        dispatched++;
    }
    return dispatched;
}

int sched_dma_complete(sched_dma_t *dma, int channel) {
    if (channel < 0 || channel >= dma->channel_count) return -1;
    if (!dma->channels[channel].busy) return -2;
    int req = dma->channels[channel].current_request;
    if (req >= 0) {
        dma->queue[req].status = DMA_COMPLETE;
        dma->channels[channel].bytes_transferred += dma->queue[req].length;
        dma->total_bytes += dma->queue[req].length;
        dma->total_transfers++;
    }
    dma->channels[channel].busy = 0;
    dma->channels[channel].current_request = -1;
    return 0;
}

int sched_dma_test(void) {
    sched_dma_t dma;
    sched_dma_init(&dma, 2);
    sched_dma_submit(&dma, 0x1000, 0x2000, 512, 1);
    sched_dma_submit(&dma, 0x3000, 0x4000, 1024, 2);
    sched_dma_submit(&dma, 0x5000, 0x6000, 256, 0);
    int d = sched_dma_dispatch(&dma);
    if (d != 2) return -1;
    sched_dma_complete(&dma, 0);
    if (dma.total_transfers != 1) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C875: DMA transfer scheduler should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C875: Output should not be empty");
    assert!(code.contains("fn sched_dma_init"), "C875: Should contain sched_dma_init");
    assert!(code.contains("fn sched_dma_dispatch"), "C875: Should contain sched_dma_dispatch");
}
