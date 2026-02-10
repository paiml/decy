//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1126-C1150: State Machine Implementations -- the kind of C code found
//! in embedded controllers, protocol stacks, game engines, parsers,
//! industrial automation, and UI frameworks.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise finite state machines, parser automata, protocol
//! state machines, game/UI controllers, and industrial control systems --
//! all expressed as valid C99 with no includes, using the fsm_ prefix
//! for all function and type names.
//!
//! Organization:
//! - C1126-C1130: Classic FSMs (traffic light, vending machine, elevator, door lock, turnstile)
//! - C1131-C1135: Parser state machines (JSON tokenizer, CSV parser, XML tag, INI parser, ANSI escape)
//! - C1136-C1140: Protocol state machines (TCP, HTTP/1.1, DHCP client, TLS handshake, PPP link)
//! - C1141-C1145: Game/UI state machines (game menu, animation controller, dialog tree, combo detector, screen manager)
//! - C1146-C1150: Industrial state machines (PID controller, motor driver, battery charger, thermostat, conveyor belt)

// ============================================================================
// C1126-C1130: Classic FSMs
// ============================================================================

/// C1126: Traffic light controller with timed phase transitions
#[test]
fn c1126_traffic_light_controller() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef enum {
    FSM_TL_RED,
    FSM_TL_RED_YELLOW,
    FSM_TL_GREEN,
    FSM_TL_YELLOW
} fsm_tl_state_t;

typedef struct {
    fsm_tl_state_t state;
    int timer;
    int red_duration;
    int green_duration;
    int yellow_duration;
    int cycle_count;
} fsm_tl_context_t;

void fsm_tl_init(fsm_tl_context_t *ctx, int red_dur, int green_dur, int yellow_dur) {
    ctx->state = FSM_TL_RED;
    ctx->timer = 0;
    ctx->red_duration = red_dur;
    ctx->green_duration = green_dur;
    ctx->yellow_duration = yellow_dur;
    ctx->cycle_count = 0;
}

void fsm_tl_tick(fsm_tl_context_t *ctx) {
    ctx->timer++;
    switch (ctx->state) {
        case FSM_TL_RED:
            if (ctx->timer >= ctx->red_duration) {
                ctx->state = FSM_TL_RED_YELLOW;
                ctx->timer = 0;
            }
            break;
        case FSM_TL_RED_YELLOW:
            if (ctx->timer >= ctx->yellow_duration) {
                ctx->state = FSM_TL_GREEN;
                ctx->timer = 0;
            }
            break;
        case FSM_TL_GREEN:
            if (ctx->timer >= ctx->green_duration) {
                ctx->state = FSM_TL_YELLOW;
                ctx->timer = 0;
            }
            break;
        case FSM_TL_YELLOW:
            if (ctx->timer >= ctx->yellow_duration) {
                ctx->state = FSM_TL_RED;
                ctx->timer = 0;
                ctx->cycle_count++;
            }
            break;
    }
}

int fsm_tl_test(void) {
    fsm_tl_context_t ctx;
    fsm_tl_init(&ctx, 10, 8, 3);
    int i;
    for (i = 0; i < 50; i++) {
        fsm_tl_tick(&ctx);
    }
    if (ctx.cycle_count < 1) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1126: traffic light controller should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1126: empty output");
    assert!(code.contains("fn fsm_tl_init"), "C1126: Should contain fsm_tl_init");
    assert!(code.contains("fn fsm_tl_tick"), "C1126: Should contain fsm_tl_tick");
    Ok(())
}

/// C1127: Vending machine with coin acceptance and product dispensing
#[test]
fn c1127_vending_machine() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef enum {
    FSM_VM_IDLE,
    FSM_VM_COLLECTING,
    FSM_VM_DISPENSING,
    FSM_VM_CHANGE,
    FSM_VM_ERROR
} fsm_vm_state_t;

typedef struct {
    fsm_vm_state_t state;
    int balance;
    int prices[8];
    int stock[8];
    int num_products;
    int selected;
    int change_due;
} fsm_vm_context_t;

void fsm_vm_init(fsm_vm_context_t *ctx) {
    ctx->state = FSM_VM_IDLE;
    ctx->balance = 0;
    ctx->num_products = 4;
    ctx->selected = -1;
    ctx->change_due = 0;
    int i;
    for (i = 0; i < 8; i++) {
        ctx->prices[i] = (i + 1) * 50;
        ctx->stock[i] = 5;
    }
}

void fsm_vm_insert_coin(fsm_vm_context_t *ctx, int amount) {
    if (ctx->state == FSM_VM_IDLE) {
        ctx->state = FSM_VM_COLLECTING;
    }
    if (ctx->state == FSM_VM_COLLECTING) {
        ctx->balance += amount;
    }
}

int fsm_vm_select(fsm_vm_context_t *ctx, int product) {
    if (ctx->state != FSM_VM_COLLECTING) return -1;
    if (product < 0 || product >= ctx->num_products) return -2;
    if (ctx->stock[product] <= 0) return -3;
    if (ctx->balance < ctx->prices[product]) return -4;
    ctx->selected = product;
    ctx->change_due = ctx->balance - ctx->prices[product];
    ctx->stock[product]--;
    ctx->state = FSM_VM_DISPENSING;
    return 0;
}

void fsm_vm_complete(fsm_vm_context_t *ctx) {
    if (ctx->state == FSM_VM_DISPENSING) {
        if (ctx->change_due > 0) {
            ctx->state = FSM_VM_CHANGE;
        } else {
            ctx->state = FSM_VM_IDLE;
            ctx->balance = 0;
            ctx->selected = -1;
        }
    } else if (ctx->state == FSM_VM_CHANGE) {
        ctx->state = FSM_VM_IDLE;
        ctx->balance = 0;
        ctx->selected = -1;
        ctx->change_due = 0;
    }
}

int fsm_vm_test(void) {
    fsm_vm_context_t ctx;
    fsm_vm_init(&ctx);
    fsm_vm_insert_coin(&ctx, 100);
    fsm_vm_insert_coin(&ctx, 50);
    int r = fsm_vm_select(&ctx, 1);
    if (r != 0) return -1;
    fsm_vm_complete(&ctx);
    fsm_vm_complete(&ctx);
    if (ctx.state != FSM_VM_IDLE) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1127: vending machine should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1127: empty output");
    assert!(code.contains("fn fsm_vm_init"), "C1127: Should contain fsm_vm_init");
    assert!(code.contains("fn fsm_vm_select"), "C1127: Should contain fsm_vm_select");
    Ok(())
}

/// C1128: Elevator controller with floor requests and door management
#[test]
fn c1128_elevator_controller() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef enum {
    FSM_EL_IDLE,
    FSM_EL_MOVING_UP,
    FSM_EL_MOVING_DOWN,
    FSM_EL_DOOR_OPENING,
    FSM_EL_DOOR_OPEN,
    FSM_EL_DOOR_CLOSING
} fsm_el_state_t;

typedef struct {
    fsm_el_state_t state;
    int current_floor;
    int target_floor;
    int requests[16];
    int num_floors;
    int door_timer;
    int door_open_time;
} fsm_el_context_t;

void fsm_el_init(fsm_el_context_t *ctx, int floors) {
    ctx->state = FSM_EL_IDLE;
    ctx->current_floor = 0;
    ctx->target_floor = 0;
    ctx->num_floors = floors;
    ctx->door_timer = 0;
    ctx->door_open_time = 5;
    int i;
    for (i = 0; i < 16; i++) {
        ctx->requests[i] = 0;
    }
}

void fsm_el_request(fsm_el_context_t *ctx, int floor) {
    if (floor >= 0 && floor < ctx->num_floors) {
        ctx->requests[floor] = 1;
    }
}

int fsm_el_find_next(const fsm_el_context_t *ctx) {
    int i;
    int closest = -1;
    int min_dist = 9999;
    for (i = 0; i < ctx->num_floors; i++) {
        if (ctx->requests[i]) {
            int dist = i - ctx->current_floor;
            if (dist < 0) dist = -dist;
            if (dist < min_dist) {
                min_dist = dist;
                closest = i;
            }
        }
    }
    return closest;
}

void fsm_el_step(fsm_el_context_t *ctx) {
    switch (ctx->state) {
        case FSM_EL_IDLE: {
            int next = fsm_el_find_next(ctx);
            if (next >= 0) {
                ctx->target_floor = next;
                if (next > ctx->current_floor)
                    ctx->state = FSM_EL_MOVING_UP;
                else if (next < ctx->current_floor)
                    ctx->state = FSM_EL_MOVING_DOWN;
                else
                    ctx->state = FSM_EL_DOOR_OPENING;
            }
            break;
        }
        case FSM_EL_MOVING_UP:
            ctx->current_floor++;
            if (ctx->current_floor == ctx->target_floor)
                ctx->state = FSM_EL_DOOR_OPENING;
            break;
        case FSM_EL_MOVING_DOWN:
            ctx->current_floor--;
            if (ctx->current_floor == ctx->target_floor)
                ctx->state = FSM_EL_DOOR_OPENING;
            break;
        case FSM_EL_DOOR_OPENING:
            ctx->requests[ctx->current_floor] = 0;
            ctx->door_timer = 0;
            ctx->state = FSM_EL_DOOR_OPEN;
            break;
        case FSM_EL_DOOR_OPEN:
            ctx->door_timer++;
            if (ctx->door_timer >= ctx->door_open_time)
                ctx->state = FSM_EL_DOOR_CLOSING;
            break;
        case FSM_EL_DOOR_CLOSING:
            ctx->state = FSM_EL_IDLE;
            break;
    }
}

int fsm_el_test(void) {
    fsm_el_context_t ctx;
    fsm_el_init(&ctx, 10);
    fsm_el_request(&ctx, 5);
    int i;
    for (i = 0; i < 20; i++) {
        fsm_el_step(&ctx);
    }
    if (ctx.current_floor != 5) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1128: elevator controller should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1128: empty output");
    assert!(code.contains("fn fsm_el_init"), "C1128: Should contain fsm_el_init");
    assert!(code.contains("fn fsm_el_step"), "C1128: Should contain fsm_el_step");
    Ok(())
}

/// C1129: Door lock FSM with PIN entry and lockout
#[test]
fn c1129_door_lock_fsm() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef enum {
    FSM_DL_LOCKED,
    FSM_DL_ENTERING,
    FSM_DL_UNLOCKED,
    FSM_DL_LOCKOUT
} fsm_dl_state_t;

typedef struct {
    fsm_dl_state_t state;
    int pin[4];
    int entry[4];
    int entry_pos;
    int attempts;
    int max_attempts;
    int lockout_timer;
    int lockout_duration;
} fsm_dl_context_t;

void fsm_dl_init(fsm_dl_context_t *ctx, int p0, int p1, int p2, int p3) {
    ctx->state = FSM_DL_LOCKED;
    ctx->pin[0] = p0; ctx->pin[1] = p1;
    ctx->pin[2] = p2; ctx->pin[3] = p3;
    ctx->entry_pos = 0;
    ctx->attempts = 0;
    ctx->max_attempts = 3;
    ctx->lockout_timer = 0;
    ctx->lockout_duration = 30;
    int i;
    for (i = 0; i < 4; i++) ctx->entry[i] = -1;
}

