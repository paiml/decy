//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1126-C1150: State Machines -- finite state machines, automatons, and
//! stateful protocol implementations in C.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world state machine patterns commonly found in
//! embedded systems, protocol stacks, game engines, parsers, and industrial
//! controllers -- all expressed as valid C99.
//!
//! Organization:
//! - C1126-C1130: Classic FSMs (traffic light, vending machine, elevator, door lock, turnstile)
//! - C1131-C1135: Parser state machines (JSON tokenizer, CSV parser, XML tag parser, INI file parser, ANSI escape parser)
//! - C1136-C1140: Protocol state machines (TCP state machine, HTTP/1.1 pipeline, DHCP client, TLS handshake sim, PPP link)
//! - C1141-C1145: Game/UI state machines (game menu, animation controller, dialog tree, input combo, screen manager)
//! - C1146-C1150: Industrial state machines (PID controller, motor driver, battery charger, thermostat, conveyor belt)

use decy_core::transpile;

// ============================================================================
// C1126-C1130: Classic FSMs
// ============================================================================

/// C1126: Traffic light controller with timed state transitions
#[test]
fn c1126_traffic_light_controller() {
    let c_code = r#"
typedef enum { FSM_RED, FSM_GREEN, FSM_YELLOW } fsm_light_state;

typedef struct {
    fsm_light_state state;
    int timer;
} fsm_traffic_t;

void fsm_traffic_init(fsm_traffic_t *t) {
    t->state = FSM_RED;
    t->timer = 0;
}

void fsm_traffic_tick(fsm_traffic_t *t) {
    t->timer++;
    if (t->state == FSM_RED && t->timer >= 30) {
        t->state = FSM_GREEN; t->timer = 0;
    } else if (t->state == FSM_GREEN && t->timer >= 25) {
        t->state = FSM_YELLOW; t->timer = 0;
    } else if (t->state == FSM_YELLOW && t->timer >= 5) {
        t->state = FSM_RED; t->timer = 0;
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1126 traffic light failed: {:?}", result.err());
}

/// C1127: Coin-operated vending machine FSM
#[test]
fn c1127_vending_machine() {
    let c_code = r#"
typedef enum { FSM_IDLE, FSM_COIN_IN, FSM_SELECTING, FSM_DISPENSING } fsm_vend_state;

typedef struct {
    fsm_vend_state state;
    int balance;
    int item_price;
} fsm_vending_t;

void fsm_vend_init(fsm_vending_t *v) {
    v->state = FSM_IDLE;
    v->balance = 0;
    v->item_price = 100;
}

void fsm_vend_insert_coin(fsm_vending_t *v, int cents) {
    if (v->state == FSM_IDLE || v->state == FSM_COIN_IN) {
        v->balance += cents;
        v->state = FSM_COIN_IN;
        if (v->balance >= v->item_price) {
            v->state = FSM_SELECTING;
        }
    }
}

int fsm_vend_select(fsm_vending_t *v) {
    if (v->state == FSM_SELECTING) {
        v->state = FSM_DISPENSING;
        v->balance -= v->item_price;
        return 1;
    }
    return 0;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1127 vending machine failed: {:?}", result.err());
}

/// C1128: Elevator controller with floor tracking
#[test]
fn c1128_elevator_controller() {
    let c_code = r#"
typedef enum { FSM_ELEV_IDLE, FSM_ELEV_UP, FSM_ELEV_DOWN, FSM_ELEV_DOOR_OPEN } fsm_elev_state;

typedef struct {
    fsm_elev_state state;
    int current_floor;
    int target_floor;
    int door_timer;
} fsm_elevator_t;

void fsm_elev_init(fsm_elevator_t *e) {
    e->state = FSM_ELEV_IDLE;
    e->current_floor = 0;
    e->target_floor = 0;
    e->door_timer = 0;
}

void fsm_elev_request(fsm_elevator_t *e, int floor) {
    e->target_floor = floor;
    if (floor > e->current_floor) e->state = FSM_ELEV_UP;
    else if (floor < e->current_floor) e->state = FSM_ELEV_DOWN;
    else { e->state = FSM_ELEV_DOOR_OPEN; e->door_timer = 5; }
}

void fsm_elev_tick(fsm_elevator_t *e) {
    if (e->state == FSM_ELEV_UP) {
        e->current_floor++;
        if (e->current_floor >= e->target_floor) {
            e->state = FSM_ELEV_DOOR_OPEN; e->door_timer = 5;
        }
    } else if (e->state == FSM_ELEV_DOWN) {
        e->current_floor--;
        if (e->current_floor <= e->target_floor) {
            e->state = FSM_ELEV_DOOR_OPEN; e->door_timer = 5;
        }
    } else if (e->state == FSM_ELEV_DOOR_OPEN) {
        e->door_timer--;
        if (e->door_timer <= 0) e->state = FSM_ELEV_IDLE;
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1128 elevator controller failed: {:?}", result.err());
}

/// C1129: Digital door lock with code entry FSM
#[test]
fn c1129_door_lock_fsm() {
    let c_code = r#"
typedef enum { FSM_LOCKED, FSM_DIGIT1, FSM_DIGIT2, FSM_DIGIT3, FSM_UNLOCKED } fsm_lock_state;

typedef struct {
    fsm_lock_state state;
    int code[4];
    int attempts;
} fsm_doorlock_t;

void fsm_lock_init(fsm_doorlock_t *d) {
    d->state = FSM_LOCKED;
    d->code[0] = 1; d->code[1] = 2;
    d->code[2] = 3; d->code[3] = 4;
    d->attempts = 0;
}

void fsm_lock_enter(fsm_doorlock_t *d, int digit) {
    if (d->state == FSM_LOCKED && digit == d->code[0]) {
        d->state = FSM_DIGIT1;
    } else if (d->state == FSM_DIGIT1 && digit == d->code[1]) {
        d->state = FSM_DIGIT2;
    } else if (d->state == FSM_DIGIT2 && digit == d->code[2]) {
        d->state = FSM_DIGIT3;
    } else if (d->state == FSM_DIGIT3 && digit == d->code[3]) {
        d->state = FSM_UNLOCKED;
    } else {
        d->state = FSM_LOCKED;
        d->attempts++;
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1129 door lock failed: {:?}", result.err());
}

/// C1130: Turnstile FSM with coin and push events
#[test]
fn c1130_turnstile_fsm() {
    let c_code = r#"
typedef enum { FSM_TURN_LOCKED, FSM_TURN_UNLOCKED } fsm_turn_state;

typedef struct {
    fsm_turn_state state;
    int coins_collected;
    int people_passed;
} fsm_turnstile_t;

void fsm_turn_init(fsm_turnstile_t *t) {
    t->state = FSM_TURN_LOCKED;
    t->coins_collected = 0;
    t->people_passed = 0;
}

void fsm_turn_coin(fsm_turnstile_t *t) {
    t->coins_collected++;
    t->state = FSM_TURN_UNLOCKED;
}

void fsm_turn_push(fsm_turnstile_t *t) {
    if (t->state == FSM_TURN_UNLOCKED) {
        t->people_passed++;
        t->state = FSM_TURN_LOCKED;
    }
}

int fsm_turn_is_locked(fsm_turnstile_t *t) {
    return t->state == FSM_TURN_LOCKED;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1130 turnstile failed: {:?}", result.err());
}

// ============================================================================
// C1131-C1135: Parser State Machines
// ============================================================================

/// C1131: JSON tokenizer state machine
#[test]
fn c1131_json_tokenizer() {
    let c_code = r#"
typedef enum {
    FSM_JSON_START, FSM_JSON_STRING, FSM_JSON_NUMBER,
    FSM_JSON_ESCAPE, FSM_JSON_DONE, FSM_JSON_ERROR
} fsm_json_state;

typedef struct {
    fsm_json_state state;
    int pos;
    int token_start;
} fsm_json_tok_t;

void fsm_json_init(fsm_json_tok_t *t) {
    t->state = FSM_JSON_START;
    t->pos = 0;
    t->token_start = 0;
}

void fsm_json_feed(fsm_json_tok_t *t, char c) {
    if (t->state == FSM_JSON_START) {
        if (c == '"') { t->state = FSM_JSON_STRING; t->token_start = t->pos; }
        else if (c >= '0' && c <= '9') { t->state = FSM_JSON_NUMBER; t->token_start = t->pos; }
        else if (c == '{' || c == '}' || c == '[' || c == ']' || c == ':' || c == ',') { t->state = FSM_JSON_DONE; }
    } else if (t->state == FSM_JSON_STRING) {
        if (c == '\\') t->state = FSM_JSON_ESCAPE;
        else if (c == '"') t->state = FSM_JSON_DONE;
    } else if (t->state == FSM_JSON_ESCAPE) {
        t->state = FSM_JSON_STRING;
    } else if (t->state == FSM_JSON_NUMBER) {
        if (c < '0' || c > '9') t->state = FSM_JSON_DONE;
    }
    t->pos++;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1131 JSON tokenizer failed: {:?}", result.err());
}

/// C1132: CSV field parser with quoting support
#[test]
fn c1132_csv_parser() {
    let c_code = r#"
typedef enum { FSM_CSV_FIELD, FSM_CSV_QUOTED, FSM_CSV_QUOTE_END, FSM_CSV_DELIM } fsm_csv_state;

typedef struct {
    fsm_csv_state state;
    int field_count;
    int row_count;
    int field_len;
} fsm_csv_t;

void fsm_csv_init(fsm_csv_t *p) {
    p->state = FSM_CSV_FIELD;
    p->field_count = 0;
    p->row_count = 0;
    p->field_len = 0;
}

void fsm_csv_feed(fsm_csv_t *p, char c) {
    if (p->state == FSM_CSV_FIELD) {
        if (c == '"') { p->state = FSM_CSV_QUOTED; }
        else if (c == ',') { p->field_count++; p->field_len = 0; }
        else if (c == '\n') { p->field_count++; p->row_count++; p->field_len = 0; }
        else { p->field_len++; }
    } else if (p->state == FSM_CSV_QUOTED) {
        if (c == '"') p->state = FSM_CSV_QUOTE_END;
        else p->field_len++;
    } else if (p->state == FSM_CSV_QUOTE_END) {
        if (c == '"') { p->state = FSM_CSV_QUOTED; p->field_len++; }
        else if (c == ',') { p->field_count++; p->state = FSM_CSV_FIELD; p->field_len = 0; }
        else { p->state = FSM_CSV_FIELD; }
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1132 CSV parser failed: {:?}", result.err());
}

/// C1133: XML tag parser state machine
#[test]
fn c1133_xml_tag_parser() {
    let c_code = r#"
typedef enum {
    FSM_XML_TEXT, FSM_XML_TAG_OPEN, FSM_XML_TAG_NAME,
    FSM_XML_TAG_CLOSE, FSM_XML_ATTR
} fsm_xml_state;

typedef struct {
    fsm_xml_state state;
    int depth;
    int tag_count;
} fsm_xml_t;

void fsm_xml_init(fsm_xml_t *x) {
    x->state = FSM_XML_TEXT;
    x->depth = 0;
    x->tag_count = 0;
}

void fsm_xml_feed(fsm_xml_t *x, char c) {
    if (x->state == FSM_XML_TEXT) {
        if (c == '<') x->state = FSM_XML_TAG_OPEN;
    } else if (x->state == FSM_XML_TAG_OPEN) {
        if (c == '/') { x->state = FSM_XML_TAG_CLOSE; }
        else { x->state = FSM_XML_TAG_NAME; x->tag_count++; }
    } else if (x->state == FSM_XML_TAG_NAME) {
        if (c == '>') { x->depth++; x->state = FSM_XML_TEXT; }
        else if (c == ' ') x->state = FSM_XML_ATTR;
    } else if (x->state == FSM_XML_ATTR) {
        if (c == '>') { x->depth++; x->state = FSM_XML_TEXT; }
    } else if (x->state == FSM_XML_TAG_CLOSE) {
        if (c == '>') { x->depth--; x->state = FSM_XML_TEXT; }
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1133 XML tag parser failed: {:?}", result.err());
}

/// C1134: INI file parser with section/key/value states
#[test]
fn c1134_ini_file_parser() {
    let c_code = r#"
typedef enum {
    FSM_INI_START, FSM_INI_SECTION, FSM_INI_KEY,
    FSM_INI_VALUE, FSM_INI_COMMENT
} fsm_ini_state;

typedef struct {
    fsm_ini_state state;
    int section_count;
    int key_count;
} fsm_ini_t;

void fsm_ini_init(fsm_ini_t *p) {
    p->state = FSM_INI_START;
    p->section_count = 0;
    p->key_count = 0;
}

void fsm_ini_feed(fsm_ini_t *p, char c) {
    if (p->state == FSM_INI_START) {
        if (c == '[') p->state = FSM_INI_SECTION;
        else if (c == ';' || c == '#') p->state = FSM_INI_COMMENT;
        else if (c != '\n' && c != ' ') p->state = FSM_INI_KEY;
    } else if (p->state == FSM_INI_SECTION) {
        if (c == ']') { p->section_count++; p->state = FSM_INI_START; }
    } else if (p->state == FSM_INI_KEY) {
        if (c == '=') p->state = FSM_INI_VALUE;
    } else if (p->state == FSM_INI_VALUE) {
        if (c == '\n') { p->key_count++; p->state = FSM_INI_START; }
    } else if (p->state == FSM_INI_COMMENT) {
        if (c == '\n') p->state = FSM_INI_START;
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1134 INI file parser failed: {:?}", result.err());
}

/// C1135: ANSI escape sequence parser
#[test]
fn c1135_ansi_escape_parser() {
    let c_code = r#"
typedef enum {
    FSM_ANSI_NORMAL, FSM_ANSI_ESC, FSM_ANSI_CSI,
    FSM_ANSI_PARAM, FSM_ANSI_DONE
} fsm_ansi_state;

typedef struct {
    fsm_ansi_state state;
    int params[8];
    int param_count;
    char final_char;
} fsm_ansi_t;

void fsm_ansi_init(fsm_ansi_t *a) {
    a->state = FSM_ANSI_NORMAL;
    a->param_count = 0;
    a->final_char = 0;
}

void fsm_ansi_feed(fsm_ansi_t *a, char c) {
    if (a->state == FSM_ANSI_NORMAL) {
        if (c == 27) a->state = FSM_ANSI_ESC;
    } else if (a->state == FSM_ANSI_ESC) {
        if (c == '[') { a->state = FSM_ANSI_CSI; a->param_count = 0; }
        else a->state = FSM_ANSI_NORMAL;
    } else if (a->state == FSM_ANSI_CSI || a->state == FSM_ANSI_PARAM) {
        if (c >= '0' && c <= '9') {
            a->state = FSM_ANSI_PARAM;
        } else if (c == ';') {
            if (a->param_count < 8) a->param_count++;
        } else {
            a->final_char = c;
            a->state = FSM_ANSI_DONE;
        }
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1135 ANSI escape parser failed: {:?}", result.err());
}

// ============================================================================
// C1136-C1140: Protocol State Machines
// ============================================================================

/// C1136: Simplified TCP connection state machine
#[test]
fn c1136_tcp_state_machine() {
    let c_code = r#"
typedef enum {
    FSM_TCP_CLOSED, FSM_TCP_LISTEN, FSM_TCP_SYN_SENT,
    FSM_TCP_SYN_RCVD, FSM_TCP_ESTABLISHED, FSM_TCP_FIN_WAIT
} fsm_tcp_state;

typedef struct {
    fsm_tcp_state state;
    int seq_num;
    int ack_num;
} fsm_tcp_t;

void fsm_tcp_init(fsm_tcp_t *t) {
    t->state = FSM_TCP_CLOSED;
    t->seq_num = 0;
    t->ack_num = 0;
}

void fsm_tcp_open(fsm_tcp_t *t, int passive) {
    if (t->state == FSM_TCP_CLOSED) {
        if (passive) t->state = FSM_TCP_LISTEN;
        else { t->state = FSM_TCP_SYN_SENT; t->seq_num = 1000; }
    }
}

void fsm_tcp_recv_syn(fsm_tcp_t *t, int seq) {
    if (t->state == FSM_TCP_LISTEN) {
        t->ack_num = seq + 1;
        t->state = FSM_TCP_SYN_RCVD;
    }
}

void fsm_tcp_recv_ack(fsm_tcp_t *t) {
    if (t->state == FSM_TCP_SYN_RCVD) t->state = FSM_TCP_ESTABLISHED;
    else if (t->state == FSM_TCP_SYN_SENT) t->state = FSM_TCP_ESTABLISHED;
}

void fsm_tcp_close(fsm_tcp_t *t) {
    if (t->state == FSM_TCP_ESTABLISHED) t->state = FSM_TCP_FIN_WAIT;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1136 TCP state machine failed: {:?}", result.err());
}

/// C1137: HTTP/1.1 request pipeline state machine
#[test]
fn c1137_http_pipeline() {
    let c_code = r#"
typedef enum {
    FSM_HTTP_METHOD, FSM_HTTP_URI, FSM_HTTP_VERSION,
    FSM_HTTP_HEADER_KEY, FSM_HTTP_HEADER_VAL, FSM_HTTP_BODY
} fsm_http_state;

typedef struct {
    fsm_http_state state;
    int content_length;
    int body_read;
    int header_count;
} fsm_http_t;

void fsm_http_init(fsm_http_t *h) {
    h->state = FSM_HTTP_METHOD;
    h->content_length = 0;
    h->body_read = 0;
    h->header_count = 0;
}

void fsm_http_feed(fsm_http_t *h, char c) {
    if (h->state == FSM_HTTP_METHOD) {
        if (c == ' ') h->state = FSM_HTTP_URI;
    } else if (h->state == FSM_HTTP_URI) {
        if (c == ' ') h->state = FSM_HTTP_VERSION;
    } else if (h->state == FSM_HTTP_VERSION) {
        if (c == '\n') h->state = FSM_HTTP_HEADER_KEY;
    } else if (h->state == FSM_HTTP_HEADER_KEY) {
        if (c == ':') h->state = FSM_HTTP_HEADER_VAL;
        else if (c == '\n') h->state = FSM_HTTP_BODY;
    } else if (h->state == FSM_HTTP_HEADER_VAL) {
        if (c == '\n') { h->header_count++; h->state = FSM_HTTP_HEADER_KEY; }
    } else if (h->state == FSM_HTTP_BODY) {
        h->body_read++;
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1137 HTTP pipeline failed: {:?}", result.err());
}

/// C1138: DHCP client state machine
#[test]
fn c1138_dhcp_client() {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef enum {
    FSM_DHCP_INIT, FSM_DHCP_SELECTING, FSM_DHCP_REQUESTING,
    FSM_DHCP_BOUND, FSM_DHCP_RENEWING
} fsm_dhcp_state;

typedef struct {
    fsm_dhcp_state state;
    uint32_t offered_ip;
    uint32_t server_ip;
    int lease_time;
    int timer;
} fsm_dhcp_t;

void fsm_dhcp_init(fsm_dhcp_t *d) {
    d->state = FSM_DHCP_INIT;
    d->offered_ip = 0;
    d->server_ip = 0;
    d->lease_time = 0;
    d->timer = 0;
}

void fsm_dhcp_discover(fsm_dhcp_t *d) {
    if (d->state == FSM_DHCP_INIT) d->state = FSM_DHCP_SELECTING;
}

void fsm_dhcp_offer(fsm_dhcp_t *d, uint32_t ip, uint32_t server) {
    if (d->state == FSM_DHCP_SELECTING) {
        d->offered_ip = ip;
        d->server_ip = server;
        d->state = FSM_DHCP_REQUESTING;
    }
}

void fsm_dhcp_ack(fsm_dhcp_t *d, int lease) {
    if (d->state == FSM_DHCP_REQUESTING || d->state == FSM_DHCP_RENEWING) {
        d->lease_time = lease;
        d->timer = 0;
        d->state = FSM_DHCP_BOUND;
    }
}

void fsm_dhcp_tick(fsm_dhcp_t *d) {
    if (d->state == FSM_DHCP_BOUND) {
        d->timer++;
        if (d->timer >= d->lease_time / 2) d->state = FSM_DHCP_RENEWING;
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1138 DHCP client failed: {:?}", result.err());
}

/// C1139: TLS handshake simulation state machine
#[test]
fn c1139_tls_handshake() {
    let c_code = r#"
typedef enum {
    FSM_TLS_IDLE, FSM_TLS_CLIENT_HELLO, FSM_TLS_SERVER_HELLO,
    FSM_TLS_KEY_EXCHANGE, FSM_TLS_FINISHED, FSM_TLS_ACTIVE
} fsm_tls_state;

typedef struct {
    fsm_tls_state state;
    int cipher_suite;
    int version;
} fsm_tls_t;

void fsm_tls_init(fsm_tls_t *t) {
    t->state = FSM_TLS_IDLE;
    t->cipher_suite = 0;
    t->version = 0;
}

void fsm_tls_client_hello(fsm_tls_t *t, int version) {
    if (t->state == FSM_TLS_IDLE) {
        t->version = version;
        t->state = FSM_TLS_CLIENT_HELLO;
    }
}

void fsm_tls_server_hello(fsm_tls_t *t, int cipher) {
    if (t->state == FSM_TLS_CLIENT_HELLO) {
        t->cipher_suite = cipher;
        t->state = FSM_TLS_SERVER_HELLO;
    }
}

void fsm_tls_key_exchange(fsm_tls_t *t) {
    if (t->state == FSM_TLS_SERVER_HELLO) t->state = FSM_TLS_KEY_EXCHANGE;
}

void fsm_tls_finish(fsm_tls_t *t) {
    if (t->state == FSM_TLS_KEY_EXCHANGE) t->state = FSM_TLS_FINISHED;
}

void fsm_tls_activate(fsm_tls_t *t) {
    if (t->state == FSM_TLS_FINISHED) t->state = FSM_TLS_ACTIVE;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1139 TLS handshake failed: {:?}", result.err());
}

/// C1140: PPP link control protocol state machine
#[test]
fn c1140_ppp_link_control() {
    let c_code = r#"
typedef enum {
    FSM_PPP_DEAD, FSM_PPP_ESTABLISH, FSM_PPP_AUTH,
    FSM_PPP_NETWORK, FSM_PPP_OPEN, FSM_PPP_TERMINATE
} fsm_ppp_state;

typedef struct {
    fsm_ppp_state state;
    int retries;
    int max_retries;
} fsm_ppp_t;

void fsm_ppp_init(fsm_ppp_t *p) {
    p->state = FSM_PPP_DEAD;
    p->retries = 0;
    p->max_retries = 5;
}

void fsm_ppp_up(fsm_ppp_t *p) {
    if (p->state == FSM_PPP_DEAD) {
        p->state = FSM_PPP_ESTABLISH;
        p->retries = 0;
    }
}

void fsm_ppp_auth_ok(fsm_ppp_t *p) {
    if (p->state == FSM_PPP_ESTABLISH) p->state = FSM_PPP_AUTH;
}

void fsm_ppp_net_ok(fsm_ppp_t *p) {
    if (p->state == FSM_PPP_AUTH) p->state = FSM_PPP_NETWORK;
}

void fsm_ppp_opened(fsm_ppp_t *p) {
    if (p->state == FSM_PPP_NETWORK) p->state = FSM_PPP_OPEN;
}

void fsm_ppp_down(fsm_ppp_t *p) {
    p->state = FSM_PPP_TERMINATE;
    p->retries++;
    if (p->retries >= p->max_retries) p->state = FSM_PPP_DEAD;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1140 PPP link control failed: {:?}", result.err());
}

// ============================================================================
// C1141-C1145: Game/UI State Machines
// ============================================================================

/// C1141: Game menu navigation FSM
#[test]
fn c1141_game_menu_fsm() {
    let c_code = r#"
typedef enum {
    FSM_MENU_MAIN, FSM_MENU_PLAY, FSM_MENU_OPTIONS,
    FSM_MENU_CREDITS, FSM_MENU_QUIT
} fsm_menu_state;

typedef struct {
    fsm_menu_state state;
    int selected_item;
    int num_items;
} fsm_menu_t;

void fsm_menu_init(fsm_menu_t *m) {
    m->state = FSM_MENU_MAIN;
    m->selected_item = 0;
    m->num_items = 4;
}

void fsm_menu_navigate(fsm_menu_t *m, int direction) {
    m->selected_item += direction;
    if (m->selected_item < 0) m->selected_item = m->num_items - 1;
    if (m->selected_item >= m->num_items) m->selected_item = 0;
}

void fsm_menu_select(fsm_menu_t *m) {
    if (m->state == FSM_MENU_MAIN) {
        if (m->selected_item == 0) m->state = FSM_MENU_PLAY;
        else if (m->selected_item == 1) m->state = FSM_MENU_OPTIONS;
        else if (m->selected_item == 2) m->state = FSM_MENU_CREDITS;
        else if (m->selected_item == 3) m->state = FSM_MENU_QUIT;
    }
}

void fsm_menu_back(fsm_menu_t *m) {
    if (m->state != FSM_MENU_MAIN && m->state != FSM_MENU_QUIT) {
        m->state = FSM_MENU_MAIN;
        m->selected_item = 0;
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1141 game menu failed: {:?}", result.err());
}

/// C1142: Sprite animation controller FSM
#[test]
fn c1142_animation_controller() {
    let c_code = r#"
typedef enum {
    FSM_ANIM_IDLE, FSM_ANIM_WALK, FSM_ANIM_RUN,
    FSM_ANIM_JUMP, FSM_ANIM_FALL
} fsm_anim_state;

typedef struct {
    fsm_anim_state state;
    int frame;
    int frame_count;
    int speed;
} fsm_anim_t;

void fsm_anim_init(fsm_anim_t *a) {
    a->state = FSM_ANIM_IDLE;
    a->frame = 0;
    a->frame_count = 4;
    a->speed = 1;
}

void fsm_anim_set_moving(fsm_anim_t *a, int velocity) {
    if (velocity == 0) { a->state = FSM_ANIM_IDLE; a->speed = 1; }
    else if (velocity < 5) { a->state = FSM_ANIM_WALK; a->speed = 2; }
    else { a->state = FSM_ANIM_RUN; a->speed = 3; }
    a->frame = 0;
}

void fsm_anim_set_airborne(fsm_anim_t *a, int vy) {
    if (vy < 0) a->state = FSM_ANIM_JUMP;
    else a->state = FSM_ANIM_FALL;
    a->frame = 0;
}

void fsm_anim_tick(fsm_anim_t *a) {
    a->frame += a->speed;
    if (a->frame >= a->frame_count * 10) a->frame = 0;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1142 animation controller failed: {:?}", result.err());
}

/// C1143: Dialog tree / conversation FSM
#[test]
fn c1143_dialog_tree() {
    let c_code = r#"
#define FSM_MAX_CHOICES 4

typedef struct {
    int node_id;
    int choices[FSM_MAX_CHOICES];
    int choice_count;
    int visited;
} fsm_dialog_node_t;

typedef struct {
    fsm_dialog_node_t nodes[16];
    int node_count;
    int current_node;
} fsm_dialog_t;

void fsm_dialog_init(fsm_dialog_t *d) {
    d->node_count = 0;
    d->current_node = 0;
}

int fsm_dialog_add_node(fsm_dialog_t *d) {
    if (d->node_count >= 16) return -1;
    int id = d->node_count;
    d->nodes[id].node_id = id;
    d->nodes[id].choice_count = 0;
    d->nodes[id].visited = 0;
    d->node_count++;
    return id;
}

void fsm_dialog_add_choice(fsm_dialog_t *d, int from, int to) {
    if (from >= 0 && from < d->node_count) {
        int idx = d->nodes[from].choice_count;
        if (idx < FSM_MAX_CHOICES) {
            d->nodes[from].choices[idx] = to;
            d->nodes[from].choice_count++;
        }
    }
}

int fsm_dialog_choose(fsm_dialog_t *d, int choice) {
    int cur = d->current_node;
    if (choice >= 0 && choice < d->nodes[cur].choice_count) {
        d->nodes[cur].visited = 1;
        d->current_node = d->nodes[cur].choices[choice];
        return d->current_node;
    }
    return -1;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1143 dialog tree failed: {:?}", result.err());
}

/// C1144: Input combo detector (fighting game style)
#[test]
fn c1144_input_combo_detector() {
    let c_code = r#"
typedef enum {
    FSM_COMBO_IDLE, FSM_COMBO_SEQ1, FSM_COMBO_SEQ2,
    FSM_COMBO_SEQ3, FSM_COMBO_TRIGGERED
} fsm_combo_state;

typedef struct {
    fsm_combo_state state;
    int timer;
    int timeout;
    int combo_count;
} fsm_combo_t;

void fsm_combo_init(fsm_combo_t *c) {
    c->state = FSM_COMBO_IDLE;
    c->timer = 0;
    c->timeout = 15;
    c->combo_count = 0;
}

void fsm_combo_input(fsm_combo_t *c, int button) {
    if (c->timer > c->timeout) {
        c->state = FSM_COMBO_IDLE;
        c->timer = 0;
    }
    if (c->state == FSM_COMBO_IDLE && button == 1) {
        c->state = FSM_COMBO_SEQ1; c->timer = 0;
    } else if (c->state == FSM_COMBO_SEQ1 && button == 2) {
        c->state = FSM_COMBO_SEQ2; c->timer = 0;
    } else if (c->state == FSM_COMBO_SEQ2 && button == 3) {
        c->state = FSM_COMBO_SEQ3; c->timer = 0;
    } else if (c->state == FSM_COMBO_SEQ3 && button == 4) {
        c->state = FSM_COMBO_TRIGGERED;
        c->combo_count++;
    } else {
        c->state = FSM_COMBO_IDLE;
    }
}

void fsm_combo_tick(fsm_combo_t *c) {
    c->timer++;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1144 input combo failed: {:?}", result.err());
}

/// C1145: Screen manager with transitions
#[test]
fn c1145_screen_manager() {
    let c_code = r#"
typedef enum {
    FSM_SCREEN_SPLASH, FSM_SCREEN_LOADING, FSM_SCREEN_GAME,
    FSM_SCREEN_PAUSE, FSM_SCREEN_GAMEOVER
} fsm_screen_state;

typedef struct {
    fsm_screen_state state;
    fsm_screen_state prev_state;
    int transition_timer;
    int fade_alpha;
} fsm_screen_t;

void fsm_screen_init(fsm_screen_t *s) {
    s->state = FSM_SCREEN_SPLASH;
    s->prev_state = FSM_SCREEN_SPLASH;
    s->transition_timer = 0;
    s->fade_alpha = 255;
}

void fsm_screen_goto(fsm_screen_t *s, fsm_screen_state next) {
    s->prev_state = s->state;
    s->state = next;
    s->transition_timer = 30;
    s->fade_alpha = 0;
}

void fsm_screen_tick(fsm_screen_t *s) {
    if (s->transition_timer > 0) {
        s->transition_timer--;
        s->fade_alpha = 255 - (s->transition_timer * 255 / 30);
    }
    if (s->state == FSM_SCREEN_SPLASH && s->transition_timer == 0) {
        fsm_screen_goto(s, FSM_SCREEN_LOADING);
    }
}

void fsm_screen_pause(fsm_screen_t *s) {
    if (s->state == FSM_SCREEN_GAME) fsm_screen_goto(s, FSM_SCREEN_PAUSE);
}

void fsm_screen_resume(fsm_screen_t *s) {
    if (s->state == FSM_SCREEN_PAUSE) fsm_screen_goto(s, FSM_SCREEN_GAME);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1145 screen manager failed: {:?}", result.err());
}

// ============================================================================
// C1146-C1150: Industrial State Machines
// ============================================================================

/// C1146: PID controller with mode switching
#[test]
fn c1146_pid_controller() {
    let c_code = r#"
typedef enum {
    FSM_PID_OFF, FSM_PID_MANUAL, FSM_PID_AUTO, FSM_PID_CASCADE
} fsm_pid_mode;

typedef struct {
    fsm_pid_mode mode;
    float setpoint;
    float output;
    float error_sum;
    float last_error;
    float kp;
    float ki;
    float kd;
} fsm_pid_t;

void fsm_pid_init(fsm_pid_t *p) {
    p->mode = FSM_PID_OFF;
    p->setpoint = 0.0f;
    p->output = 0.0f;
    p->error_sum = 0.0f;
    p->last_error = 0.0f;
    p->kp = 1.0f;
    p->ki = 0.1f;
    p->kd = 0.05f;
}

void fsm_pid_set_mode(fsm_pid_t *p, fsm_pid_mode mode) {
    if (mode != p->mode) {
        p->error_sum = 0.0f;
        p->last_error = 0.0f;
        p->mode = mode;
    }
}

float fsm_pid_compute(fsm_pid_t *p, float measurement) {
    if (p->mode == FSM_PID_OFF) return 0.0f;
    if (p->mode == FSM_PID_MANUAL) return p->output;
    float error = p->setpoint - measurement;
    p->error_sum += error;
    float derivative = error - p->last_error;
    p->output = p->kp * error + p->ki * p->error_sum + p->kd * derivative;
    p->last_error = error;
    if (p->output > 100.0f) p->output = 100.0f;
    if (p->output < 0.0f) p->output = 0.0f;
    return p->output;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1146 PID controller failed: {:?}", result.err());
}

/// C1147: Motor driver state machine with ramp control
#[test]
fn c1147_motor_driver() {
    let c_code = r#"
typedef enum {
    FSM_MOT_STOPPED, FSM_MOT_RAMPING_UP, FSM_MOT_RUNNING,
    FSM_MOT_RAMPING_DOWN, FSM_MOT_FAULT
} fsm_motor_state;

typedef struct {
    fsm_motor_state state;
    int target_speed;
    int current_speed;
    int ramp_rate;
    int fault_code;
} fsm_motor_t;

void fsm_motor_init(fsm_motor_t *m) {
    m->state = FSM_MOT_STOPPED;
    m->target_speed = 0;
    m->current_speed = 0;
    m->ramp_rate = 10;
    m->fault_code = 0;
}

void fsm_motor_start(fsm_motor_t *m, int speed) {
    if (m->state == FSM_MOT_STOPPED) {
        m->target_speed = speed;
        m->state = FSM_MOT_RAMPING_UP;
    }
}

void fsm_motor_stop(fsm_motor_t *m) {
    if (m->state == FSM_MOT_RUNNING || m->state == FSM_MOT_RAMPING_UP) {
        m->target_speed = 0;
        m->state = FSM_MOT_RAMPING_DOWN;
    }
}

void fsm_motor_tick(fsm_motor_t *m) {
    if (m->state == FSM_MOT_RAMPING_UP) {
        m->current_speed += m->ramp_rate;
        if (m->current_speed >= m->target_speed) {
            m->current_speed = m->target_speed;
            m->state = FSM_MOT_RUNNING;
        }
    } else if (m->state == FSM_MOT_RAMPING_DOWN) {
        m->current_speed -= m->ramp_rate;
        if (m->current_speed <= 0) {
            m->current_speed = 0;
            m->state = FSM_MOT_STOPPED;
        }
    }
}

void fsm_motor_fault(fsm_motor_t *m, int code) {
    m->state = FSM_MOT_FAULT;
    m->fault_code = code;
    m->current_speed = 0;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1147 motor driver failed: {:?}", result.err());
}

/// C1148: Battery charger FSM with charge stages
#[test]
fn c1148_battery_charger() {
    let c_code = r#"
typedef enum {
    FSM_BAT_IDLE, FSM_BAT_TRICKLE, FSM_BAT_CONSTANT_CURRENT,
    FSM_BAT_CONSTANT_VOLTAGE, FSM_BAT_FULL, FSM_BAT_ERROR
} fsm_battery_state;

typedef struct {
    fsm_battery_state state;
    float voltage;
    float current;
    float charge_pct;
} fsm_battery_t;

void fsm_bat_init(fsm_battery_t *b) {
    b->state = FSM_BAT_IDLE;
    b->voltage = 0.0f;
    b->current = 0.0f;
    b->charge_pct = 0.0f;
}

void fsm_bat_plug_in(fsm_battery_t *b, float voltage) {
    b->voltage = voltage;
    if (b->state == FSM_BAT_IDLE) {
        if (voltage < 3.0f) b->state = FSM_BAT_TRICKLE;
        else b->state = FSM_BAT_CONSTANT_CURRENT;
    }
}

void fsm_bat_tick(fsm_battery_t *b) {
    if (b->state == FSM_BAT_TRICKLE) {
        b->charge_pct += 0.1f;
        if (b->voltage >= 3.0f) b->state = FSM_BAT_CONSTANT_CURRENT;
    } else if (b->state == FSM_BAT_CONSTANT_CURRENT) {
        b->charge_pct += 1.0f;
        if (b->charge_pct >= 80.0f) b->state = FSM_BAT_CONSTANT_VOLTAGE;
    } else if (b->state == FSM_BAT_CONSTANT_VOLTAGE) {
        b->charge_pct += 0.5f;
        if (b->charge_pct >= 100.0f) {
            b->charge_pct = 100.0f;
            b->state = FSM_BAT_FULL;
        }
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1148 battery charger failed: {:?}", result.err());
}

/// C1149: Thermostat with hysteresis and modes
#[test]
fn c1149_thermostat() {
    let c_code = r#"
typedef enum {
    FSM_THERM_OFF, FSM_THERM_HEATING, FSM_THERM_COOLING,
    FSM_THERM_IDLE, FSM_THERM_FAN_ONLY
} fsm_therm_state;

typedef struct {
    fsm_therm_state state;
    float target_temp;
    float current_temp;
    float hysteresis;
    int fan_on;
} fsm_therm_t;

void fsm_therm_init(fsm_therm_t *t) {
    t->state = FSM_THERM_OFF;
    t->target_temp = 22.0f;
    t->current_temp = 20.0f;
    t->hysteresis = 1.0f;
    t->fan_on = 0;
}

void fsm_therm_power_on(fsm_therm_t *t) {
    if (t->state == FSM_THERM_OFF) t->state = FSM_THERM_IDLE;
}

void fsm_therm_update(fsm_therm_t *t, float temp) {
    t->current_temp = temp;
    if (t->state == FSM_THERM_OFF) return;
    if (temp < t->target_temp - t->hysteresis) {
        t->state = FSM_THERM_HEATING;
        t->fan_on = 1;
    } else if (temp > t->target_temp + t->hysteresis) {
        t->state = FSM_THERM_COOLING;
        t->fan_on = 1;
    } else {
        t->state = FSM_THERM_IDLE;
        t->fan_on = 0;
    }
}

void fsm_therm_fan_only(fsm_therm_t *t) {
    if (t->state != FSM_THERM_OFF) {
        t->state = FSM_THERM_FAN_ONLY;
        t->fan_on = 1;
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1149 thermostat failed: {:?}", result.err());
}

/// C1150: Conveyor belt controller FSM
#[test]
fn c1150_conveyor_belt() {
    let c_code = r#"
typedef enum {
    FSM_CONV_STOPPED, FSM_CONV_STARTING, FSM_CONV_RUNNING,
    FSM_CONV_STOPPING, FSM_CONV_JAMMED, FSM_CONV_EMERGENCY
} fsm_conv_state;

typedef struct {
    fsm_conv_state state;
    int speed;
    int target_speed;
    int item_count;
    int jam_sensor;
} fsm_conv_t;

void fsm_conv_init(fsm_conv_t *c) {
    c->state = FSM_CONV_STOPPED;
    c->speed = 0;
    c->target_speed = 100;
    c->item_count = 0;
    c->jam_sensor = 0;
}

void fsm_conv_start(fsm_conv_t *c) {
    if (c->state == FSM_CONV_STOPPED) c->state = FSM_CONV_STARTING;
}

void fsm_conv_tick(fsm_conv_t *c) {
    if (c->jam_sensor) { c->state = FSM_CONV_JAMMED; c->speed = 0; return; }
    if (c->state == FSM_CONV_STARTING) {
        c->speed += 10;
        if (c->speed >= c->target_speed) {
            c->speed = c->target_speed;
            c->state = FSM_CONV_RUNNING;
        }
    } else if (c->state == FSM_CONV_STOPPING) {
        c->speed -= 10;
        if (c->speed <= 0) {
            c->speed = 0;
            c->state = FSM_CONV_STOPPED;
        }
    } else if (c->state == FSM_CONV_RUNNING) {
        c->item_count++;
    }
}

void fsm_conv_stop(fsm_conv_t *c) {
    if (c->state == FSM_CONV_RUNNING || c->state == FSM_CONV_STARTING) {
        c->state = FSM_CONV_STOPPING;
    }
}

void fsm_conv_emergency(fsm_conv_t *c) {
    c->state = FSM_CONV_EMERGENCY;
    c->speed = 0;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1150 conveyor belt failed: {:?}", result.err());
}