void fsm_dl_press(fsm_dl_context_t *ctx, int digit) {
    if (ctx->state == FSM_DL_LOCKED) {
        ctx->state = FSM_DL_ENTERING;
        ctx->entry_pos = 0;
    }
    if (ctx->state == FSM_DL_ENTERING) {
        if (ctx->entry_pos < 4) {
            ctx->entry[ctx->entry_pos] = digit;
            ctx->entry_pos++;
        }
        if (ctx->entry_pos == 4) {
            int correct = 1;
            int i;
            for (i = 0; i < 4; i++) {
                if (ctx->entry[i] != ctx->pin[i]) correct = 0;
            }
            if (correct) {
                ctx->state = FSM_DL_UNLOCKED;
                ctx->attempts = 0;
            } else {
                ctx->attempts++;
                if (ctx->attempts >= ctx->max_attempts) {
                    ctx->state = FSM_DL_LOCKOUT;
                    ctx->lockout_timer = 0;
                } else {
                    ctx->state = FSM_DL_LOCKED;
                }
            }
            ctx->entry_pos = 0;
        }
    }
}

void fsm_dl_lock(fsm_dl_context_t *ctx) {
    if (ctx->state == FSM_DL_UNLOCKED) {
        ctx->state = FSM_DL_LOCKED;
    }
}

void fsm_dl_tick(fsm_dl_context_t *ctx) {
    if (ctx->state == FSM_DL_LOCKOUT) {
        ctx->lockout_timer++;
        if (ctx->lockout_timer >= ctx->lockout_duration) {
            ctx->state = FSM_DL_LOCKED;
            ctx->attempts = 0;
        }
    }
}

int fsm_dl_test(void) {
    fsm_dl_context_t ctx;
    fsm_dl_init(&ctx, 1, 2, 3, 4);
    fsm_dl_press(&ctx, 1);
    fsm_dl_press(&ctx, 2);
    fsm_dl_press(&ctx, 3);
    fsm_dl_press(&ctx, 4);
    if (ctx.state != FSM_DL_UNLOCKED) return -1;
    fsm_dl_lock(&ctx);
    if (ctx.state != FSM_DL_LOCKED) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1129: door lock should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1129: empty output");
    assert!(code.contains("fn fsm_dl_init"), "C1129: Should contain fsm_dl_init");
    assert!(code.contains("fn fsm_dl_press"), "C1129: Should contain fsm_dl_press");
    Ok(())
}

/// C1130: Turnstile FSM with coin and push events plus maintenance mode
#[test]
fn c1130_turnstile_fsm() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef enum {
    FSM_TS_LOCKED,
    FSM_TS_UNLOCKED,
    FSM_TS_BROKEN,
    FSM_TS_MAINTENANCE
} fsm_ts_state_t;

typedef enum {
    FSM_TS_EV_COIN,
    FSM_TS_EV_PUSH,
    FSM_TS_EV_REPAIR,
    FSM_TS_EV_MAINT,
    FSM_TS_EV_DONE
} fsm_ts_event_t;

typedef struct {
    fsm_ts_state_t state;
    int coins_collected;
    int entries_count;
    int push_while_locked;
    int error_count;
} fsm_ts_context_t;

void fsm_ts_init(fsm_ts_context_t *ctx) {
    ctx->state = FSM_TS_LOCKED;
    ctx->coins_collected = 0;
    ctx->entries_count = 0;
    ctx->push_while_locked = 0;
    ctx->error_count = 0;
}

void fsm_ts_handle(fsm_ts_context_t *ctx, fsm_ts_event_t event) {
    switch (ctx->state) {
        case FSM_TS_LOCKED:
            if (event == FSM_TS_EV_COIN) {
                ctx->coins_collected++;
                ctx->state = FSM_TS_UNLOCKED;
            } else if (event == FSM_TS_EV_PUSH) {
                ctx->push_while_locked++;
                if (ctx->push_while_locked > 5) {
                    ctx->state = FSM_TS_BROKEN;
                    ctx->error_count++;
                }
            } else if (event == FSM_TS_EV_MAINT) {
                ctx->state = FSM_TS_MAINTENANCE;
            }
            break;
        case FSM_TS_UNLOCKED:
            if (event == FSM_TS_EV_PUSH) {
                ctx->entries_count++;
                ctx->push_while_locked = 0;
                ctx->state = FSM_TS_LOCKED;
            } else if (event == FSM_TS_EV_COIN) {
                ctx->coins_collected++;
            }
            break;
        case FSM_TS_BROKEN:
            if (event == FSM_TS_EV_REPAIR) {
                ctx->state = FSM_TS_LOCKED;
                ctx->push_while_locked = 0;
            }
            break;
        case FSM_TS_MAINTENANCE:
            if (event == FSM_TS_EV_DONE) {
                ctx->state = FSM_TS_LOCKED;
            }
            break;
    }
}

int fsm_ts_test(void) {
    fsm_ts_context_t ctx;
    fsm_ts_init(&ctx);
    fsm_ts_handle(&ctx, FSM_TS_EV_COIN);
    if (ctx.state != FSM_TS_UNLOCKED) return -1;
    fsm_ts_handle(&ctx, FSM_TS_EV_PUSH);
    if (ctx.state != FSM_TS_LOCKED) return -2;
    if (ctx.entries_count != 1) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1130: turnstile should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1130: empty output");
    assert!(code.contains("fn fsm_ts_init"), "C1130: Should contain fsm_ts_init");
    assert!(code.contains("fn fsm_ts_handle"), "C1130: Should contain fsm_ts_handle");
    Ok(())
}

// ============================================================================
// C1131-C1135: Parser State Machines
// ============================================================================

/// C1131: JSON tokenizer FSM that classifies token types
#[test]
fn c1131_json_tokenizer() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef enum {
    FSM_JT_START,
    FSM_JT_IN_STRING,
    FSM_JT_IN_ESCAPE,
    FSM_JT_IN_NUMBER,
    FSM_JT_ERROR
} fsm_jt_state_t;

typedef enum {
    FSM_JT_TOK_STRING,
    FSM_JT_TOK_NUMBER,
    FSM_JT_TOK_LBRACE,
    FSM_JT_TOK_RBRACE,
    FSM_JT_TOK_COLON,
    FSM_JT_TOK_COMMA,
    FSM_JT_TOK_EOF,
    FSM_JT_TOK_ERROR
} fsm_jt_token_t;

typedef struct {
    fsm_jt_state_t state;
    const char *input;
    int pos;
    int len;
} fsm_jt_context_t;

void fsm_jt_init(fsm_jt_context_t *ctx, const char *input, int len) {
    ctx->state = FSM_JT_START;
    ctx->input = input;
    ctx->pos = 0;
    ctx->len = len;
}

static int fsm_jt_is_digit(char c) {
    return c >= '0' && c <= '9';
}

static int fsm_jt_is_space(char c) {
    return c == ' ' || c == '\t' || c == '\n' || c == '\r';
}

fsm_jt_token_t fsm_jt_next(fsm_jt_context_t *ctx) {
    while (ctx->pos < ctx->len && fsm_jt_is_space(ctx->input[ctx->pos]))
        ctx->pos++;
    if (ctx->pos >= ctx->len) return FSM_JT_TOK_EOF;

    char c = ctx->input[ctx->pos];
    if (c == '{') { ctx->pos++; return FSM_JT_TOK_LBRACE; }
    if (c == '}') { ctx->pos++; return FSM_JT_TOK_RBRACE; }
    if (c == ':') { ctx->pos++; return FSM_JT_TOK_COLON; }
    if (c == ',') { ctx->pos++; return FSM_JT_TOK_COMMA; }

    if (c == '"') {
        ctx->pos++;
        ctx->state = FSM_JT_IN_STRING;
        while (ctx->pos < ctx->len) {
            c = ctx->input[ctx->pos];
            if (c == '\\') {
                ctx->pos += 2;
            } else if (c == '"') {
                ctx->pos++;
                ctx->state = FSM_JT_START;
                return FSM_JT_TOK_STRING;
            } else {
                ctx->pos++;
            }
        }
        return FSM_JT_TOK_ERROR;
    }

    if (fsm_jt_is_digit(c) || c == '-') {
        while (ctx->pos < ctx->len &&
               (fsm_jt_is_digit(ctx->input[ctx->pos]) ||
                ctx->input[ctx->pos] == '.' ||
                ctx->input[ctx->pos] == '-'))
            ctx->pos++;
        return FSM_JT_TOK_NUMBER;
    }

    return FSM_JT_TOK_ERROR;
}

int fsm_jt_test(void) {
    const char *json = "{\"key\": 42}";
    fsm_jt_context_t ctx;
    fsm_jt_init(&ctx, json, 11);
    fsm_jt_token_t t1 = fsm_jt_next(&ctx);
    if (t1 != FSM_JT_TOK_LBRACE) return -1;
    fsm_jt_token_t t2 = fsm_jt_next(&ctx);
    if (t2 != FSM_JT_TOK_STRING) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1131: JSON tokenizer should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1131: empty output");
    assert!(code.contains("fn fsm_jt_init"), "C1131: Should contain fsm_jt_init");
    assert!(code.contains("fn fsm_jt_next"), "C1131: Should contain fsm_jt_next");
    Ok(())
}

/// C1132: CSV parser FSM with field and record detection
#[test]
fn c1132_csv_parser() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef enum {
    FSM_CSV_FIELD_START,
    FSM_CSV_IN_FIELD,
    FSM_CSV_IN_QUOTED,
    FSM_CSV_QUOTE_END,
    FSM_CSV_ERROR
} fsm_csv_state_t;

typedef struct {
    fsm_csv_state_t state;
    int field_count;
    int char_pos;
    int record_count;
} fsm_csv_context_t;

void fsm_csv_init(fsm_csv_context_t *ctx) {
    ctx->state = FSM_CSV_FIELD_START;
    ctx->field_count = 0;
    ctx->char_pos = 0;
    ctx->record_count = 0;
}

void fsm_csv_feed(fsm_csv_context_t *ctx, char c) {
    switch (ctx->state) {
        case FSM_CSV_FIELD_START:
            if (c == '"') {
                ctx->state = FSM_CSV_IN_QUOTED;
                ctx->char_pos = 0;
            } else if (c == ',') {
                ctx->field_count++;
            } else if (c == '\n') {
                ctx->field_count++;
                ctx->record_count++;
                ctx->field_count = 0;
            } else {
                ctx->char_pos = 0;
                ctx->state = FSM_CSV_IN_FIELD;
            }
            break;
        case FSM_CSV_IN_FIELD:
            if (c == ',') {
                ctx->field_count++;
                ctx->state = FSM_CSV_FIELD_START;
                ctx->char_pos = 0;
            } else if (c == '\n') {
                ctx->field_count++;
                ctx->record_count++;
                ctx->field_count = 0;
                ctx->state = FSM_CSV_FIELD_START;
            } else {
                ctx->char_pos++;
            }
            break;
        case FSM_CSV_IN_QUOTED:
            if (c == '"') {
                ctx->state = FSM_CSV_QUOTE_END;
            } else {
                ctx->char_pos++;
            }
            break;
        case FSM_CSV_QUOTE_END:
            if (c == '"') {
                ctx->char_pos++;
                ctx->state = FSM_CSV_IN_QUOTED;
            } else if (c == ',') {
                ctx->field_count++;
                ctx->state = FSM_CSV_FIELD_START;
                ctx->char_pos = 0;
            } else {
                ctx->field_count++;
                ctx->record_count++;
                ctx->field_count = 0;
                ctx->state = FSM_CSV_FIELD_START;
            }
            break;
        case FSM_CSV_ERROR:
            break;
    }
}

int fsm_csv_test(void) {
    fsm_csv_context_t ctx;
    fsm_csv_init(&ctx);
    const char *data = "a,b,c\n1,2,3\n";
    int i = 0;
    while (data[i] != '\0') {
        fsm_csv_feed(&ctx, data[i]);
        i++;
    }
    if (ctx.record_count < 2) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1132: CSV parser should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1132: empty output");
    assert!(code.contains("fn fsm_csv_init"), "C1132: Should contain fsm_csv_init");
    assert!(code.contains("fn fsm_csv_feed"), "C1132: Should contain fsm_csv_feed");
    Ok(())
}

/// C1133: XML tag parser FSM for element name and depth tracking
#[test]
fn c1133_xml_tag_parser() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef enum {
    FSM_XML_TEXT,
    FSM_XML_TAG_OPEN,
    FSM_XML_TAG_NAME,
    FSM_XML_TAG_ATTR,
    FSM_XML_SELF_CLOSE,
    FSM_XML_ERROR
} fsm_xml_state_t;

typedef struct {
    fsm_xml_state_t state;
    char tag_name[64];
    int name_len;
    int depth;
    int tag_count;
    int is_closing;
} fsm_xml_context_t;

void fsm_xml_init(fsm_xml_context_t *ctx) {
    ctx->state = FSM_XML_TEXT;
    ctx->name_len = 0;
    ctx->depth = 0;
    ctx->tag_count = 0;
    ctx->is_closing = 0;
    ctx->tag_name[0] = '\0';
}

void fsm_xml_feed(fsm_xml_context_t *ctx, char c) {
    switch (ctx->state) {
        case FSM_XML_TEXT:
            if (c == '<') {
                ctx->state = FSM_XML_TAG_OPEN;
                ctx->name_len = 0;
                ctx->is_closing = 0;
            }
            break;
        case FSM_XML_TAG_OPEN:
            if (c == '/') {
                ctx->is_closing = 1;
                ctx->state = FSM_XML_TAG_NAME;
            } else if ((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')) {
                ctx->tag_name[0] = c;
                ctx->name_len = 1;
                ctx->state = FSM_XML_TAG_NAME;
            } else {
                ctx->state = FSM_XML_ERROR;
            }
            break;
        case FSM_XML_TAG_NAME:
            if (c == '>') {
                ctx->tag_name[ctx->name_len] = '\0';
                ctx->tag_count++;
                if (ctx->is_closing) ctx->depth--;
                else ctx->depth++;
                ctx->state = FSM_XML_TEXT;
            } else if (c == '/') {
                ctx->state = FSM_XML_SELF_CLOSE;
            } else if (c == ' ') {
                ctx->tag_name[ctx->name_len] = '\0';
                ctx->state = FSM_XML_TAG_ATTR;
            } else {
                if (ctx->name_len < 63) ctx->tag_name[ctx->name_len++] = c;
            }
            break;
        case FSM_XML_TAG_ATTR:
            if (c == '>') {
                ctx->tag_count++;
                ctx->depth++;
                ctx->state = FSM_XML_TEXT;
            } else if (c == '/') {
                ctx->state = FSM_XML_SELF_CLOSE;
            }
            break;
        case FSM_XML_SELF_CLOSE:
            if (c == '>') {
                ctx->tag_name[ctx->name_len] = '\0';
                ctx->tag_count++;
                ctx->state = FSM_XML_TEXT;
            }
            break;
        case FSM_XML_ERROR:
            break;
    }
}

int fsm_xml_test(void) {
    fsm_xml_context_t ctx;
    fsm_xml_init(&ctx);
    const char *xml = "<root><item/></root>";
    int i = 0;
    while (xml[i] != '\0') {
        fsm_xml_feed(&ctx, xml[i]);
        i++;
    }
    if (ctx.tag_count < 3) return -1;
    if (ctx.depth != 0) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1133: XML tag parser should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1133: empty output");
    assert!(code.contains("fn fsm_xml_init"), "C1133: Should contain fsm_xml_init");
    assert!(code.contains("fn fsm_xml_feed"), "C1133: Should contain fsm_xml_feed");
    Ok(())
}

/// C1134: INI file parser FSM for sections and key-value pairs
#[test]
fn c1134_ini_file_parser() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef enum {
    FSM_INI_LINE_START,
    FSM_INI_SECTION,
    FSM_INI_KEY,
    FSM_INI_VALUE,
    FSM_INI_COMMENT,
    FSM_INI_ERROR
} fsm_ini_state_t;

typedef struct {
    fsm_ini_state_t state;
    int section_count;
    int key_count;
    char current_section[32];
    int sec_len;
    char current_key[32];
    int key_len;
    char current_value[64];
    int val_len;
} fsm_ini_context_t;

void fsm_ini_init(fsm_ini_context_t *ctx) {
    ctx->state = FSM_INI_LINE_START;
    ctx->section_count = 0;
    ctx->key_count = 0;
    ctx->sec_len = 0;
    ctx->key_len = 0;
    ctx->val_len = 0;
    ctx->current_section[0] = '\0';
    ctx->current_key[0] = '\0';
    ctx->current_value[0] = '\0';
}

void fsm_ini_feed(fsm_ini_context_t *ctx, char c) {
    switch (ctx->state) {
        case FSM_INI_LINE_START:
            if (c == '[') {
                ctx->sec_len = 0;
                ctx->state = FSM_INI_SECTION;
            } else if (c == ';' || c == '#') {
                ctx->state = FSM_INI_COMMENT;
            } else if (c == '\n') {
                break;
            } else if (c != ' ' && c != '\t') {
                ctx->key_len = 0;
                ctx->current_key[ctx->key_len++] = c;
                ctx->state = FSM_INI_KEY;
            }
            break;
        case FSM_INI_SECTION:
            if (c == ']') {
                ctx->current_section[ctx->sec_len] = '\0';
                ctx->section_count++;
                ctx->state = FSM_INI_COMMENT;
            } else if (ctx->sec_len < 31) {
                ctx->current_section[ctx->sec_len++] = c;
            }
            break;
        case FSM_INI_KEY:
            if (c == '=') {
                ctx->current_key[ctx->key_len] = '\0';
                ctx->val_len = 0;
                ctx->state = FSM_INI_VALUE;
            } else if (c == '\n') {
                ctx->state = FSM_INI_LINE_START;
            } else if (ctx->key_len < 31 && c != ' ') {
                ctx->current_key[ctx->key_len++] = c;
            }
            break;
        case FSM_INI_VALUE:
            if (c == '\n') {
                ctx->current_value[ctx->val_len] = '\0';
                ctx->key_count++;
                ctx->state = FSM_INI_LINE_START;
            } else if (ctx->val_len < 63) {
                ctx->current_value[ctx->val_len++] = c;
            }
            break;
        case FSM_INI_COMMENT:
            if (c == '\n') ctx->state = FSM_INI_LINE_START;
            break;
        case FSM_INI_ERROR:
            break;
    }
}

int fsm_ini_test(void) {
    fsm_ini_context_t ctx;
    fsm_ini_init(&ctx);
    const char *ini = "[db]\nhost=local\nport=5432\n";
    int i = 0;
    while (ini[i] != '\0') {
        fsm_ini_feed(&ctx, ini[i]);
        i++;
    }
    if (ctx.key_count != 2) return -1;
    if (ctx.section_count != 1) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1134: INI file parser should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1134: empty output");
    assert!(code.contains("fn fsm_ini_init"), "C1134: Should contain fsm_ini_init");
    assert!(code.contains("fn fsm_ini_feed"), "C1134: Should contain fsm_ini_feed");
    Ok(())
}

/// C1135: ANSI escape sequence parser FSM
#[test]
fn c1135_ansi_escape_parser() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef enum {
    FSM_ANS_NORMAL,
    FSM_ANS_ESC,
    FSM_ANS_CSI,
    FSM_ANS_PARAM,
    FSM_ANS_INTERMEDIATE,
    FSM_ANS_COMPLETE
} fsm_ans_state_t;

typedef struct {
    fsm_ans_state_t state;
    int params[8];
    int param_count;
    int current_param;
    char final_char;
    int sequence_count;
} fsm_ans_context_t;

void fsm_ans_init(fsm_ans_context_t *ctx) {
    ctx->state = FSM_ANS_NORMAL;
    ctx->param_count = 0;
    ctx->current_param = 0;
    ctx->final_char = '\0';
    ctx->sequence_count = 0;
    int i;
    for (i = 0; i < 8; i++) ctx->params[i] = 0;
}

void fsm_ans_reset_params(fsm_ans_context_t *ctx) {
    ctx->param_count = 0;
    ctx->current_param = 0;
    int i;
    for (i = 0; i < 8; i++) ctx->params[i] = 0;
}

void fsm_ans_feed(fsm_ans_context_t *ctx, char c) {
    switch (ctx->state) {
        case FSM_ANS_NORMAL:
            if (c == 27) {
                ctx->state = FSM_ANS_ESC;
                fsm_ans_reset_params(ctx);
            }
            break;
        case FSM_ANS_ESC:
            if (c == '[') {
                ctx->state = FSM_ANS_CSI;
            } else {
                ctx->state = FSM_ANS_NORMAL;
            }
            break;
        case FSM_ANS_CSI:
            if (c >= '0' && c <= '9') {
                ctx->current_param = c - '0';
                ctx->state = FSM_ANS_PARAM;
            } else if (c >= '@' && c <= '~') {
                ctx->final_char = c;
                ctx->sequence_count++;
                ctx->state = FSM_ANS_NORMAL;
            }
            break;
        case FSM_ANS_PARAM:
            if (c >= '0' && c <= '9') {
                ctx->current_param = ctx->current_param * 10 + (c - '0');
            } else if (c == ';') {
                if (ctx->param_count < 8) {
                    ctx->params[ctx->param_count++] = ctx->current_param;
                }
                ctx->current_param = 0;
            } else if (c >= '@' && c <= '~') {
                if (ctx->param_count < 8) {
                    ctx->params[ctx->param_count++] = ctx->current_param;
                }
                ctx->final_char = c;
                ctx->sequence_count++;
                ctx->state = FSM_ANS_NORMAL;
            }
            break;
        case FSM_ANS_INTERMEDIATE:
        case FSM_ANS_COMPLETE:
            ctx->state = FSM_ANS_NORMAL;
            break;
    }
}

int fsm_ans_test(void) {
    fsm_ans_context_t ctx;
    fsm_ans_init(&ctx);
    char seq[5];
    seq[0] = 27; seq[1] = '['; seq[2] = '3'; seq[3] = '1'; seq[4] = 'm';
    int i;
    for (i = 0; i < 5; i++) {
        fsm_ans_feed(&ctx, seq[i]);
    }
    if (ctx.sequence_count != 1) return -1;
    if (ctx.params[0] != 31) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1135: ANSI escape parser should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1135: empty output");
    assert!(code.contains("fn fsm_ans_init"), "C1135: Should contain fsm_ans_init");
    assert!(code.contains("fn fsm_ans_feed"), "C1135: Should contain fsm_ans_feed");
    Ok(())
}

// ============================================================================
// C1136-C1140: Protocol State Machines
// ============================================================================

/// C1136: TCP connection state machine (simplified RFC 793)
#[test]
fn c1136_tcp_state_machine() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef enum {
    FSM_TCP_CLOSED,
    FSM_TCP_LISTEN,
    FSM_TCP_SYN_SENT,
    FSM_TCP_SYN_RECEIVED,
    FSM_TCP_ESTABLISHED,
    FSM_TCP_FIN_WAIT_1,
    FSM_TCP_FIN_WAIT_2,
    FSM_TCP_CLOSE_WAIT,
    FSM_TCP_CLOSING,
    FSM_TCP_LAST_ACK,
    FSM_TCP_TIME_WAIT
} fsm_tcp_state_t;

typedef enum {
    FSM_TCP_EV_PASSIVE_OPEN,
    FSM_TCP_EV_ACTIVE_OPEN,
    FSM_TCP_EV_SYN,
    FSM_TCP_EV_SYN_ACK,
    FSM_TCP_EV_ACK,
    FSM_TCP_EV_FIN,
    FSM_TCP_EV_CLOSE,
    FSM_TCP_EV_TIMEOUT
} fsm_tcp_event_t;

typedef struct {
    fsm_tcp_state_t state;
    int seq_num;
    int ack_num;
    int transitions;
} fsm_tcp_context_t;

void fsm_tcp_init(fsm_tcp_context_t *ctx) {
    ctx->state = FSM_TCP_CLOSED;
    ctx->seq_num = 1000;
    ctx->ack_num = 0;
    ctx->transitions = 0;
}

void fsm_tcp_event(fsm_tcp_context_t *ctx, fsm_tcp_event_t ev) {
    ctx->transitions++;
    switch (ctx->state) {
        case FSM_TCP_CLOSED:
            if (ev == FSM_TCP_EV_PASSIVE_OPEN) ctx->state = FSM_TCP_LISTEN;
            else if (ev == FSM_TCP_EV_ACTIVE_OPEN) {
                ctx->state = FSM_TCP_SYN_SENT;
                ctx->seq_num++;
            }
            break;
        case FSM_TCP_LISTEN:
            if (ev == FSM_TCP_EV_SYN) {
                ctx->state = FSM_TCP_SYN_RECEIVED;
                ctx->ack_num = ctx->seq_num + 1;
            }
            break;
        case FSM_TCP_SYN_SENT:
            if (ev == FSM_TCP_EV_SYN_ACK) {
                ctx->state = FSM_TCP_ESTABLISHED;
                ctx->ack_num++;
            }
            break;
        case FSM_TCP_SYN_RECEIVED:
            if (ev == FSM_TCP_EV_ACK) ctx->state = FSM_TCP_ESTABLISHED;
            break;
        case FSM_TCP_ESTABLISHED:
            if (ev == FSM_TCP_EV_FIN) ctx->state = FSM_TCP_CLOSE_WAIT;
            else if (ev == FSM_TCP_EV_CLOSE) ctx->state = FSM_TCP_FIN_WAIT_1;
            break;
        case FSM_TCP_FIN_WAIT_1:
            if (ev == FSM_TCP_EV_ACK) ctx->state = FSM_TCP_FIN_WAIT_2;
            else if (ev == FSM_TCP_EV_FIN) ctx->state = FSM_TCP_CLOSING;
            break;
        case FSM_TCP_FIN_WAIT_2:
            if (ev == FSM_TCP_EV_FIN) ctx->state = FSM_TCP_TIME_WAIT;
            break;
        case FSM_TCP_CLOSE_WAIT:
            if (ev == FSM_TCP_EV_CLOSE) ctx->state = FSM_TCP_LAST_ACK;
            break;
        case FSM_TCP_CLOSING:
            if (ev == FSM_TCP_EV_ACK) ctx->state = FSM_TCP_TIME_WAIT;
            break;
        case FSM_TCP_LAST_ACK:
            if (ev == FSM_TCP_EV_ACK) ctx->state = FSM_TCP_CLOSED;
            break;
        case FSM_TCP_TIME_WAIT:
            if (ev == FSM_TCP_EV_TIMEOUT) ctx->state = FSM_TCP_CLOSED;
            break;
    }
}

int fsm_tcp_test(void) {
    fsm_tcp_context_t ctx;
    fsm_tcp_init(&ctx);
    fsm_tcp_event(&ctx, FSM_TCP_EV_ACTIVE_OPEN);
    if (ctx.state != FSM_TCP_SYN_SENT) return -1;
    fsm_tcp_event(&ctx, FSM_TCP_EV_SYN_ACK);
    if (ctx.state != FSM_TCP_ESTABLISHED) return -2;
    fsm_tcp_event(&ctx, FSM_TCP_EV_CLOSE);
    if (ctx.state != FSM_TCP_FIN_WAIT_1) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1136: TCP state machine should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1136: empty output");
    assert!(code.contains("fn fsm_tcp_init"), "C1136: Should contain fsm_tcp_init");
    assert!(code.contains("fn fsm_tcp_event"), "C1136: Should contain fsm_tcp_event");
    Ok(())
}

/// C1137: HTTP/1.1 request pipeline state machine
#[test]
fn c1137_http_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef enum {
    FSM_HTTP_IDLE,
    FSM_HTTP_METHOD,
    FSM_HTTP_URI,
    FSM_HTTP_VERSION,
    FSM_HTTP_HEADER_NAME,
    FSM_HTTP_HEADER_VALUE,
    FSM_HTTP_BODY,
    FSM_HTTP_COMPLETE,
    FSM_HTTP_ERROR
} fsm_http_state_t;

typedef struct {
    fsm_http_state_t state;
    char method[16];
    int method_len;
    int header_count;
    int content_length;
    int body_received;
    int request_count;
} fsm_http_context_t;

void fsm_http_init(fsm_http_context_t *ctx) {
    ctx->state = FSM_HTTP_IDLE;
    ctx->method_len = 0;
    ctx->header_count = 0;
    ctx->content_length = 0;
    ctx->body_received = 0;
    ctx->request_count = 0;
}

void fsm_http_feed(fsm_http_context_t *ctx, char c) {
    switch (ctx->state) {
        case FSM_HTTP_IDLE:
            if (c != ' ' && c != '\r' && c != '\n') {
                ctx->method_len = 0;
                ctx->method[ctx->method_len++] = c;
                ctx->state = FSM_HTTP_METHOD;
            }
            break;
        case FSM_HTTP_METHOD:
            if (c == ' ') {
                ctx->method[ctx->method_len] = '\0';
                ctx->state = FSM_HTTP_URI;
            } else if (ctx->method_len < 15) {
                ctx->method[ctx->method_len++] = c;
            }
            break;
        case FSM_HTTP_URI:
            if (c == ' ') ctx->state = FSM_HTTP_VERSION;
            break;
        case FSM_HTTP_VERSION:
            if (c == '\n') {
                ctx->state = FSM_HTTP_HEADER_NAME;
                ctx->header_count = 0;
            }
            break;
        case FSM_HTTP_HEADER_NAME:
            if (c == '\n') {
                if (ctx->content_length > 0) {
                    ctx->state = FSM_HTTP_BODY;
                    ctx->body_received = 0;
                } else {
                    ctx->state = FSM_HTTP_COMPLETE;
                    ctx->request_count++;
                }
            } else if (c == ':') {
                ctx->state = FSM_HTTP_HEADER_VALUE;
                ctx->header_count++;
            }
            break;
        case FSM_HTTP_HEADER_VALUE:
            if (c == '\n') ctx->state = FSM_HTTP_HEADER_NAME;
            break;
        case FSM_HTTP_BODY:
            ctx->body_received++;
            if (ctx->body_received >= ctx->content_length) {
                ctx->state = FSM_HTTP_COMPLETE;
                ctx->request_count++;
            }
            break;
        case FSM_HTTP_COMPLETE:
            ctx->state = FSM_HTTP_IDLE;
            break;
        case FSM_HTTP_ERROR:
            break;
    }
}

int fsm_http_test(void) {
    fsm_http_context_t ctx;
    fsm_http_init(&ctx);
    const char *req = "GET /index HTTP/1.1\r\nHost: local\r\n\r\n";
    int i = 0;
    while (req[i] != '\0') {
        fsm_http_feed(&ctx, req[i]);
        i++;
    }
    if (ctx.request_count != 1) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1137: HTTP pipeline should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1137: empty output");
    assert!(code.contains("fn fsm_http_init"), "C1137: Should contain fsm_http_init");
    assert!(code.contains("fn fsm_http_feed"), "C1137: Should contain fsm_http_feed");
    Ok(())
}

/// C1138: DHCP client state machine (RFC 2131 simplified)
#[test]
fn c1138_dhcp_client() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef enum {
    FSM_DHCP_INIT,
    FSM_DHCP_SELECTING,
    FSM_DHCP_REQUESTING,
    FSM_DHCP_BOUND,
    FSM_DHCP_RENEWING,
    FSM_DHCP_REBINDING
} fsm_dhcp_state_t;

typedef enum {
    FSM_DHCP_EV_START,
    FSM_DHCP_EV_OFFER,
    FSM_DHCP_EV_ACK,
    FSM_DHCP_EV_NAK,
    FSM_DHCP_EV_T1_EXPIRE,
    FSM_DHCP_EV_T2_EXPIRE,
    FSM_DHCP_EV_LEASE_EXPIRE,
    FSM_DHCP_EV_RELEASE
} fsm_dhcp_event_t;

typedef struct {
    fsm_dhcp_state_t state;
    uint32_t offered_ip;
    uint32_t server_ip;
    int lease_time;
    int t1_time;
    int t2_time;
    int elapsed;
    int retry_count;
} fsm_dhcp_context_t;

void fsm_dhcp_init(fsm_dhcp_context_t *ctx) {
    ctx->state = FSM_DHCP_INIT;
    ctx->offered_ip = 0;
    ctx->server_ip = 0;
    ctx->lease_time = 0;
    ctx->t1_time = 0;
    ctx->t2_time = 0;
    ctx->elapsed = 0;
    ctx->retry_count = 0;
}

void fsm_dhcp_event(fsm_dhcp_context_t *ctx, fsm_dhcp_event_t ev) {
    switch (ctx->state) {
        case FSM_DHCP_INIT:
            if (ev == FSM_DHCP_EV_START) {
                ctx->state = FSM_DHCP_SELECTING;
                ctx->retry_count = 0;
            }
            break;
        case FSM_DHCP_SELECTING:
            if (ev == FSM_DHCP_EV_OFFER) {
                ctx->state = FSM_DHCP_REQUESTING;
                ctx->offered_ip = 0xC0A80164;
                ctx->server_ip = 0xC0A80101;
            }
            break;
        case FSM_DHCP_REQUESTING:
            if (ev == FSM_DHCP_EV_ACK) {
                ctx->state = FSM_DHCP_BOUND;
                ctx->lease_time = 3600;
                ctx->t1_time = 1800;
                ctx->t2_time = 3150;
                ctx->elapsed = 0;
            } else if (ev == FSM_DHCP_EV_NAK) {
                ctx->state = FSM_DHCP_INIT;
                ctx->offered_ip = 0;
            }
            break;
        case FSM_DHCP_BOUND:
            if (ev == FSM_DHCP_EV_T1_EXPIRE) {
                ctx->state = FSM_DHCP_RENEWING;
            } else if (ev == FSM_DHCP_EV_RELEASE) {
                ctx->state = FSM_DHCP_INIT;
                ctx->offered_ip = 0;
            }
            break;
        case FSM_DHCP_RENEWING:
            if (ev == FSM_DHCP_EV_ACK) {
                ctx->state = FSM_DHCP_BOUND;
                ctx->elapsed = 0;
            } else if (ev == FSM_DHCP_EV_T2_EXPIRE) {
                ctx->state = FSM_DHCP_REBINDING;
            }
            break;
        case FSM_DHCP_REBINDING:
            if (ev == FSM_DHCP_EV_ACK) {
                ctx->state = FSM_DHCP_BOUND;
                ctx->elapsed = 0;
            } else if (ev == FSM_DHCP_EV_LEASE_EXPIRE) {
                ctx->state = FSM_DHCP_INIT;
                ctx->offered_ip = 0;
            }
            break;
    }
}

int fsm_dhcp_test(void) {
    fsm_dhcp_context_t ctx;
    fsm_dhcp_init(&ctx);
    fsm_dhcp_event(&ctx, FSM_DHCP_EV_START);
    if (ctx.state != FSM_DHCP_SELECTING) return -1;
    fsm_dhcp_event(&ctx, FSM_DHCP_EV_OFFER);
    if (ctx.state != FSM_DHCP_REQUESTING) return -2;
    fsm_dhcp_event(&ctx, FSM_DHCP_EV_ACK);
    if (ctx.state != FSM_DHCP_BOUND) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1138: DHCP client should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1138: empty output");
    assert!(code.contains("fn fsm_dhcp_init"), "C1138: Should contain fsm_dhcp_init");
    assert!(code.contains("fn fsm_dhcp_event"), "C1138: Should contain fsm_dhcp_event");
    Ok(())
}

/// C1139: TLS handshake simulation state machine
#[test]
fn c1139_tls_handshake() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef enum {
    FSM_TLS_IDLE,
    FSM_TLS_CLIENT_HELLO,
    FSM_TLS_SERVER_HELLO,
    FSM_TLS_CERT_VERIFY,
    FSM_TLS_KEY_EXCHANGE,
    FSM_TLS_FINISHED,
    FSM_TLS_ACTIVE,
    FSM_TLS_ERROR
} fsm_tls_state_t;

typedef struct {
    fsm_tls_state_t state;
    int cipher_suite;
    int version;
    int session_id;
    int handshake_steps;
    int error_code;
} fsm_tls_context_t;

void fsm_tls_init(fsm_tls_context_t *ctx) {
    ctx->state = FSM_TLS_IDLE;
    ctx->cipher_suite = 0;
    ctx->version = 0;
    ctx->session_id = 0;
    ctx->handshake_steps = 0;
    ctx->error_code = 0;
}

int fsm_tls_step(fsm_tls_context_t *ctx, int msg_type, int payload) {
    ctx->handshake_steps++;
    switch (ctx->state) {
        case FSM_TLS_IDLE:
            if (msg_type == 1) {
                ctx->version = payload;
                ctx->state = FSM_TLS_CLIENT_HELLO;
                return 0;
            }
            break;
        case FSM_TLS_CLIENT_HELLO:
            if (msg_type == 2) {
                ctx->cipher_suite = payload;
                ctx->state = FSM_TLS_SERVER_HELLO;
                return 0;
            }
            break;
        case FSM_TLS_SERVER_HELLO:
            if (msg_type == 3) {
                ctx->state = FSM_TLS_CERT_VERIFY;
                return 0;
            }
            break;
        case FSM_TLS_CERT_VERIFY:
            if (msg_type == 4) {
                if (payload == 1) {
                    ctx->state = FSM_TLS_KEY_EXCHANGE;
                    return 0;
                } else {
                    ctx->state = FSM_TLS_ERROR;
                    ctx->error_code = 42;
                    return -1;
                }
            }
            break;
        case FSM_TLS_KEY_EXCHANGE:
            if (msg_type == 5) {
                ctx->session_id = payload;
                ctx->state = FSM_TLS_FINISHED;
                return 0;
            }
            break;
        case FSM_TLS_FINISHED:
            if (msg_type == 6) {
                ctx->state = FSM_TLS_ACTIVE;
                return 0;
            }
            break;
        case FSM_TLS_ACTIVE:
        case FSM_TLS_ERROR:
            break;
    }
    ctx->state = FSM_TLS_ERROR;
    ctx->error_code = 99;
    return -1;
}

int fsm_tls_test(void) {
    fsm_tls_context_t ctx;
    fsm_tls_init(&ctx);
    if (fsm_tls_step(&ctx, 1, 0x0303) != 0) return -1;
    if (fsm_tls_step(&ctx, 2, 0x1301) != 0) return -2;
    if (fsm_tls_step(&ctx, 3, 0) != 0) return -3;
    if (fsm_tls_step(&ctx, 4, 1) != 0) return -4;
    if (fsm_tls_step(&ctx, 5, 12345) != 0) return -5;
    if (fsm_tls_step(&ctx, 6, 0) != 0) return -6;
    if (ctx.state != FSM_TLS_ACTIVE) return -7;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1139: TLS handshake should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1139: empty output");
    assert!(code.contains("fn fsm_tls_init"), "C1139: Should contain fsm_tls_init");
    assert!(code.contains("fn fsm_tls_step"), "C1139: Should contain fsm_tls_step");
    Ok(())
}

/// C1140: PPP link control protocol state machine
#[test]
fn c1140_ppp_link_control() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef enum {
    FSM_PPP_DEAD,
    FSM_PPP_ESTABLISH,
    FSM_PPP_AUTH,
    FSM_PPP_NETWORK,
    FSM_PPP_OPEN,
    FSM_PPP_TERMINATE
} fsm_ppp_state_t;

typedef enum {
    FSM_PPP_EV_UP,
    FSM_PPP_EV_DOWN,
    FSM_PPP_EV_OPEN,
    FSM_PPP_EV_CLOSE,
    FSM_PPP_EV_AUTH_OK,
    FSM_PPP_EV_AUTH_FAIL,
    FSM_PPP_EV_NCP_OK,
    FSM_PPP_EV_TIMEOUT
} fsm_ppp_event_t;

typedef struct {
    fsm_ppp_state_t state;
    int retries;
    int max_retries;
    int restart_timer;
    int auth_failures;
} fsm_ppp_context_t;

void fsm_ppp_init(fsm_ppp_context_t *ctx) {
    ctx->state = FSM_PPP_DEAD;
    ctx->retries = 0;
    ctx->max_retries = 5;
    ctx->restart_timer = 0;
    ctx->auth_failures = 0;
}

void fsm_ppp_event(fsm_ppp_context_t *ctx, fsm_ppp_event_t ev) {
    switch (ctx->state) {
        case FSM_PPP_DEAD:
            if (ev == FSM_PPP_EV_UP) {
                ctx->state = FSM_PPP_ESTABLISH;
                ctx->retries = 0;
            }
            break;
        case FSM_PPP_ESTABLISH:
            if (ev == FSM_PPP_EV_OPEN) {
                ctx->state = FSM_PPP_AUTH;
            } else if (ev == FSM_PPP_EV_DOWN) {
                ctx->state = FSM_PPP_DEAD;
            } else if (ev == FSM_PPP_EV_TIMEOUT) {
                ctx->retries++;
                if (ctx->retries >= ctx->max_retries)
                    ctx->state = FSM_PPP_DEAD;
            }
            break;
        case FSM_PPP_AUTH:
            if (ev == FSM_PPP_EV_AUTH_OK) {
                ctx->state = FSM_PPP_NETWORK;
            } else if (ev == FSM_PPP_EV_AUTH_FAIL) {
                ctx->auth_failures++;
                if (ctx->auth_failures >= 3)
                    ctx->state = FSM_PPP_TERMINATE;
                else
                    ctx->state = FSM_PPP_ESTABLISH;
            }
            break;
        case FSM_PPP_NETWORK:
            if (ev == FSM_PPP_EV_NCP_OK) {
                ctx->state = FSM_PPP_OPEN;
            } else if (ev == FSM_PPP_EV_DOWN) {
                ctx->state = FSM_PPP_DEAD;
            }
            break;
        case FSM_PPP_OPEN:
            if (ev == FSM_PPP_EV_CLOSE) {
                ctx->state = FSM_PPP_TERMINATE;
            } else if (ev == FSM_PPP_EV_DOWN) {
                ctx->state = FSM_PPP_DEAD;
            }
            break;
        case FSM_PPP_TERMINATE:
            if (ev == FSM_PPP_EV_TIMEOUT) {
                ctx->state = FSM_PPP_DEAD;
            }
            break;
    }
}

int fsm_ppp_test(void) {
    fsm_ppp_context_t ctx;
    fsm_ppp_init(&ctx);
    fsm_ppp_event(&ctx, FSM_PPP_EV_UP);
    if (ctx.state != FSM_PPP_ESTABLISH) return -1;
    fsm_ppp_event(&ctx, FSM_PPP_EV_OPEN);
    if (ctx.state != FSM_PPP_AUTH) return -2;
    fsm_ppp_event(&ctx, FSM_PPP_EV_AUTH_OK);
    fsm_ppp_event(&ctx, FSM_PPP_EV_NCP_OK);
    if (ctx.state != FSM_PPP_OPEN) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1140: PPP link control should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1140: empty output");
    assert!(code.contains("fn fsm_ppp_init"), "C1140: Should contain fsm_ppp_init");
    assert!(code.contains("fn fsm_ppp_event"), "C1140: Should contain fsm_ppp_event");
    Ok(())
}

// ============================================================================
// C1141-C1145: Game/UI State Machines
// ============================================================================

/// C1141: Game menu system with sub-menus and navigation
#[test]
fn c1141_game_menu_fsm() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef enum {
    FSM_MENU_MAIN,
    FSM_MENU_PLAY,
    FSM_MENU_OPTIONS,
    FSM_MENU_AUDIO,
    FSM_MENU_VIDEO,
    FSM_MENU_CREDITS,
    FSM_MENU_QUIT_CONFIRM
} fsm_menu_state_t;

typedef struct {
    fsm_menu_state_t state;
    fsm_menu_state_t history[8];
    int history_depth;
    int selected_item;
    int max_items;
} fsm_menu_context_t;

void fsm_menu_init(fsm_menu_context_t *ctx) {
    ctx->state = FSM_MENU_MAIN;
    ctx->history_depth = 0;
    ctx->selected_item = 0;
    ctx->max_items = 4;
}

void fsm_menu_push(fsm_menu_context_t *ctx, fsm_menu_state_t next) {
    if (ctx->history_depth < 8) {
        ctx->history[ctx->history_depth++] = ctx->state;
    }
    ctx->state = next;
    ctx->selected_item = 0;
}

void fsm_menu_pop(fsm_menu_context_t *ctx) {
    if (ctx->history_depth > 0) {
        ctx->state = ctx->history[--ctx->history_depth];
        ctx->selected_item = 0;
    }
}

void fsm_menu_navigate(fsm_menu_context_t *ctx, int direction) {
    ctx->selected_item += direction;
    if (ctx->selected_item < 0) ctx->selected_item = ctx->max_items - 1;
    if (ctx->selected_item >= ctx->max_items) ctx->selected_item = 0;
}

void fsm_menu_select(fsm_menu_context_t *ctx) {
    switch (ctx->state) {
        case FSM_MENU_MAIN:
            if (ctx->selected_item == 0) fsm_menu_push(ctx, FSM_MENU_PLAY);
            else if (ctx->selected_item == 1) fsm_menu_push(ctx, FSM_MENU_OPTIONS);
            else if (ctx->selected_item == 2) fsm_menu_push(ctx, FSM_MENU_CREDITS);
            else if (ctx->selected_item == 3) fsm_menu_push(ctx, FSM_MENU_QUIT_CONFIRM);
            break;
        case FSM_MENU_OPTIONS:
            if (ctx->selected_item == 0) fsm_menu_push(ctx, FSM_MENU_AUDIO);
            else if (ctx->selected_item == 1) fsm_menu_push(ctx, FSM_MENU_VIDEO);
            break;
        default:
            break;
    }
}

int fsm_menu_test(void) {
    fsm_menu_context_t ctx;
    fsm_menu_init(&ctx);
    ctx.selected_item = 1;
    fsm_menu_select(&ctx);
    if (ctx.state != FSM_MENU_OPTIONS) return -1;
    fsm_menu_select(&ctx);
    if (ctx.state != FSM_MENU_AUDIO) return -2;
    fsm_menu_pop(&ctx);
    if (ctx.state != FSM_MENU_OPTIONS) return -3;
    fsm_menu_pop(&ctx);
    if (ctx.state != FSM_MENU_MAIN) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1141: game menu should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1141: empty output");
    assert!(code.contains("fn fsm_menu_init"), "C1141: Should contain fsm_menu_init");
    assert!(code.contains("fn fsm_menu_select"), "C1141: Should contain fsm_menu_select");
    Ok(())
}

/// C1142: Animation controller with blend and transition states
#[test]
fn c1142_animation_controller() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef enum {
    FSM_ANIM_IDLE,
    FSM_ANIM_WALK,
    FSM_ANIM_RUN,
    FSM_ANIM_JUMP,
    FSM_ANIM_FALL,
    FSM_ANIM_LAND,
    FSM_ANIM_BLEND
} fsm_anim_state_t;

typedef struct {
    fsm_anim_state_t state;
    fsm_anim_state_t target_state;
    int frame;
    int frame_count;
    int speed;
    int blend_timer;
    int blend_duration;
    int grounded;
} fsm_anim_context_t;

void fsm_anim_init(fsm_anim_context_t *ctx) {
    ctx->state = FSM_ANIM_IDLE;
    ctx->target_state = FSM_ANIM_IDLE;
    ctx->frame = 0;
    ctx->frame_count = 8;
    ctx->speed = 1;
    ctx->blend_timer = 0;
    ctx->blend_duration = 4;
    ctx->grounded = 1;
}

void fsm_anim_transition(fsm_anim_context_t *ctx, fsm_anim_state_t next) {
    if (ctx->state != next) {
        ctx->target_state = next;
        ctx->state = FSM_ANIM_BLEND;
        ctx->blend_timer = 0;
    }
}

void fsm_anim_update(fsm_anim_context_t *ctx, int velocity, int vy) {
    if (!ctx->grounded) {
        if (vy < 0) fsm_anim_transition(ctx, FSM_ANIM_JUMP);
        else fsm_anim_transition(ctx, FSM_ANIM_FALL);
    } else {
        if (velocity == 0) fsm_anim_transition(ctx, FSM_ANIM_IDLE);
        else if (velocity < 5) fsm_anim_transition(ctx, FSM_ANIM_WALK);
        else fsm_anim_transition(ctx, FSM_ANIM_RUN);
    }
}

void fsm_anim_tick(fsm_anim_context_t *ctx) {
    if (ctx->state == FSM_ANIM_BLEND) {
        ctx->blend_timer++;
        if (ctx->blend_timer >= ctx->blend_duration) {
            ctx->state = ctx->target_state;
            ctx->frame = 0;
        }
    } else {
        ctx->frame += ctx->speed;
        if (ctx->frame >= ctx->frame_count) ctx->frame = 0;
    }
}

int fsm_anim_test(void) {
    fsm_anim_context_t ctx;
    fsm_anim_init(&ctx);
    fsm_anim_update(&ctx, 3, 0);
    int i;
    for (i = 0; i < 10; i++) fsm_anim_tick(&ctx);
    if (ctx.state != FSM_ANIM_WALK) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1142: animation controller should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1142: empty output");
    assert!(code.contains("fn fsm_anim_init"), "C1142: Should contain fsm_anim_init");
    assert!(code.contains("fn fsm_anim_tick"), "C1142: Should contain fsm_anim_tick");
    Ok(())
}

/// C1143: Dialog tree with branching conversations
#[test]
fn c1143_dialog_tree() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int node_id;
    int choices[4];
    int choice_count;
    int visited;
    int flags;
} fsm_dlg_node_t;

typedef struct {
    fsm_dlg_node_t nodes[32];
    int node_count;
    int current_node;
    int global_flags;
    int nodes_visited;
} fsm_dlg_context_t;

void fsm_dlg_init(fsm_dlg_context_t *ctx) {
    ctx->node_count = 0;
    ctx->current_node = 0;
    ctx->global_flags = 0;
    ctx->nodes_visited = 0;
    int i;
    for (i = 0; i < 32; i++) {
        ctx->nodes[i].node_id = i;
        ctx->nodes[i].choice_count = 0;
        ctx->nodes[i].visited = 0;
        ctx->nodes[i].flags = 0;
    }
}

int fsm_dlg_add_node(fsm_dlg_context_t *ctx) {
    if (ctx->node_count >= 32) return -1;
    int id = ctx->node_count;
    ctx->nodes[id].choice_count = 0;
    ctx->nodes[id].visited = 0;
    ctx->nodes[id].flags = 0;
    ctx->node_count++;
    return id;
}

void fsm_dlg_add_choice(fsm_dlg_context_t *ctx, int from, int to) {
    if (from >= 0 && from < ctx->node_count) {
        int idx = ctx->nodes[from].choice_count;
        if (idx < 4) {
            ctx->nodes[from].choices[idx] = to;
            ctx->nodes[from].choice_count++;
        }
    }
}

int fsm_dlg_choose(fsm_dlg_context_t *ctx, int choice) {
    int cur = ctx->current_node;
    if (cur < 0 || cur >= ctx->node_count) return -1;
    if (choice < 0 || choice >= ctx->nodes[cur].choice_count) return -2;
    if (!ctx->nodes[cur].visited) {
        ctx->nodes[cur].visited = 1;
        ctx->nodes_visited++;
    }
    ctx->global_flags = ctx->global_flags | ctx->nodes[cur].flags;
    ctx->current_node = ctx->nodes[cur].choices[choice];
    return ctx->current_node;
}

int fsm_dlg_test(void) {
    fsm_dlg_context_t ctx;
    fsm_dlg_init(&ctx);
    int n0 = fsm_dlg_add_node(&ctx);
    int n1 = fsm_dlg_add_node(&ctx);
    int n2 = fsm_dlg_add_node(&ctx);
    fsm_dlg_add_choice(&ctx, n0, n1);
    fsm_dlg_add_choice(&ctx, n0, n2);
    fsm_dlg_add_choice(&ctx, n1, n2);
    int r = fsm_dlg_choose(&ctx, 0);
    if (r != n1) return -1;
    r = fsm_dlg_choose(&ctx, 0);
    if (r != n2) return -2;
    if (ctx.nodes_visited != 2) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1143: dialog tree should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1143: empty output");
    assert!(code.contains("fn fsm_dlg_init"), "C1143: Should contain fsm_dlg_init");
    assert!(code.contains("fn fsm_dlg_choose"), "C1143: Should contain fsm_dlg_choose");
    Ok(())
}

/// C1144: Input combo detector (fighting game style sequences)
#[test]
fn c1144_input_combo_detector() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef enum {
    FSM_CB_IDLE,
    FSM_CB_SEQ1,
    FSM_CB_SEQ2,
    FSM_CB_SEQ3,
    FSM_CB_TRIGGERED,
    FSM_CB_COOLDOWN
} fsm_cb_state_t;

typedef struct {
    fsm_cb_state_t state;
    int timer;
    int timeout;
    int combo_count;
    int cooldown_timer;
    int cooldown_duration;
    int sequence[4];
    int seq_len;
} fsm_cb_context_t;

void fsm_cb_init(fsm_cb_context_t *ctx) {
    ctx->state = FSM_CB_IDLE;
    ctx->timer = 0;
    ctx->timeout = 15;
    ctx->combo_count = 0;
    ctx->cooldown_timer = 0;
    ctx->cooldown_duration = 10;
    ctx->sequence[0] = 1;
    ctx->sequence[1] = 2;
    ctx->sequence[2] = 3;
    ctx->sequence[3] = 4;
    ctx->seq_len = 4;
}

void fsm_cb_input(fsm_cb_context_t *ctx, int button) {
    if (ctx->state == FSM_CB_COOLDOWN) return;
    if (ctx->timer > ctx->timeout && ctx->state != FSM_CB_IDLE) {
        ctx->state = FSM_CB_IDLE;
        ctx->timer = 0;
    }
    switch (ctx->state) {
        case FSM_CB_IDLE:
            if (button == ctx->sequence[0]) {
                ctx->state = FSM_CB_SEQ1;
                ctx->timer = 0;
            }
            break;
        case FSM_CB_SEQ1:
            if (button == ctx->sequence[1]) {
                ctx->state = FSM_CB_SEQ2;
                ctx->timer = 0;
            } else {
                ctx->state = FSM_CB_IDLE;
            }
            break;
        case FSM_CB_SEQ2:
            if (button == ctx->sequence[2]) {
                ctx->state = FSM_CB_SEQ3;
                ctx->timer = 0;
            } else {
                ctx->state = FSM_CB_IDLE;
            }
            break;
        case FSM_CB_SEQ3:
            if (button == ctx->sequence[3]) {
                ctx->state = FSM_CB_TRIGGERED;
                ctx->combo_count++;
                ctx->cooldown_timer = 0;
            } else {
                ctx->state = FSM_CB_IDLE;
            }
            break;
        case FSM_CB_TRIGGERED:
            ctx->state = FSM_CB_COOLDOWN;
            break;
        case FSM_CB_COOLDOWN:
            break;
    }
}

void fsm_cb_tick(fsm_cb_context_t *ctx) {
    ctx->timer++;
    if (ctx->state == FSM_CB_COOLDOWN) {
        ctx->cooldown_timer++;
        if (ctx->cooldown_timer >= ctx->cooldown_duration) {
            ctx->state = FSM_CB_IDLE;
        }
    }
}

int fsm_cb_test(void) {
    fsm_cb_context_t ctx;
    fsm_cb_init(&ctx);
    fsm_cb_input(&ctx, 1);
    fsm_cb_input(&ctx, 2);
    fsm_cb_input(&ctx, 3);
    fsm_cb_input(&ctx, 4);
    if (ctx.state != FSM_CB_TRIGGERED) return -1;
    if (ctx.combo_count != 1) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1144: input combo detector should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1144: empty output");
    assert!(code.contains("fn fsm_cb_init"), "C1144: Should contain fsm_cb_init");
    assert!(code.contains("fn fsm_cb_input"), "C1144: Should contain fsm_cb_input");
    Ok(())
}

/// C1145: Screen manager with fade transitions
#[test]
fn c1145_screen_manager() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef enum {
    FSM_SCR_SPLASH,
    FSM_SCR_LOADING,
    FSM_SCR_GAME,
    FSM_SCR_PAUSE,
    FSM_SCR_GAMEOVER,
    FSM_SCR_FADE_OUT,
    FSM_SCR_FADE_IN
} fsm_scr_state_t;

typedef struct {
    fsm_scr_state_t state;
    fsm_scr_state_t next_screen;
    fsm_scr_state_t prev_screen;
    int transition_timer;
    int transition_duration;
    int fade_alpha;
    int screen_time;
} fsm_scr_context_t;

void fsm_scr_init(fsm_scr_context_t *ctx) {
    ctx->state = FSM_SCR_SPLASH;
    ctx->next_screen = FSM_SCR_SPLASH;
    ctx->prev_screen = FSM_SCR_SPLASH;
    ctx->transition_timer = 0;
    ctx->transition_duration = 30;
    ctx->fade_alpha = 255;
    ctx->screen_time = 0;
}

void fsm_scr_goto(fsm_scr_context_t *ctx, fsm_scr_state_t next) {
    ctx->prev_screen = ctx->state;
    ctx->next_screen = next;
    ctx->state = FSM_SCR_FADE_OUT;
    ctx->transition_timer = 0;
}

void fsm_scr_tick(fsm_scr_context_t *ctx) {
    ctx->screen_time++;
    switch (ctx->state) {
        case FSM_SCR_FADE_OUT:
            ctx->transition_timer++;
            ctx->fade_alpha = 255 * ctx->transition_timer / ctx->transition_duration;
            if (ctx->fade_alpha > 255) ctx->fade_alpha = 255;
            if (ctx->transition_timer >= ctx->transition_duration) {
                ctx->state = FSM_SCR_FADE_IN;
                ctx->transition_timer = 0;
            }
            break;
        case FSM_SCR_FADE_IN:
            ctx->transition_timer++;
            ctx->fade_alpha = 255 - 255 * ctx->transition_timer / ctx->transition_duration;
            if (ctx->fade_alpha < 0) ctx->fade_alpha = 0;
            if (ctx->transition_timer >= ctx->transition_duration) {
                ctx->state = ctx->next_screen;
                ctx->fade_alpha = 0;
                ctx->screen_time = 0;
            }
            break;
        case FSM_SCR_SPLASH:
            if (ctx->screen_time >= 60) {
                fsm_scr_goto(ctx, FSM_SCR_LOADING);
            }
            break;
        default:
            break;
    }
}

void fsm_scr_pause(fsm_scr_context_t *ctx) {
    if (ctx->state == FSM_SCR_GAME) {
        ctx->prev_screen = FSM_SCR_GAME;
        ctx->state = FSM_SCR_PAUSE;
    }
}

void fsm_scr_resume(fsm_scr_context_t *ctx) {
    if (ctx->state == FSM_SCR_PAUSE) {
        ctx->state = FSM_SCR_GAME;
    }
}

int fsm_scr_test(void) {
    fsm_scr_context_t ctx;
    fsm_scr_init(&ctx);
    int i;
    for (i = 0; i < 200; i++) {
        fsm_scr_tick(&ctx);
    }
    if (ctx.state == FSM_SCR_SPLASH) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1145: screen manager should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1145: empty output");
    assert!(code.contains("fn fsm_scr_init"), "C1145: Should contain fsm_scr_init");
    assert!(code.contains("fn fsm_scr_tick"), "C1145: Should contain fsm_scr_tick");
    Ok(())
}

// ============================================================================
// C1146-C1150: Industrial State Machines
// ============================================================================

/// C1146: PID controller with mode switching and output clamping
#[test]
fn c1146_pid_controller() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef enum {
    FSM_PID_OFF,
    FSM_PID_MANUAL,
    FSM_PID_AUTO,
    FSM_PID_CASCADE
} fsm_pid_mode_t;

typedef struct {
    fsm_pid_mode_t mode;
    float setpoint;
    float output;
    float error_sum;
    float last_error;
    float kp;
    float ki;
    float kd;
    float out_min;
    float out_max;
} fsm_pid_context_t;

void fsm_pid_init(fsm_pid_context_t *ctx) {
    ctx->mode = FSM_PID_OFF;
    ctx->setpoint = 0.0f;
    ctx->output = 0.0f;
    ctx->error_sum = 0.0f;
    ctx->last_error = 0.0f;
    ctx->kp = 1.0f;
    ctx->ki = 0.1f;
    ctx->kd = 0.05f;
    ctx->out_min = 0.0f;
    ctx->out_max = 100.0f;
}

void fsm_pid_set_mode(fsm_pid_context_t *ctx, fsm_pid_mode_t mode) {
    if (mode != ctx->mode) {
        ctx->error_sum = 0.0f;
        ctx->last_error = 0.0f;
        ctx->mode = mode;
    }
}

float fsm_pid_clamp(float val, float lo, float hi) {
    if (val < lo) return lo;
    if (val > hi) return hi;
    return val;
}

float fsm_pid_compute(fsm_pid_context_t *ctx, float measurement) {
    if (ctx->mode == FSM_PID_OFF) return 0.0f;
    if (ctx->mode == FSM_PID_MANUAL) return ctx->output;
    float error = ctx->setpoint - measurement;
    ctx->error_sum += error;
    float derivative = error - ctx->last_error;
    ctx->output = ctx->kp * error + ctx->ki * ctx->error_sum + ctx->kd * derivative;
    ctx->output = fsm_pid_clamp(ctx->output, ctx->out_min, ctx->out_max);
    ctx->last_error = error;
    return ctx->output;
}

int fsm_pid_test(void) {
    fsm_pid_context_t ctx;
    fsm_pid_init(&ctx);
    ctx.setpoint = 50.0f;
    fsm_pid_set_mode(&ctx, FSM_PID_AUTO);
    float out = fsm_pid_compute(&ctx, 30.0f);
    if (out < 0.0f) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1146: PID controller should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1146: empty output");
    assert!(code.contains("fn fsm_pid_init"), "C1146: Should contain fsm_pid_init");
    assert!(code.contains("fn fsm_pid_compute"), "C1146: Should contain fsm_pid_compute");
    Ok(())
}

/// C1147: Motor driver state machine with ramp control and fault handling
#[test]
fn c1147_motor_driver() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef enum {
    FSM_MOT_STOPPED,
    FSM_MOT_RAMPING_UP,
    FSM_MOT_RUNNING,
    FSM_MOT_RAMPING_DOWN,
    FSM_MOT_FAULT,
    FSM_MOT_BRAKING
} fsm_mot_state_t;

typedef struct {
    fsm_mot_state_t state;
    int target_speed;
    int current_speed;
    int ramp_rate;
    int fault_code;
    int brake_timer;
    int run_time;
} fsm_mot_context_t;

void fsm_mot_init(fsm_mot_context_t *ctx) {
    ctx->state = FSM_MOT_STOPPED;
    ctx->target_speed = 0;
    ctx->current_speed = 0;
    ctx->ramp_rate = 10;
    ctx->fault_code = 0;
    ctx->brake_timer = 0;
    ctx->run_time = 0;
}

void fsm_mot_start(fsm_mot_context_t *ctx, int speed) {
    if (ctx->state == FSM_MOT_STOPPED && ctx->fault_code == 0) {
        ctx->target_speed = speed;
        ctx->state = FSM_MOT_RAMPING_UP;
    }
}

void fsm_mot_stop(fsm_mot_context_t *ctx) {
    if (ctx->state == FSM_MOT_RUNNING || ctx->state == FSM_MOT_RAMPING_UP) {
        ctx->target_speed = 0;
        ctx->state = FSM_MOT_RAMPING_DOWN;
    }
}

void fsm_mot_tick(fsm_mot_context_t *ctx) {
    switch (ctx->state) {
        case FSM_MOT_RAMPING_UP:
            ctx->current_speed += ctx->ramp_rate;
            if (ctx->current_speed >= ctx->target_speed) {
                ctx->current_speed = ctx->target_speed;
                ctx->state = FSM_MOT_RUNNING;
                ctx->run_time = 0;
            }
            break;
        case FSM_MOT_RUNNING:
            ctx->run_time++;
            break;
        case FSM_MOT_RAMPING_DOWN:
            ctx->current_speed -= ctx->ramp_rate;
            if (ctx->current_speed <= 0) {
                ctx->current_speed = 0;
                ctx->state = FSM_MOT_BRAKING;
                ctx->brake_timer = 5;
            }
            break;
        case FSM_MOT_BRAKING:
            ctx->brake_timer--;
            if (ctx->brake_timer <= 0) {
                ctx->state = FSM_MOT_STOPPED;
            }
            break;
        case FSM_MOT_FAULT:
        case FSM_MOT_STOPPED:
            break;
    }
}

void fsm_mot_fault(fsm_mot_context_t *ctx, int code) {
    ctx->state = FSM_MOT_FAULT;
    ctx->fault_code = code;
    ctx->current_speed = 0;
}

void fsm_mot_reset(fsm_mot_context_t *ctx) {
    if (ctx->state == FSM_MOT_FAULT) {
        ctx->state = FSM_MOT_STOPPED;
        ctx->fault_code = 0;
    }
}

int fsm_mot_test(void) {
    fsm_mot_context_t ctx;
    fsm_mot_init(&ctx);
    fsm_mot_start(&ctx, 100);
    int i;
    for (i = 0; i < 20; i++) fsm_mot_tick(&ctx);
    if (ctx.state != FSM_MOT_RUNNING) return -1;
    fsm_mot_stop(&ctx);
    for (i = 0; i < 30; i++) fsm_mot_tick(&ctx);
    if (ctx.state != FSM_MOT_STOPPED) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1147: motor driver should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1147: empty output");
    assert!(code.contains("fn fsm_mot_init"), "C1147: Should contain fsm_mot_init");
    assert!(code.contains("fn fsm_mot_tick"), "C1147: Should contain fsm_mot_tick");
    Ok(())
}

/// C1148: Battery charger FSM with multi-stage charging
#[test]
fn c1148_battery_charger() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef enum {
    FSM_BAT_IDLE,
    FSM_BAT_TRICKLE,
    FSM_BAT_CC,
    FSM_BAT_CV,
    FSM_BAT_FULL,
    FSM_BAT_ERROR
} fsm_bat_state_t;

typedef struct {
    fsm_bat_state_t state;
    float voltage;
    float current;
    float charge_pct;
    float max_voltage;
    float trickle_threshold;
    int error_code;
    int charge_time;
} fsm_bat_context_t;

void fsm_bat_init(fsm_bat_context_t *ctx) {
    ctx->state = FSM_BAT_IDLE;
    ctx->voltage = 0.0f;
    ctx->current = 0.0f;
    ctx->charge_pct = 0.0f;
    ctx->max_voltage = 4.2f;
    ctx->trickle_threshold = 3.0f;
    ctx->error_code = 0;
    ctx->charge_time = 0;
}

void fsm_bat_plug_in(fsm_bat_context_t *ctx, float voltage) {
    ctx->voltage = voltage;
    if (ctx->state == FSM_BAT_IDLE) {
        if (voltage < ctx->trickle_threshold)
            ctx->state = FSM_BAT_TRICKLE;
        else
            ctx->state = FSM_BAT_CC;
    }
}

void fsm_bat_tick(fsm_bat_context_t *ctx) {
    ctx->charge_time++;
    switch (ctx->state) {
        case FSM_BAT_TRICKLE:
            ctx->charge_pct += 0.1f;
            ctx->voltage += 0.01f;
            if (ctx->voltage >= ctx->trickle_threshold)
                ctx->state = FSM_BAT_CC;
            break;
        case FSM_BAT_CC:
            ctx->charge_pct += 1.0f;
            ctx->voltage += 0.02f;
            if (ctx->charge_pct >= 80.0f)
                ctx->state = FSM_BAT_CV;
            break;
        case FSM_BAT_CV:
            ctx->charge_pct += 0.5f;
            ctx->current -= 0.01f;
            if (ctx->charge_pct >= 100.0f) {
                ctx->charge_pct = 100.0f;
                ctx->state = FSM_BAT_FULL;
            }
            break;
        case FSM_BAT_FULL:
        case FSM_BAT_IDLE:
        case FSM_BAT_ERROR:
            break;
    }
    if (ctx->voltage > ctx->max_voltage + 0.5f) {
        ctx->state = FSM_BAT_ERROR;
        ctx->error_code = 1;
    }
}

void fsm_bat_unplug(fsm_bat_context_t *ctx) {
    ctx->state = FSM_BAT_IDLE;
    ctx->current = 0.0f;
}

int fsm_bat_test(void) {
    fsm_bat_context_t ctx;
    fsm_bat_init(&ctx);
    fsm_bat_plug_in(&ctx, 3.5f);
    if (ctx.state != FSM_BAT_CC) return -1;
    int i;
    for (i = 0; i < 200; i++) fsm_bat_tick(&ctx);
    if (ctx.state != FSM_BAT_FULL) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1148: battery charger should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1148: empty output");
    assert!(code.contains("fn fsm_bat_init"), "C1148: Should contain fsm_bat_init");
    assert!(code.contains("fn fsm_bat_tick"), "C1148: Should contain fsm_bat_tick");
    Ok(())
}

/// C1149: Thermostat with hysteresis, modes, and scheduling
#[test]
fn c1149_thermostat() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef enum {
    FSM_TH_OFF,
    FSM_TH_HEATING,
    FSM_TH_COOLING,
    FSM_TH_IDLE,
    FSM_TH_FAN_ONLY,
    FSM_TH_EMERGENCY
} fsm_th_state_t;

typedef enum {
    FSM_TH_MODE_HEAT,
    FSM_TH_MODE_COOL,
    FSM_TH_MODE_AUTO
} fsm_th_mode_t;

typedef struct {
    fsm_th_state_t state;
    fsm_th_mode_t mode;
    float target_temp;
    float current_temp;
    float hysteresis;
    int fan_on;
    int compressor_on;
    int run_time;
    float temp_max;
} fsm_th_context_t;

void fsm_th_init(fsm_th_context_t *ctx) {
    ctx->state = FSM_TH_OFF;
    ctx->mode = FSM_TH_MODE_AUTO;
    ctx->target_temp = 22.0f;
    ctx->current_temp = 20.0f;
    ctx->hysteresis = 1.0f;
    ctx->fan_on = 0;
    ctx->compressor_on = 0;
    ctx->run_time = 0;
    ctx->temp_max = 40.0f;
}

void fsm_th_power_on(fsm_th_context_t *ctx) {
    if (ctx->state == FSM_TH_OFF) ctx->state = FSM_TH_IDLE;
}

void fsm_th_power_off(fsm_th_context_t *ctx) {
    ctx->state = FSM_TH_OFF;
    ctx->fan_on = 0;
    ctx->compressor_on = 0;
}

void fsm_th_update(fsm_th_context_t *ctx, float temp) {
    ctx->current_temp = temp;
    if (ctx->state == FSM_TH_OFF) return;

    if (temp > ctx->temp_max) {
        ctx->state = FSM_TH_EMERGENCY;
        ctx->fan_on = 1;
        ctx->compressor_on = 1;
        return;
    }

    if (ctx->mode == FSM_TH_MODE_HEAT || ctx->mode == FSM_TH_MODE_AUTO) {
        if (temp < ctx->target_temp - ctx->hysteresis) {
            ctx->state = FSM_TH_HEATING;
            ctx->fan_on = 1;
            ctx->compressor_on = 0;
            return;
        }
    }

    if (ctx->mode == FSM_TH_MODE_COOL || ctx->mode == FSM_TH_MODE_AUTO) {
        if (temp > ctx->target_temp + ctx->hysteresis) {
            ctx->state = FSM_TH_COOLING;
            ctx->fan_on = 1;
            ctx->compressor_on = 1;
            return;
        }
    }

    ctx->state = FSM_TH_IDLE;
    ctx->fan_on = 0;
    ctx->compressor_on = 0;
}

void fsm_th_tick(fsm_th_context_t *ctx) {
    if (ctx->state == FSM_TH_HEATING || ctx->state == FSM_TH_COOLING) {
        ctx->run_time++;
    }
}

int fsm_th_test(void) {
    fsm_th_context_t ctx;
    fsm_th_init(&ctx);
    fsm_th_power_on(&ctx);
    fsm_th_update(&ctx, 18.0f);
    if (ctx.state != FSM_TH_HEATING) return -1;
    fsm_th_update(&ctx, 22.5f);
    if (ctx.state != FSM_TH_IDLE) return -2;
    fsm_th_update(&ctx, 24.0f);
    if (ctx.state != FSM_TH_COOLING) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1149: thermostat should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1149: empty output");
    assert!(code.contains("fn fsm_th_init"), "C1149: Should contain fsm_th_init");
    assert!(code.contains("fn fsm_th_update"), "C1149: Should contain fsm_th_update");
    Ok(())
}

/// C1150: Conveyor belt controller with sensors and emergency stop
#[test]
fn c1150_conveyor_belt() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef enum {
    FSM_CV_STOPPED,
    FSM_CV_STARTING,
    FSM_CV_RUNNING,
    FSM_CV_STOPPING,
    FSM_CV_JAMMED,
    FSM_CV_EMERGENCY
} fsm_cv_state_t;

typedef struct {
    fsm_cv_state_t state;
    int speed;
    int target_speed;
    int ramp_rate;
    int item_count;
    int jam_sensor;
    int emergency;
    int total_items;
    int fault_count;
} fsm_cv_context_t;

void fsm_cv_init(fsm_cv_context_t *ctx) {
    ctx->state = FSM_CV_STOPPED;
    ctx->speed = 0;
    ctx->target_speed = 100;
    ctx->ramp_rate = 5;
    ctx->item_count = 0;
    ctx->jam_sensor = 0;
    ctx->emergency = 0;
    ctx->total_items = 0;
    ctx->fault_count = 0;
}

void fsm_cv_start(fsm_cv_context_t *ctx) {
    if (ctx->state == FSM_CV_STOPPED && !ctx->emergency) {
        ctx->state = FSM_CV_STARTING;
    }
}

void fsm_cv_stop(fsm_cv_context_t *ctx) {
    if (ctx->state == FSM_CV_RUNNING || ctx->state == FSM_CV_STARTING) {
        ctx->state = FSM_CV_STOPPING;
    }
}

void fsm_cv_emergency_stop(fsm_cv_context_t *ctx) {
    ctx->state = FSM_CV_EMERGENCY;
    ctx->speed = 0;
    ctx->emergency = 1;
}

void fsm_cv_clear_fault(fsm_cv_context_t *ctx) {
    if (ctx->state == FSM_CV_JAMMED || ctx->state == FSM_CV_EMERGENCY) {
        ctx->state = FSM_CV_STOPPED;
        ctx->jam_sensor = 0;
        ctx->emergency = 0;
    }
}

void fsm_cv_tick(fsm_cv_context_t *ctx) {
    if (ctx->jam_sensor && ctx->state == FSM_CV_RUNNING) {
        ctx->state = FSM_CV_JAMMED;
        ctx->speed = 0;
        ctx->fault_count++;
        return;
    }

    switch (ctx->state) {
        case FSM_CV_STARTING:
            ctx->speed += ctx->ramp_rate;
            if (ctx->speed >= ctx->target_speed) {
                ctx->speed = ctx->target_speed;
                ctx->state = FSM_CV_RUNNING;
            }
            break;
        case FSM_CV_RUNNING:
            ctx->item_count++;
            ctx->total_items++;
            break;
        case FSM_CV_STOPPING:
            ctx->speed -= ctx->ramp_rate;
            if (ctx->speed <= 0) {
                ctx->speed = 0;
                ctx->state = FSM_CV_STOPPED;
            }
            break;
        case FSM_CV_STOPPED:
        case FSM_CV_JAMMED:
        case FSM_CV_EMERGENCY:
            break;
    }
}

int fsm_cv_test(void) {
    fsm_cv_context_t ctx;
    fsm_cv_init(&ctx);
    fsm_cv_start(&ctx);
    int i;
    for (i = 0; i < 30; i++) fsm_cv_tick(&ctx);
    if (ctx.state != FSM_CV_RUNNING) return -1;
    if (ctx.total_items < 1) return -2;
    fsm_cv_stop(&ctx);
    for (i = 0; i < 30; i++) fsm_cv_tick(&ctx);
    if (ctx.state != FSM_CV_STOPPED) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1150: conveyor belt should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1150: empty output");
    assert!(code.contains("fn fsm_cv_init"), "C1150: Should contain fsm_cv_init");
    assert!(code.contains("fn fsm_cv_tick"), "C1150: Should contain fsm_cv_tick");
    Ok(())
}
