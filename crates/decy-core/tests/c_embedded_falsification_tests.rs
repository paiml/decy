//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C276-C300: Embedded Systems and RTOS patterns -- firmware, real-time
//! operating systems, microcontroller drivers, and bare-metal programming.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world embedded systems patterns commonly
//! found in RTOS kernels, MCU firmware, peripheral drivers, and
//! hardware abstraction layers -- all expressed as valid C99.
//!
//! Organization:
//! - C276-C280: Register access, DMA, and communication buffers
//! - C281-C285: RTOS primitives (scheduling, synchronization, GPIO)
//! - C286-C290: Bus protocols (SPI, I2C, PWM, ADC, watchdog)
//! - C291-C295: Flash, CAN, RTC, power management, bootloader
//! - C296-C300: Stack guard, memory pool, event flags, scheduling, HAL
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C276-C280: Register Access, DMA, and Communication Buffers
// ============================================================================

#[test]
fn c276_volatile_register_access() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define REG_BASE 0x40000000

typedef struct {
    volatile uint32_t CR;
    volatile uint32_t SR;
    volatile uint32_t DR;
    volatile uint32_t BRR;
} uart_regs_t;

void uart_write_reg(uart_regs_t *regs, uint32_t value) {
    regs->DR = value;
}

uint32_t uart_read_reg(const uart_regs_t *regs) {
    return regs->DR;
}

void uart_set_baud(uart_regs_t *regs, uint32_t baud_div) {
    regs->BRR = baud_div;
}

int uart_tx_ready(const uart_regs_t *regs) {
    return (regs->SR & (1U << 7)) != 0;
}

int uart_rx_ready(const uart_regs_t *regs) {
    return (regs->SR & (1U << 5)) != 0;
}

void uart_enable(uart_regs_t *regs) {
    regs->CR |= (1U << 13);
}

void uart_disable(uart_regs_t *regs) {
    regs->CR &= ~(1U << 13);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C276: Volatile register access should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C276: Output should not be empty");
    assert!(
        code.contains("fn uart_write_reg"),
        "C276: Should contain uart_write_reg function"
    );
}

#[test]
fn c277_mmio_register_bitfield_manipulation() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint32_t control;
    uint32_t status;
    uint32_t data;
    uint32_t config;
} periph_regs_t;

void periph_set_bits(periph_regs_t *regs, uint32_t mask) {
    regs->control |= mask;
}

void periph_clear_bits(periph_regs_t *regs, uint32_t mask) {
    regs->control &= ~mask;
}

void periph_modify_field(periph_regs_t *regs, uint32_t mask, uint32_t shift, uint32_t value) {
    uint32_t tmp = regs->config;
    tmp &= ~(mask << shift);
    tmp |= (value & mask) << shift;
    regs->config = tmp;
}

uint32_t periph_read_field(const periph_regs_t *regs, uint32_t mask, uint32_t shift) {
    return (regs->config >> shift) & mask;
}

int periph_wait_flag(const periph_regs_t *regs, uint32_t flag_mask, int max_iter) {
    for (int i = 0; i < max_iter; i++) {
        if (regs->status & flag_mask) {
            return i;
        }
    }
    return -1;
}

void periph_write_data(periph_regs_t *regs, uint32_t value) {
    regs->data = value;
}

uint32_t periph_read_data(const periph_regs_t *regs) {
    return regs->data;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C277: MMIO register bitfield manipulation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C277: Output should not be empty");
    assert!(
        code.contains("fn periph_set_bits"),
        "C277: Should contain periph_set_bits function"
    );
    assert!(
        code.contains("fn periph_modify_field"),
        "C277: Should contain periph_modify_field function"
    );
    assert!(
        code.contains("fn periph_read_field"),
        "C277: Should contain periph_read_field function"
    );
}

#[test]
fn c278_interrupt_service_routine_dispatch() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_ISR 64

typedef void (*isr_fn_t)(void);

typedef struct {
    isr_fn_t handlers[64];
    uint32_t pending;
    uint32_t enabled;
    uint32_t priority[64];
    uint32_t nest_count;
} nvic_t;

void nvic_init(nvic_t *nvic) {
    nvic->pending = 0;
    nvic->enabled = 0;
    nvic->nest_count = 0;
    for (int i = 0; i < 64; i++) {
        nvic->handlers[i] = 0;
        nvic->priority[i] = 15;
    }
}

int nvic_register_isr(nvic_t *nvic, int irq, isr_fn_t handler, uint32_t prio) {
    if (irq < 0 || irq >= 64) return -1;
    nvic->handlers[irq] = handler;
    nvic->priority[irq] = prio;
    return 0;
}

void nvic_enable_irq(nvic_t *nvic, int irq) {
    if (irq >= 0 && irq < 32) {
        nvic->enabled |= (1U << irq);
    }
}

void nvic_disable_irq(nvic_t *nvic, int irq) {
    if (irq >= 0 && irq < 32) {
        nvic->enabled &= ~(1U << irq);
    }
}

void nvic_set_pending(nvic_t *nvic, int irq) {
    if (irq >= 0 && irq < 32) {
        nvic->pending |= (1U << irq);
    }
}

void nvic_clear_pending(nvic_t *nvic, int irq) {
    if (irq >= 0 && irq < 32) {
        nvic->pending &= ~(1U << irq);
    }
}

int nvic_get_highest_pending(const nvic_t *nvic) {
    uint32_t active = nvic->pending & nvic->enabled;
    if (active == 0) return -1;
    int best_irq = -1;
    uint32_t best_prio = 16;
    for (int i = 0; i < 32; i++) {
        if ((active & (1U << i)) && nvic->priority[i] < best_prio) {
            best_prio = nvic->priority[i];
            best_irq = i;
        }
    }
    return best_irq;
}

int nvic_is_nested(const nvic_t *nvic) {
    return nvic->nest_count > 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C278: Interrupt service routine dispatch should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C278: Output should not be empty");
    assert!(
        code.contains("fn nvic_init"),
        "C278: Should contain nvic_init function"
    );
    assert!(
        code.contains("fn nvic_register_isr"),
        "C278: Should contain nvic_register_isr function"
    );
    assert!(
        code.contains("fn nvic_get_highest_pending"),
        "C278: Should contain nvic_get_highest_pending function"
    );
}

#[test]
fn c279_dma_transfer_descriptor_chain() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;

#define DMA_MAX_DESC 32

#define DMA_DIR_MEM_TO_PERIPH 0
#define DMA_DIR_PERIPH_TO_MEM 1
#define DMA_DIR_MEM_TO_MEM   2

typedef struct {
    uint32_t src_addr;
    uint32_t dst_addr;
    uint16_t transfer_size;
    uint16_t flags;
    int next_desc;
    int completed;
} dma_desc_t;

typedef struct {
    dma_desc_t descriptors[32];
    int desc_count;
    int active_chain_head;
    int current_desc;
    uint32_t bytes_transferred;
    int error_code;
} dma_channel_t;

void dma_init(dma_channel_t *ch) {
    ch->desc_count = 0;
    ch->active_chain_head = -1;
    ch->current_desc = -1;
    ch->bytes_transferred = 0;
    ch->error_code = 0;
    for (int i = 0; i < 32; i++) {
        ch->descriptors[i].next_desc = -1;
        ch->descriptors[i].completed = 0;
        ch->descriptors[i].transfer_size = 0;
        ch->descriptors[i].flags = 0;
    }
}

int dma_add_descriptor(dma_channel_t *ch, uint32_t src, uint32_t dst,
                       uint16_t size, uint16_t flags) {
    if (ch->desc_count >= 32) return -1;
    int idx = ch->desc_count;
    ch->descriptors[idx].src_addr = src;
    ch->descriptors[idx].dst_addr = dst;
    ch->descriptors[idx].transfer_size = size;
    ch->descriptors[idx].flags = flags;
    ch->descriptors[idx].completed = 0;
    ch->descriptors[idx].next_desc = -1;
    if (idx > 0) {
        ch->descriptors[idx - 1].next_desc = idx;
    }
    ch->desc_count++;
    return idx;
}

int dma_start(dma_channel_t *ch) {
    if (ch->desc_count == 0) return -1;
    ch->active_chain_head = 0;
    ch->current_desc = 0;
    ch->bytes_transferred = 0;
    ch->error_code = 0;
    return 0;
}

int dma_step(dma_channel_t *ch) {
    if (ch->current_desc < 0) return -1;
    dma_desc_t *desc = &ch->descriptors[ch->current_desc];
    desc->completed = 1;
    ch->bytes_transferred += desc->transfer_size;
    ch->current_desc = desc->next_desc;
    return ch->current_desc;
}

int dma_is_complete(const dma_channel_t *ch) {
    return ch->current_desc == -1 && ch->desc_count > 0;
}

uint32_t dma_get_bytes_transferred(const dma_channel_t *ch) {
    return ch->bytes_transferred;
}

int dma_chain_length(const dma_channel_t *ch) {
    int count = 0;
    int idx = ch->active_chain_head;
    while (idx >= 0 && idx < 32) {
        count++;
        idx = ch->descriptors[idx].next_desc;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C279: DMA transfer descriptor chain should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C279: Output should not be empty");
    assert!(
        code.contains("fn dma_init"),
        "C279: Should contain dma_init function"
    );
    assert!(
        code.contains("fn dma_add_descriptor"),
        "C279: Should contain dma_add_descriptor function"
    );
    assert!(
        code.contains("fn dma_step"),
        "C279: Should contain dma_step function"
    );
    assert!(
        code.contains("fn dma_is_complete"),
        "C279: Should contain dma_is_complete function"
    );
}

#[test]
fn c280_circular_buffer_uart_communication() {
    let c_code = r#"
typedef unsigned char uint8_t;

#define UART_BUF_SIZE 256

typedef struct {
    uint8_t tx_buf[256];
    uint8_t rx_buf[256];
    int tx_head;
    int tx_tail;
    int tx_count;
    int rx_head;
    int rx_tail;
    int rx_count;
    int overrun_errors;
    int framing_errors;
} uart_state_t;

void uart_buf_init(uart_state_t *u) {
    u->tx_head = 0;
    u->tx_tail = 0;
    u->tx_count = 0;
    u->rx_head = 0;
    u->rx_tail = 0;
    u->rx_count = 0;
    u->overrun_errors = 0;
    u->framing_errors = 0;
}

int uart_tx_enqueue(uart_state_t *u, uint8_t byte) {
    if (u->tx_count >= 256) return -1;
    u->tx_buf[u->tx_head] = byte;
    u->tx_head = (u->tx_head + 1) % 256;
    u->tx_count++;
    return 0;
}

int uart_tx_dequeue(uart_state_t *u, uint8_t *byte) {
    if (u->tx_count == 0) return -1;
    *byte = u->tx_buf[u->tx_tail];
    u->tx_tail = (u->tx_tail + 1) % 256;
    u->tx_count--;
    return 0;
}

int uart_rx_enqueue(uart_state_t *u, uint8_t byte) {
    if (u->rx_count >= 256) {
        u->overrun_errors++;
        return -1;
    }
    u->rx_buf[u->rx_head] = byte;
    u->rx_head = (u->rx_head + 1) % 256;
    u->rx_count++;
    return 0;
}

int uart_rx_dequeue(uart_state_t *u, uint8_t *byte) {
    if (u->rx_count == 0) return -1;
    *byte = u->rx_buf[u->rx_tail];
    u->rx_tail = (u->rx_tail + 1) % 256;
    u->rx_count--;
    return 0;
}

int uart_tx_space(const uart_state_t *u) {
    return 256 - u->tx_count;
}

int uart_rx_available(const uart_state_t *u) {
    return u->rx_count;
}

int uart_tx_write_block(uart_state_t *u, const uint8_t *data, int len) {
    int written = 0;
    for (int i = 0; i < len; i++) {
        if (uart_tx_enqueue(u, data[i]) != 0) break;
        written++;
    }
    return written;
}

int uart_get_error_count(const uart_state_t *u) {
    return u->overrun_errors + u->framing_errors;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C280: Circular buffer UART communication should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C280: Output should not be empty");
    assert!(
        code.contains("fn uart_buf_init"),
        "C280: Should contain uart_buf_init function"
    );
    assert!(
        code.contains("fn uart_tx_enqueue"),
        "C280: Should contain uart_tx_enqueue function"
    );
    assert!(
        code.contains("fn uart_rx_dequeue"),
        "C280: Should contain uart_rx_dequeue function"
    );
    assert!(
        code.contains("fn uart_tx_write_block"),
        "C280: Should contain uart_tx_write_block function"
    );
}

// ============================================================================
// C281-C285: RTOS Primitives (Scheduling, Synchronization, GPIO)
// ============================================================================

#[test]
fn c281_rtos_priority_queue_task_scheduling() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_TASKS 32

#define TASK_READY    0
#define TASK_RUNNING  1
#define TASK_BLOCKED  2
#define TASK_SUSPENDED 3

typedef struct {
    int task_id;
    int priority;
    int state;
    uint32_t stack_size;
    uint32_t tick_count;
    uint32_t wake_tick;
} tcb_t;

typedef struct {
    tcb_t tasks[32];
    int task_count;
    int current_task;
    uint32_t system_tick;
} scheduler_t;

void sched_init(scheduler_t *s) {
    s->task_count = 0;
    s->current_task = -1;
    s->system_tick = 0;
}

int sched_create_task(scheduler_t *s, int priority, uint32_t stack_size) {
    if (s->task_count >= 32) return -1;
    int idx = s->task_count;
    s->tasks[idx].task_id = idx;
    s->tasks[idx].priority = priority;
    s->tasks[idx].state = TASK_READY;
    s->tasks[idx].stack_size = stack_size;
    s->tasks[idx].tick_count = 0;
    s->tasks[idx].wake_tick = 0;
    s->task_count++;
    return idx;
}

int sched_get_next_task(const scheduler_t *s) {
    int best = -1;
    int best_prio = 256;
    for (int i = 0; i < s->task_count; i++) {
        if (s->tasks[i].state == TASK_READY && s->tasks[i].priority < best_prio) {
            best_prio = s->tasks[i].priority;
            best = i;
        }
    }
    return best;
}

void sched_tick(scheduler_t *s) {
    s->system_tick++;
    for (int i = 0; i < s->task_count; i++) {
        if (s->tasks[i].state == TASK_BLOCKED) {
            if (s->system_tick >= s->tasks[i].wake_tick) {
                s->tasks[i].state = TASK_READY;
            }
        }
    }
    if (s->current_task >= 0) {
        s->tasks[s->current_task].tick_count++;
    }
}

void sched_sleep_task(scheduler_t *s, int task_id, uint32_t ticks) {
    if (task_id >= 0 && task_id < s->task_count) {
        s->tasks[task_id].state = TASK_BLOCKED;
        s->tasks[task_id].wake_tick = s->system_tick + ticks;
    }
}

void sched_suspend_task(scheduler_t *s, int task_id) {
    if (task_id >= 0 && task_id < s->task_count) {
        s->tasks[task_id].state = TASK_SUSPENDED;
    }
}

void sched_resume_task(scheduler_t *s, int task_id) {
    if (task_id >= 0 && task_id < s->task_count) {
        if (s->tasks[task_id].state == TASK_SUSPENDED) {
            s->tasks[task_id].state = TASK_READY;
        }
    }
}

int sched_ready_count(const scheduler_t *s) {
    int count = 0;
    for (int i = 0; i < s->task_count; i++) {
        if (s->tasks[i].state == TASK_READY) count++;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C281: RTOS priority queue task scheduling should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C281: Output should not be empty");
    assert!(
        code.contains("fn sched_init"),
        "C281: Should contain sched_init function"
    );
    assert!(
        code.contains("fn sched_create_task"),
        "C281: Should contain sched_create_task function"
    );
    assert!(
        code.contains("fn sched_get_next_task"),
        "C281: Should contain sched_get_next_task function"
    );
    assert!(
        code.contains("fn sched_tick"),
        "C281: Should contain sched_tick function"
    );
}

#[test]
fn c282_mutex_with_priority_inheritance() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_MUTEXES 16
#define MUTEX_UNLOCKED 0
#define MUTEX_LOCKED   1

typedef struct {
    int state;
    int owner_task;
    int original_priority;
    int inherited_priority;
    int wait_count;
    int waiters[8];
} rtos_mutex_t;

typedef struct {
    rtos_mutex_t mutexes[16];
    int mutex_count;
} mutex_pool_t;

void mutex_pool_init(mutex_pool_t *pool) {
    pool->mutex_count = 0;
}

int mutex_create(mutex_pool_t *pool) {
    if (pool->mutex_count >= 16) return -1;
    int idx = pool->mutex_count;
    pool->mutexes[idx].state = MUTEX_UNLOCKED;
    pool->mutexes[idx].owner_task = -1;
    pool->mutexes[idx].original_priority = -1;
    pool->mutexes[idx].inherited_priority = -1;
    pool->mutexes[idx].wait_count = 0;
    for (int w = 0; w < 8; w++) {
        pool->mutexes[idx].waiters[w] = -1;
    }
    pool->mutex_count++;
    return idx;
}

int mutex_try_lock(mutex_pool_t *pool, int mutex_id, int task_id, int task_priority) {
    if (mutex_id < 0 || mutex_id >= pool->mutex_count) return -1;
    rtos_mutex_t *m = &pool->mutexes[mutex_id];
    if (m->state == MUTEX_UNLOCKED) {
        m->state = MUTEX_LOCKED;
        m->owner_task = task_id;
        m->original_priority = task_priority;
        m->inherited_priority = task_priority;
        return 0;
    }
    if (task_priority < m->inherited_priority) {
        m->inherited_priority = task_priority;
    }
    if (m->wait_count < 8) {
        m->waiters[m->wait_count] = task_id;
        m->wait_count++;
    }
    return 1;
}

int mutex_unlock(mutex_pool_t *pool, int mutex_id, int task_id) {
    if (mutex_id < 0 || mutex_id >= pool->mutex_count) return -1;
    rtos_mutex_t *m = &pool->mutexes[mutex_id];
    if (m->owner_task != task_id) return -2;
    m->state = MUTEX_UNLOCKED;
    m->owner_task = -1;
    m->inherited_priority = -1;
    if (m->wait_count > 0) {
        m->wait_count--;
    }
    return 0;
}

int mutex_is_locked(const mutex_pool_t *pool, int mutex_id) {
    if (mutex_id < 0 || mutex_id >= pool->mutex_count) return -1;
    return pool->mutexes[mutex_id].state == MUTEX_LOCKED;
}

int mutex_get_owner(const mutex_pool_t *pool, int mutex_id) {
    if (mutex_id < 0 || mutex_id >= pool->mutex_count) return -1;
    return pool->mutexes[mutex_id].owner_task;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C282: Mutex with priority inheritance should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C282: Output should not be empty");
    assert!(
        code.contains("fn mutex_pool_init"),
        "C282: Should contain mutex_pool_init function"
    );
    assert!(
        code.contains("fn mutex_create"),
        "C282: Should contain mutex_create function"
    );
    assert!(
        code.contains("fn mutex_try_lock"),
        "C282: Should contain mutex_try_lock function"
    );
    assert!(
        code.contains("fn mutex_unlock"),
        "C282: Should contain mutex_unlock function"
    );
}

#[test]
fn c283_semaphore_producer_consumer() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define SEM_MAX_VALUE 255
#define QUEUE_SIZE 64

typedef struct {
    int count;
    int max_value;
} semaphore_t;

typedef struct {
    int data[64];
    int head;
    int tail;
    int count;
    semaphore_t items_sem;
    semaphore_t spaces_sem;
} pc_queue_t;

void sem_init(semaphore_t *s, int initial, int max_val) {
    s->count = initial;
    s->max_value = max_val;
}

int sem_wait(semaphore_t *s) {
    if (s->count <= 0) return -1;
    s->count--;
    return 0;
}

int sem_signal(semaphore_t *s) {
    if (s->count >= s->max_value) return -1;
    s->count++;
    return 0;
}

int sem_get_count(const semaphore_t *s) {
    return s->count;
}

void pcq_init(pc_queue_t *q) {
    q->head = 0;
    q->tail = 0;
    q->count = 0;
    sem_init(&q->items_sem, 0, 64);
    sem_init(&q->spaces_sem, 64, 64);
}

int pcq_produce(pc_queue_t *q, int item) {
    if (sem_wait(&q->spaces_sem) != 0) return -1;
    q->data[q->head] = item;
    q->head = (q->head + 1) % 64;
    q->count++;
    sem_signal(&q->items_sem);
    return 0;
}

int pcq_consume(pc_queue_t *q, int *item) {
    if (sem_wait(&q->items_sem) != 0) return -1;
    *item = q->data[q->tail];
    q->tail = (q->tail + 1) % 64;
    q->count--;
    sem_signal(&q->spaces_sem);
    return 0;
}

int pcq_is_empty(const pc_queue_t *q) {
    return q->count == 0;
}

int pcq_is_full(const pc_queue_t *q) {
    return q->count >= 64;
}

int pcq_size(const pc_queue_t *q) {
    return q->count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C283: Semaphore producer-consumer should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C283: Output should not be empty");
    assert!(
        code.contains("fn sem_init"),
        "C283: Should contain sem_init function"
    );
    assert!(
        code.contains("fn pcq_init"),
        "C283: Should contain pcq_init function"
    );
    assert!(
        code.contains("fn pcq_produce"),
        "C283: Should contain pcq_produce function"
    );
    assert!(
        code.contains("fn pcq_consume"),
        "C283: Should contain pcq_consume function"
    );
}

#[test]
fn c284_timer_callback_function_pointer_table() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_TIMERS 16

typedef void (*timer_callback_t)(int timer_id, void *user_data);

typedef struct {
    timer_callback_t callback;
    void *user_data;
    uint32_t interval_ms;
    uint32_t remaining_ms;
    int active;
    int one_shot;
    uint32_t fire_count;
} hw_timer_t;

typedef struct {
    hw_timer_t timers[16];
    int timer_count;
    uint32_t tick_ms;
} timer_mgr_t;

void timer_mgr_init(timer_mgr_t *mgr, uint32_t tick_ms) {
    mgr->timer_count = 0;
    mgr->tick_ms = tick_ms;
    for (int i = 0; i < 16; i++) {
        mgr->timers[i].active = 0;
        mgr->timers[i].callback = 0;
        mgr->timers[i].user_data = 0;
        mgr->timers[i].fire_count = 0;
    }
}

int timer_create_periodic(timer_mgr_t *mgr, uint32_t interval_ms,
                          timer_callback_t cb, void *data) {
    if (mgr->timer_count >= 16) return -1;
    int idx = mgr->timer_count;
    mgr->timers[idx].callback = cb;
    mgr->timers[idx].user_data = data;
    mgr->timers[idx].interval_ms = interval_ms;
    mgr->timers[idx].remaining_ms = interval_ms;
    mgr->timers[idx].active = 1;
    mgr->timers[idx].one_shot = 0;
    mgr->timers[idx].fire_count = 0;
    mgr->timer_count++;
    return idx;
}

int timer_create_oneshot(timer_mgr_t *mgr, uint32_t delay_ms,
                         timer_callback_t cb, void *data) {
    if (mgr->timer_count >= 16) return -1;
    int idx = mgr->timer_count;
    mgr->timers[idx].callback = cb;
    mgr->timers[idx].user_data = data;
    mgr->timers[idx].interval_ms = delay_ms;
    mgr->timers[idx].remaining_ms = delay_ms;
    mgr->timers[idx].active = 1;
    mgr->timers[idx].one_shot = 1;
    mgr->timers[idx].fire_count = 0;
    mgr->timer_count++;
    return idx;
}

int timer_process_tick(timer_mgr_t *mgr) {
    int fired = 0;
    for (int i = 0; i < mgr->timer_count; i++) {
        if (!mgr->timers[i].active) continue;
        if (mgr->timers[i].remaining_ms <= mgr->tick_ms) {
            mgr->timers[i].fire_count++;
            fired++;
            if (mgr->timers[i].one_shot) {
                mgr->timers[i].active = 0;
            } else {
                mgr->timers[i].remaining_ms = mgr->timers[i].interval_ms;
            }
        } else {
            mgr->timers[i].remaining_ms -= mgr->tick_ms;
        }
    }
    return fired;
}

int timer_cancel(timer_mgr_t *mgr, int timer_id) {
    if (timer_id < 0 || timer_id >= mgr->timer_count) return -1;
    mgr->timers[timer_id].active = 0;
    return 0;
}

int timer_active_count(const timer_mgr_t *mgr) {
    int count = 0;
    for (int i = 0; i < mgr->timer_count; i++) {
        if (mgr->timers[i].active) count++;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C284: Timer callback function pointer table should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C284: Output should not be empty");
    assert!(
        code.contains("fn timer_mgr_init"),
        "C284: Should contain timer_mgr_init function"
    );
    assert!(
        code.contains("fn timer_create_periodic"),
        "C284: Should contain timer_create_periodic function"
    );
    assert!(
        code.contains("fn timer_process_tick"),
        "C284: Should contain timer_process_tick function"
    );
}

#[test]
fn c285_gpio_pin_configuration_bitmask() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define GPIO_MODE_INPUT   0
#define GPIO_MODE_OUTPUT  1
#define GPIO_MODE_ALT     2
#define GPIO_MODE_ANALOG  3

#define GPIO_PULL_NONE  0
#define GPIO_PULL_UP    1
#define GPIO_PULL_DOWN  2

#define GPIO_SPEED_LOW    0
#define GPIO_SPEED_MED    1
#define GPIO_SPEED_HIGH   2
#define GPIO_SPEED_VHIGH  3

#define NUM_PINS 16

typedef struct {
    uint32_t mode_reg;
    uint32_t output_reg;
    uint32_t input_reg;
    uint32_t pull_reg;
    uint32_t speed_reg;
    uint32_t alt_func_low;
    uint32_t alt_func_high;
} gpio_port_t;

void gpio_init(gpio_port_t *port) {
    port->mode_reg = 0;
    port->output_reg = 0;
    port->input_reg = 0;
    port->pull_reg = 0;
    port->speed_reg = 0;
    port->alt_func_low = 0;
    port->alt_func_high = 0;
}

void gpio_set_mode(gpio_port_t *port, int pin, int mode) {
    if (pin < 0 || pin >= 16) return;
    uint32_t shift = (uint32_t)(pin * 2);
    port->mode_reg &= ~(3U << shift);
    port->mode_reg |= ((uint32_t)mode & 3U) << shift;
}

int gpio_get_mode(const gpio_port_t *port, int pin) {
    if (pin < 0 || pin >= 16) return -1;
    uint32_t shift = (uint32_t)(pin * 2);
    return (int)((port->mode_reg >> shift) & 3U);
}

void gpio_set_pull(gpio_port_t *port, int pin, int pull) {
    if (pin < 0 || pin >= 16) return;
    uint32_t shift = (uint32_t)(pin * 2);
    port->pull_reg &= ~(3U << shift);
    port->pull_reg |= ((uint32_t)pull & 3U) << shift;
}

void gpio_set_output(gpio_port_t *port, int pin) {
    if (pin >= 0 && pin < 16) {
        port->output_reg |= (1U << pin);
    }
}

void gpio_clear_output(gpio_port_t *port, int pin) {
    if (pin >= 0 && pin < 16) {
        port->output_reg &= ~(1U << pin);
    }
}

void gpio_toggle_output(gpio_port_t *port, int pin) {
    if (pin >= 0 && pin < 16) {
        port->output_reg ^= (1U << pin);
    }
}

int gpio_read_input(const gpio_port_t *port, int pin) {
    if (pin < 0 || pin >= 16) return -1;
    return (port->input_reg >> pin) & 1U;
}

void gpio_set_speed(gpio_port_t *port, int pin, int speed) {
    if (pin < 0 || pin >= 16) return;
    uint32_t shift = (uint32_t)(pin * 2);
    port->speed_reg &= ~(3U << shift);
    port->speed_reg |= ((uint32_t)speed & 3U) << shift;
}

void gpio_set_alt_func(gpio_port_t *port, int pin, int func) {
    if (pin < 0 || pin >= 16) return;
    if (pin < 8) {
        uint32_t shift = (uint32_t)(pin * 4);
        port->alt_func_low &= ~(0xFU << shift);
        port->alt_func_low |= ((uint32_t)func & 0xFU) << shift;
    } else {
        uint32_t shift = (uint32_t)((pin - 8) * 4);
        port->alt_func_high &= ~(0xFU << shift);
        port->alt_func_high |= ((uint32_t)func & 0xFU) << shift;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C285: GPIO pin configuration bitmask should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C285: Output should not be empty");
    assert!(
        code.contains("fn gpio_init"),
        "C285: Should contain gpio_init function"
    );
    assert!(
        code.contains("fn gpio_set_mode"),
        "C285: Should contain gpio_set_mode function"
    );
    assert!(
        code.contains("fn gpio_toggle_output"),
        "C285: Should contain gpio_toggle_output function"
    );
    assert!(
        code.contains("fn gpio_set_alt_func"),
        "C285: Should contain gpio_set_alt_func function"
    );
}

// ============================================================================
// C286-C290: Bus Protocols (SPI, I2C, PWM, ADC, Watchdog)
// ============================================================================

#[test]
fn c286_spi_bus_transaction_chip_select() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

#define SPI_MAX_DEVICES 8
#define SPI_BUF_SIZE 64

#define SPI_MODE_0 0
#define SPI_MODE_1 1
#define SPI_MODE_2 2
#define SPI_MODE_3 3

typedef struct {
    int cs_pin;
    int mode;
    uint32_t clock_hz;
    int active;
} spi_device_t;

typedef struct {
    spi_device_t devices[8];
    int device_count;
    uint8_t tx_buf[64];
    uint8_t rx_buf[64];
    int selected_device;
    int busy;
    uint32_t transfer_count;
} spi_bus_t;

void spi_init(spi_bus_t *bus) {
    bus->device_count = 0;
    bus->selected_device = -1;
    bus->busy = 0;
    bus->transfer_count = 0;
    for (int i = 0; i < 8; i++) {
        bus->devices[i].active = 0;
    }
}

int spi_add_device(spi_bus_t *bus, int cs_pin, int mode, uint32_t clock_hz) {
    if (bus->device_count >= 8) return -1;
    int idx = bus->device_count;
    bus->devices[idx].cs_pin = cs_pin;
    bus->devices[idx].mode = mode;
    bus->devices[idx].clock_hz = clock_hz;
    bus->devices[idx].active = 1;
    bus->device_count++;
    return idx;
}

int spi_select(spi_bus_t *bus, int device_id) {
    if (device_id < 0 || device_id >= bus->device_count) return -1;
    if (bus->busy) return -2;
    bus->selected_device = device_id;
    return 0;
}

void spi_deselect(spi_bus_t *bus) {
    bus->selected_device = -1;
}

int spi_transfer(spi_bus_t *bus, const uint8_t *tx, uint8_t *rx, int len) {
    if (bus->selected_device < 0) return -1;
    if (len > 64) return -2;
    bus->busy = 1;
    for (int i = 0; i < len; i++) {
        bus->tx_buf[i] = tx[i];
        rx[i] = bus->tx_buf[i];
    }
    bus->transfer_count++;
    bus->busy = 0;
    return len;
}

int spi_write_byte(spi_bus_t *bus, uint8_t byte) {
    uint8_t rx;
    return spi_transfer(bus, &byte, &rx, 1);
}

int spi_read_byte(spi_bus_t *bus, uint8_t *byte) {
    uint8_t tx = 0xFF;
    return spi_transfer(bus, &tx, byte, 1);
}

int spi_is_busy(const spi_bus_t *bus) {
    return bus->busy;
}

uint32_t spi_get_transfer_count(const spi_bus_t *bus) {
    return bus->transfer_count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C286: SPI bus transaction with chip select should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C286: Output should not be empty");
    assert!(
        code.contains("fn spi_init"),
        "C286: Should contain spi_init function"
    );
    assert!(
        code.contains("fn spi_transfer"),
        "C286: Should contain spi_transfer function"
    );
    assert!(
        code.contains("fn spi_select"),
        "C286: Should contain spi_select function"
    );
}

#[test]
fn c287_i2c_master_send_receive() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

#define I2C_BUF_SIZE 32

#define I2C_OK        0
#define I2C_NACK     -1
#define I2C_TIMEOUT  -2
#define I2C_BUS_ERR  -3

typedef struct {
    uint8_t tx_buf[32];
    uint8_t rx_buf[32];
    int tx_len;
    int rx_len;
    uint8_t slave_addr;
    int state;
    int error;
    uint32_t transaction_count;
    uint32_t error_count;
} i2c_master_t;

void i2c_init(i2c_master_t *i2c) {
    i2c->tx_len = 0;
    i2c->rx_len = 0;
    i2c->slave_addr = 0;
    i2c->state = 0;
    i2c->error = 0;
    i2c->transaction_count = 0;
    i2c->error_count = 0;
}

int i2c_write(i2c_master_t *i2c, uint8_t addr, const uint8_t *data, int len) {
    if (len > 32 || len <= 0) return I2C_BUS_ERR;
    i2c->slave_addr = addr;
    for (int i = 0; i < len; i++) {
        i2c->tx_buf[i] = data[i];
    }
    i2c->tx_len = len;
    i2c->transaction_count++;
    return I2C_OK;
}

int i2c_read(i2c_master_t *i2c, uint8_t addr, uint8_t *data, int len) {
    if (len > 32 || len <= 0) return I2C_BUS_ERR;
    i2c->slave_addr = addr;
    i2c->rx_len = len;
    for (int i = 0; i < len; i++) {
        data[i] = i2c->rx_buf[i];
    }
    i2c->transaction_count++;
    return I2C_OK;
}

int i2c_write_reg(i2c_master_t *i2c, uint8_t addr, uint8_t reg, uint8_t value) {
    uint8_t buf[2];
    buf[0] = reg;
    buf[1] = value;
    return i2c_write(i2c, addr, buf, 2);
}

int i2c_read_reg(i2c_master_t *i2c, uint8_t addr, uint8_t reg, uint8_t *value) {
    int rc = i2c_write(i2c, addr, &reg, 1);
    if (rc != I2C_OK) return rc;
    return i2c_read(i2c, addr, value, 1);
}

int i2c_scan_bus(i2c_master_t *i2c, uint8_t *found_addrs, int max_found) {
    int count = 0;
    for (int addr = 0x08; addr < 0x78; addr++) {
        uint8_t dummy;
        int rc = i2c_read(i2c, (uint8_t)addr, &dummy, 1);
        if (rc == I2C_OK && count < max_found) {
            found_addrs[count] = (uint8_t)addr;
            count++;
        }
    }
    return count;
}

uint32_t i2c_get_error_rate(const i2c_master_t *i2c) {
    if (i2c->transaction_count == 0) return 0;
    return (i2c->error_count * 100) / i2c->transaction_count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C287: I2C master send/receive should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C287: Output should not be empty");
    assert!(
        code.contains("fn i2c_init"),
        "C287: Should contain i2c_init function"
    );
    assert!(
        code.contains("fn i2c_write"),
        "C287: Should contain i2c_write function"
    );
    assert!(
        code.contains("fn i2c_read_reg"),
        "C287: Should contain i2c_read_reg function"
    );
    assert!(
        code.contains("fn i2c_scan_bus"),
        "C287: Should contain i2c_scan_bus function"
    );
}

#[test]
fn c288_pwm_signal_generation_duty_cycle() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;

#define PWM_MAX_CHANNELS 8

typedef struct {
    uint32_t period;
    uint32_t duty;
    int enabled;
    int polarity;
    uint32_t frequency_hz;
} pwm_channel_t;

typedef struct {
    pwm_channel_t channels[8];
    int channel_count;
    uint32_t timer_clock_hz;
    uint16_t prescaler;
} pwm_controller_t;

void pwm_init(pwm_controller_t *pwm, uint32_t timer_clock_hz) {
    pwm->timer_clock_hz = timer_clock_hz;
    pwm->prescaler = 1;
    pwm->channel_count = 0;
    for (int i = 0; i < 8; i++) {
        pwm->channels[i].period = 0;
        pwm->channels[i].duty = 0;
        pwm->channels[i].enabled = 0;
        pwm->channels[i].polarity = 0;
        pwm->channels[i].frequency_hz = 0;
    }
}

int pwm_configure(pwm_controller_t *pwm, int ch, uint32_t freq_hz, int duty_percent) {
    if (ch < 0 || ch >= 8) return -1;
    if (duty_percent < 0 || duty_percent > 100) return -2;
    if (freq_hz == 0) return -3;
    uint32_t period = pwm->timer_clock_hz / (freq_hz * pwm->prescaler);
    uint32_t duty = (period * (uint32_t)duty_percent) / 100;
    pwm->channels[ch].period = period;
    pwm->channels[ch].duty = duty;
    pwm->channels[ch].frequency_hz = freq_hz;
    if (ch >= pwm->channel_count) {
        pwm->channel_count = ch + 1;
    }
    return 0;
}

void pwm_enable(pwm_controller_t *pwm, int ch) {
    if (ch >= 0 && ch < 8) {
        pwm->channels[ch].enabled = 1;
    }
}

void pwm_disable(pwm_controller_t *pwm, int ch) {
    if (ch >= 0 && ch < 8) {
        pwm->channels[ch].enabled = 0;
    }
}

int pwm_set_duty(pwm_controller_t *pwm, int ch, int duty_percent) {
    if (ch < 0 || ch >= 8) return -1;
    if (duty_percent < 0 || duty_percent > 100) return -2;
    pwm->channels[ch].duty = (pwm->channels[ch].period * (uint32_t)duty_percent) / 100;
    return 0;
}

int pwm_get_duty_percent(const pwm_controller_t *pwm, int ch) {
    if (ch < 0 || ch >= 8) return -1;
    if (pwm->channels[ch].period == 0) return 0;
    return (int)((pwm->channels[ch].duty * 100) / pwm->channels[ch].period);
}

void pwm_set_polarity(pwm_controller_t *pwm, int ch, int polarity) {
    if (ch >= 0 && ch < 8) {
        pwm->channels[ch].polarity = polarity;
    }
}

int pwm_active_channels(const pwm_controller_t *pwm) {
    int count = 0;
    for (int i = 0; i < pwm->channel_count; i++) {
        if (pwm->channels[i].enabled) count++;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C288: PWM signal generation with duty cycle should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C288: Output should not be empty");
    assert!(
        code.contains("fn pwm_init"),
        "C288: Should contain pwm_init function"
    );
    assert!(
        code.contains("fn pwm_configure"),
        "C288: Should contain pwm_configure function"
    );
    assert!(
        code.contains("fn pwm_set_duty"),
        "C288: Should contain pwm_set_duty function"
    );
    assert!(
        code.contains("fn pwm_active_channels"),
        "C288: Should contain pwm_active_channels function"
    );
}

#[test]
fn c289_adc_conversion_with_dma_scatter_gather() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;

#define ADC_MAX_CHANNELS 16
#define ADC_BUF_DEPTH 32

typedef struct {
    uint16_t samples[16][32];
    int channel_enabled[16];
    int num_channels;
    int sample_idx;
    uint32_t conversion_count;
    int dma_active;
    uint16_t resolution_bits;
} adc_controller_t;

void adc_init(adc_controller_t *adc, uint16_t resolution) {
    adc->num_channels = 0;
    adc->sample_idx = 0;
    adc->conversion_count = 0;
    adc->dma_active = 0;
    adc->resolution_bits = resolution;
    for (int ch = 0; ch < 16; ch++) {
        adc->channel_enabled[ch] = 0;
        for (int s = 0; s < 32; s++) {
            adc->samples[ch][s] = 0;
        }
    }
}

int adc_enable_channel(adc_controller_t *adc, int ch) {
    if (ch < 0 || ch >= 16) return -1;
    if (!adc->channel_enabled[ch]) {
        adc->channel_enabled[ch] = 1;
        adc->num_channels++;
    }
    return 0;
}

int adc_disable_channel(adc_controller_t *adc, int ch) {
    if (ch < 0 || ch >= 16) return -1;
    if (adc->channel_enabled[ch]) {
        adc->channel_enabled[ch] = 0;
        adc->num_channels--;
    }
    return 0;
}

void adc_store_sample(adc_controller_t *adc, int ch, uint16_t value) {
    if (ch >= 0 && ch < 16 && adc->channel_enabled[ch]) {
        adc->samples[ch][adc->sample_idx % 32] = value;
    }
}

void adc_advance_index(adc_controller_t *adc) {
    adc->sample_idx = (adc->sample_idx + 1) % 32;
    adc->conversion_count++;
}

uint16_t adc_get_latest(const adc_controller_t *adc, int ch) {
    if (ch < 0 || ch >= 16) return 0;
    int idx = (adc->sample_idx + 32 - 1) % 32;
    return adc->samples[ch][idx];
}

uint32_t adc_get_average(const adc_controller_t *adc, int ch, int num_samples) {
    if (ch < 0 || ch >= 16 || num_samples <= 0) return 0;
    if (num_samples > 32) num_samples = 32;
    uint32_t sum = 0;
    for (int i = 0; i < num_samples; i++) {
        int idx = (adc->sample_idx + 32 - 1 - i) % 32;
        sum += adc->samples[ch][idx];
    }
    return sum / (uint32_t)num_samples;
}

uint16_t adc_get_max(const adc_controller_t *adc, int ch) {
    if (ch < 0 || ch >= 16) return 0;
    uint16_t max_val = 0;
    for (int i = 0; i < 32; i++) {
        if (adc->samples[ch][i] > max_val) {
            max_val = adc->samples[ch][i];
        }
    }
    return max_val;
}

int adc_dma_start(adc_controller_t *adc) {
    if (adc->num_channels == 0) return -1;
    adc->dma_active = 1;
    return 0;
}

void adc_dma_stop(adc_controller_t *adc) {
    adc->dma_active = 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C289: ADC conversion with DMA scatter-gather should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C289: Output should not be empty");
    assert!(
        code.contains("fn adc_init"),
        "C289: Should contain adc_init function"
    );
    assert!(
        code.contains("fn adc_enable_channel"),
        "C289: Should contain adc_enable_channel function"
    );
    assert!(
        code.contains("fn adc_get_average"),
        "C289: Should contain adc_get_average function"
    );
    assert!(
        code.contains("fn adc_dma_start"),
        "C289: Should contain adc_dma_start function"
    );
}

#[test]
fn c290_watchdog_timer_kick_feed() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define WDG_MAGIC_FEED 0xAA55

typedef struct {
    uint32_t timeout_ms;
    uint32_t remaining_ms;
    int enabled;
    uint32_t feed_count;
    uint32_t timeout_count;
    int expired;
    uint32_t window_min_ms;
    uint32_t window_max_ms;
} watchdog_t;

void wdg_init(watchdog_t *wdg, uint32_t timeout_ms) {
    wdg->timeout_ms = timeout_ms;
    wdg->remaining_ms = timeout_ms;
    wdg->enabled = 0;
    wdg->feed_count = 0;
    wdg->timeout_count = 0;
    wdg->expired = 0;
    wdg->window_min_ms = 0;
    wdg->window_max_ms = timeout_ms;
}

void wdg_enable(watchdog_t *wdg) {
    wdg->enabled = 1;
    wdg->remaining_ms = wdg->timeout_ms;
    wdg->expired = 0;
}

void wdg_disable(watchdog_t *wdg) {
    wdg->enabled = 0;
}

int wdg_feed(watchdog_t *wdg, uint32_t magic) {
    if (!wdg->enabled) return -1;
    if (magic != WDG_MAGIC_FEED) return -2;
    uint32_t elapsed = wdg->timeout_ms - wdg->remaining_ms;
    if (elapsed < wdg->window_min_ms) return -3;
    wdg->remaining_ms = wdg->timeout_ms;
    wdg->feed_count++;
    return 0;
}

int wdg_tick(watchdog_t *wdg, uint32_t elapsed_ms) {
    if (!wdg->enabled) return 0;
    if (wdg->remaining_ms <= elapsed_ms) {
        wdg->expired = 1;
        wdg->timeout_count++;
        wdg->remaining_ms = 0;
        return 1;
    }
    wdg->remaining_ms -= elapsed_ms;
    return 0;
}

int wdg_is_expired(const watchdog_t *wdg) {
    return wdg->expired;
}

uint32_t wdg_remaining(const watchdog_t *wdg) {
    return wdg->remaining_ms;
}

void wdg_set_window(watchdog_t *wdg, uint32_t min_ms, uint32_t max_ms) {
    wdg->window_min_ms = min_ms;
    wdg->window_max_ms = max_ms;
}

uint32_t wdg_get_feed_count(const watchdog_t *wdg) {
    return wdg->feed_count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C290: Watchdog timer kick/feed pattern should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C290: Output should not be empty");
    assert!(
        code.contains("fn wdg_init"),
        "C290: Should contain wdg_init function"
    );
    assert!(
        code.contains("fn wdg_feed"),
        "C290: Should contain wdg_feed function"
    );
    assert!(
        code.contains("fn wdg_tick"),
        "C290: Should contain wdg_tick function"
    );
    assert!(
        code.contains("fn wdg_is_expired"),
        "C290: Should contain wdg_is_expired function"
    );
}

// ============================================================================
// C291-C295: Flash, CAN, RTC, Power Management, Bootloader
// ============================================================================

#[test]
fn c291_flash_memory_page_erase_write() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define FLASH_PAGE_SIZE 256
#define FLASH_PAGES 64
#define FLASH_ERASED_VAL 0xFF

typedef struct {
    uint8_t pages[64][256];
    int page_locked[64];
    uint32_t write_count[64];
    uint32_t erase_count[64];
    int busy;
} flash_mem_t;

void flash_init(flash_mem_t *f) {
    f->busy = 0;
    for (int p = 0; p < 64; p++) {
        f->page_locked[p] = 0;
        f->write_count[p] = 0;
        f->erase_count[p] = 0;
        for (int b = 0; b < 256; b++) {
            f->pages[p][b] = FLASH_ERASED_VAL;
        }
    }
}

int flash_erase_page(flash_mem_t *f, int page) {
    if (page < 0 || page >= 64) return -1;
    if (f->page_locked[page]) return -2;
    if (f->busy) return -3;
    f->busy = 1;
    for (int b = 0; b < 256; b++) {
        f->pages[page][b] = FLASH_ERASED_VAL;
    }
    f->erase_count[page]++;
    f->busy = 0;
    return 0;
}

int flash_write_byte(flash_mem_t *f, int page, int offset, uint8_t value) {
    if (page < 0 || page >= 64) return -1;
    if (offset < 0 || offset >= 256) return -2;
    if (f->page_locked[page]) return -3;
    if (f->busy) return -4;
    if (f->pages[page][offset] != FLASH_ERASED_VAL) return -5;
    f->pages[page][offset] = value;
    f->write_count[page]++;
    return 0;
}

int flash_write_block(flash_mem_t *f, int page, int offset, const uint8_t *data, int len) {
    if (page < 0 || page >= 64) return -1;
    if (offset + len > 256) return -2;
    if (f->page_locked[page]) return -3;
    f->busy = 1;
    for (int i = 0; i < len; i++) {
        if (f->pages[page][offset + i] != FLASH_ERASED_VAL) {
            f->busy = 0;
            return -5;
        }
        f->pages[page][offset + i] = data[i];
    }
    f->write_count[page]++;
    f->busy = 0;
    return len;
}

uint8_t flash_read_byte(const flash_mem_t *f, int page, int offset) {
    if (page < 0 || page >= 64 || offset < 0 || offset >= 256) return FLASH_ERASED_VAL;
    return f->pages[page][offset];
}

void flash_lock_page(flash_mem_t *f, int page) {
    if (page >= 0 && page < 64) {
        f->page_locked[page] = 1;
    }
}

void flash_unlock_page(flash_mem_t *f, int page) {
    if (page >= 0 && page < 64) {
        f->page_locked[page] = 0;
    }
}

int flash_is_page_erased(const flash_mem_t *f, int page) {
    if (page < 0 || page >= 64) return -1;
    for (int b = 0; b < 256; b++) {
        if (f->pages[page][b] != FLASH_ERASED_VAL) return 0;
    }
    return 1;
}

uint32_t flash_get_wear_level(const flash_mem_t *f, int page) {
    if (page < 0 || page >= 64) return 0;
    return f->erase_count[page];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C291: Flash memory page erase and write should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C291: Output should not be empty");
    assert!(
        code.contains("fn flash_init"),
        "C291: Should contain flash_init function"
    );
    assert!(
        code.contains("fn flash_erase_page"),
        "C291: Should contain flash_erase_page function"
    );
    assert!(
        code.contains("fn flash_write_block"),
        "C291: Should contain flash_write_block function"
    );
    assert!(
        code.contains("fn flash_is_page_erased"),
        "C291: Should contain flash_is_page_erased function"
    );
}

#[test]
fn c292_can_bus_message_filtering_dispatch() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define CAN_MAX_FILTERS 16
#define CAN_RX_QUEUE_SIZE 32
#define CAN_DATA_LEN 8

typedef struct {
    uint32_t id;
    uint8_t data[8];
    uint8_t dlc;
    int is_extended;
    int is_rtr;
} can_msg_t;

typedef struct {
    uint32_t id_mask;
    uint32_t id_filter;
    int active;
} can_filter_t;

typedef struct {
    can_filter_t filters[16];
    int filter_count;
    can_msg_t rx_queue[32];
    int rx_head;
    int rx_tail;
    int rx_count;
    uint32_t rx_total;
    uint32_t rx_dropped;
    uint32_t tx_total;
    uint32_t error_count;
} can_bus_t;

void can_init(can_bus_t *can) {
    can->filter_count = 0;
    can->rx_head = 0;
    can->rx_tail = 0;
    can->rx_count = 0;
    can->rx_total = 0;
    can->rx_dropped = 0;
    can->tx_total = 0;
    can->error_count = 0;
    for (int i = 0; i < 16; i++) {
        can->filters[i].active = 0;
    }
}

int can_add_filter(can_bus_t *can, uint32_t id_filter, uint32_t id_mask) {
    if (can->filter_count >= 16) return -1;
    int idx = can->filter_count;
    can->filters[idx].id_filter = id_filter;
    can->filters[idx].id_mask = id_mask;
    can->filters[idx].active = 1;
    can->filter_count++;
    return idx;
}

int can_msg_matches_filter(const can_bus_t *can, uint32_t msg_id) {
    for (int i = 0; i < can->filter_count; i++) {
        if (!can->filters[i].active) continue;
        if ((msg_id & can->filters[i].id_mask) ==
            (can->filters[i].id_filter & can->filters[i].id_mask)) {
            return i;
        }
    }
    return -1;
}

int can_receive(can_bus_t *can, uint32_t id, const uint8_t *data, uint8_t dlc,
                int is_extended, int is_rtr) {
    can->rx_total++;
    if (can_msg_matches_filter(can, id) < 0) return 0;
    if (can->rx_count >= 32) {
        can->rx_dropped++;
        return -1;
    }
    can_msg_t *msg = &can->rx_queue[can->rx_head];
    msg->id = id;
    msg->dlc = dlc;
    msg->is_extended = is_extended;
    msg->is_rtr = is_rtr;
    for (int i = 0; i < dlc && i < 8; i++) {
        msg->data[i] = data[i];
    }
    can->rx_head = (can->rx_head + 1) % 32;
    can->rx_count++;
    return 1;
}

int can_read(can_bus_t *can, can_msg_t *msg) {
    if (can->rx_count == 0) return -1;
    *msg = can->rx_queue[can->rx_tail];
    can->rx_tail = (can->rx_tail + 1) % 32;
    can->rx_count--;
    return 0;
}

int can_rx_pending(const can_bus_t *can) {
    return can->rx_count;
}

uint32_t can_get_rx_total(const can_bus_t *can) {
    return can->rx_total;
}

uint32_t can_get_drop_rate(const can_bus_t *can) {
    if (can->rx_total == 0) return 0;
    return (can->rx_dropped * 100) / can->rx_total;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C292: CAN bus message filtering and dispatch should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C292: Output should not be empty");
    assert!(
        code.contains("fn can_init"),
        "C292: Should contain can_init function"
    );
    assert!(
        code.contains("fn can_add_filter"),
        "C292: Should contain can_add_filter function"
    );
    assert!(
        code.contains("fn can_receive"),
        "C292: Should contain can_receive function"
    );
    assert!(
        code.contains("fn can_read"),
        "C292: Should contain can_read function"
    );
}

#[test]
fn c293_rtc_alarm_configuration() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define MAX_ALARMS 8

typedef struct {
    uint8_t hours;
    uint8_t minutes;
    uint8_t seconds;
    uint8_t day;
    uint8_t month;
    uint8_t year;
    uint8_t weekday;
} rtc_time_t;

typedef struct {
    rtc_time_t match_time;
    int active;
    int repeat_daily;
    uint32_t trigger_count;
} rtc_alarm_t;

typedef struct {
    rtc_time_t current;
    rtc_alarm_t alarms[8];
    int alarm_count;
    uint32_t tick_count;
} rtc_t;

void rtc_init(rtc_t *rtc) {
    rtc->current.hours = 0;
    rtc->current.minutes = 0;
    rtc->current.seconds = 0;
    rtc->current.day = 1;
    rtc->current.month = 1;
    rtc->current.year = 0;
    rtc->current.weekday = 0;
    rtc->alarm_count = 0;
    rtc->tick_count = 0;
    for (int i = 0; i < 8; i++) {
        rtc->alarms[i].active = 0;
        rtc->alarms[i].trigger_count = 0;
    }
}

void rtc_set_time(rtc_t *rtc, uint8_t h, uint8_t m, uint8_t s) {
    rtc->current.hours = h;
    rtc->current.minutes = m;
    rtc->current.seconds = s;
}

void rtc_set_date(rtc_t *rtc, uint8_t y, uint8_t mon, uint8_t d) {
    rtc->current.year = y;
    rtc->current.month = mon;
    rtc->current.day = d;
}

int rtc_add_alarm(rtc_t *rtc, uint8_t h, uint8_t m, uint8_t s, int repeat) {
    if (rtc->alarm_count >= 8) return -1;
    int idx = rtc->alarm_count;
    rtc->alarms[idx].match_time.hours = h;
    rtc->alarms[idx].match_time.minutes = m;
    rtc->alarms[idx].match_time.seconds = s;
    rtc->alarms[idx].active = 1;
    rtc->alarms[idx].repeat_daily = repeat;
    rtc->alarms[idx].trigger_count = 0;
    rtc->alarm_count++;
    return idx;
}

void rtc_tick_second(rtc_t *rtc) {
    rtc->tick_count++;
    rtc->current.seconds++;
    if (rtc->current.seconds >= 60) {
        rtc->current.seconds = 0;
        rtc->current.minutes++;
        if (rtc->current.minutes >= 60) {
            rtc->current.minutes = 0;
            rtc->current.hours++;
            if (rtc->current.hours >= 24) {
                rtc->current.hours = 0;
                rtc->current.day++;
            }
        }
    }
}

int rtc_check_alarms(rtc_t *rtc) {
    int fired = 0;
    for (int i = 0; i < rtc->alarm_count; i++) {
        if (!rtc->alarms[i].active) continue;
        if (rtc->alarms[i].match_time.hours == rtc->current.hours &&
            rtc->alarms[i].match_time.minutes == rtc->current.minutes &&
            rtc->alarms[i].match_time.seconds == rtc->current.seconds) {
            rtc->alarms[i].trigger_count++;
            fired++;
            if (!rtc->alarms[i].repeat_daily) {
                rtc->alarms[i].active = 0;
            }
        }
    }
    return fired;
}

int rtc_cancel_alarm(rtc_t *rtc, int alarm_id) {
    if (alarm_id < 0 || alarm_id >= rtc->alarm_count) return -1;
    rtc->alarms[alarm_id].active = 0;
    return 0;
}

uint32_t rtc_seconds_since_midnight(const rtc_t *rtc) {
    return (uint32_t)rtc->current.hours * 3600 +
           (uint32_t)rtc->current.minutes * 60 +
           (uint32_t)rtc->current.seconds;
}

int rtc_active_alarm_count(const rtc_t *rtc) {
    int count = 0;
    for (int i = 0; i < rtc->alarm_count; i++) {
        if (rtc->alarms[i].active) count++;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C293: RTC alarm configuration should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C293: Output should not be empty");
    assert!(
        code.contains("fn rtc_init"),
        "C293: Should contain rtc_init function"
    );
    assert!(
        code.contains("fn rtc_add_alarm"),
        "C293: Should contain rtc_add_alarm function"
    );
    assert!(
        code.contains("fn rtc_tick_second"),
        "C293: Should contain rtc_tick_second function"
    );
    assert!(
        code.contains("fn rtc_check_alarms"),
        "C293: Should contain rtc_check_alarms function"
    );
}

#[test]
fn c294_power_management_state_machine() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define PM_STATE_ACTIVE  0
#define PM_STATE_IDLE    1
#define PM_STATE_SLEEP   2
#define PM_STATE_DEEP_SLEEP 3
#define PM_STATE_STANDBY 4

#define PM_WAKEUP_TIMER  1
#define PM_WAKEUP_IRQ    2
#define PM_WAKEUP_PIN    4
#define PM_WAKEUP_RTC    8

typedef struct {
    int current_state;
    int previous_state;
    uint32_t wakeup_sources;
    uint32_t idle_timeout_ms;
    uint32_t sleep_timeout_ms;
    uint32_t idle_counter_ms;
    uint32_t state_duration_ms;
    uint32_t total_active_ms;
    uint32_t total_sleep_ms;
    uint32_t transition_count;
} power_mgr_t;

void pm_init(power_mgr_t *pm) {
    pm->current_state = PM_STATE_ACTIVE;
    pm->previous_state = PM_STATE_ACTIVE;
    pm->wakeup_sources = PM_WAKEUP_TIMER | PM_WAKEUP_IRQ;
    pm->idle_timeout_ms = 1000;
    pm->sleep_timeout_ms = 10000;
    pm->idle_counter_ms = 0;
    pm->state_duration_ms = 0;
    pm->total_active_ms = 0;
    pm->total_sleep_ms = 0;
    pm->transition_count = 0;
}

int pm_transition(power_mgr_t *pm, int new_state) {
    if (new_state < PM_STATE_ACTIVE || new_state > PM_STATE_STANDBY) return -1;
    if (new_state == pm->current_state) return 0;
    if (pm->current_state == PM_STATE_ACTIVE) {
        pm->total_active_ms += pm->state_duration_ms;
    } else {
        pm->total_sleep_ms += pm->state_duration_ms;
    }
    pm->previous_state = pm->current_state;
    pm->current_state = new_state;
    pm->state_duration_ms = 0;
    pm->idle_counter_ms = 0;
    pm->transition_count++;
    return 1;
}

void pm_tick(power_mgr_t *pm, uint32_t elapsed_ms) {
    pm->state_duration_ms += elapsed_ms;
    if (pm->current_state == PM_STATE_ACTIVE) {
        pm->idle_counter_ms += elapsed_ms;
        if (pm->idle_counter_ms >= pm->idle_timeout_ms) {
            pm_transition(pm, PM_STATE_IDLE);
        }
    } else if (pm->current_state == PM_STATE_IDLE) {
        pm->idle_counter_ms += elapsed_ms;
        if (pm->idle_counter_ms >= pm->sleep_timeout_ms) {
            pm_transition(pm, PM_STATE_SLEEP);
        }
    }
}

void pm_activity(power_mgr_t *pm) {
    pm->idle_counter_ms = 0;
    if (pm->current_state != PM_STATE_ACTIVE) {
        pm_transition(pm, PM_STATE_ACTIVE);
    }
}

int pm_wakeup(power_mgr_t *pm, uint32_t source) {
    if (!(pm->wakeup_sources & source)) return -1;
    pm_transition(pm, PM_STATE_ACTIVE);
    return 0;
}

void pm_set_wakeup_sources(power_mgr_t *pm, uint32_t sources) {
    pm->wakeup_sources = sources;
}

int pm_get_state(const power_mgr_t *pm) {
    return pm->current_state;
}

uint32_t pm_get_active_percent(const power_mgr_t *pm) {
    uint32_t total = pm->total_active_ms + pm->total_sleep_ms;
    if (total == 0) return 100;
    return (pm->total_active_ms * 100) / total;
}

uint32_t pm_get_transition_count(const power_mgr_t *pm) {
    return pm->transition_count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C294: Power management state machine should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C294: Output should not be empty");
    assert!(
        code.contains("fn pm_init"),
        "C294: Should contain pm_init function"
    );
    assert!(
        code.contains("fn pm_transition"),
        "C294: Should contain pm_transition function"
    );
    assert!(
        code.contains("fn pm_tick"),
        "C294: Should contain pm_tick function"
    );
    assert!(
        code.contains("fn pm_get_active_percent"),
        "C294: Should contain pm_get_active_percent function"
    );
}

#[test]
fn c295_bootloader_jump_table_function_pointers() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define BOOT_MAX_ENTRIES 16
#define BOOT_MAGIC 0xDEADBEEF

typedef int (*boot_fn_t)(void *arg);

typedef struct {
    uint32_t magic;
    uint32_t version;
    boot_fn_t entries[16];
    int entry_count;
    uint32_t checksum;
} boot_table_t;

typedef struct {
    uint32_t app_start_addr;
    uint32_t app_size;
    uint32_t app_checksum;
    int verified;
} boot_image_t;

void boot_table_init(boot_table_t *bt) {
    bt->magic = BOOT_MAGIC;
    bt->version = 1;
    bt->entry_count = 0;
    bt->checksum = 0;
    for (int i = 0; i < 16; i++) {
        bt->entries[i] = 0;
    }
}

int boot_register_entry(boot_table_t *bt, boot_fn_t fn) {
    if (bt->entry_count >= 16) return -1;
    bt->entries[bt->entry_count] = fn;
    bt->entry_count++;
    return bt->entry_count - 1;
}

int boot_validate_table(const boot_table_t *bt) {
    if (bt->magic != BOOT_MAGIC) return -1;
    if (bt->entry_count > 16) return -2;
    if (bt->version == 0) return -3;
    return 0;
}

void boot_image_init(boot_image_t *img, uint32_t start, uint32_t size) {
    img->app_start_addr = start;
    img->app_size = size;
    img->app_checksum = 0;
    img->verified = 0;
}

uint32_t boot_compute_checksum(const uint32_t *data, int words) {
    uint32_t sum = 0;
    for (int i = 0; i < words; i++) {
        sum = sum * 31 + data[i];
    }
    return sum;
}

int boot_verify_image(boot_image_t *img, uint32_t expected_checksum) {
    if (img->app_size == 0) return -1;
    if (img->app_checksum == expected_checksum) {
        img->verified = 1;
        return 0;
    }
    return -2;
}

int boot_is_verified(const boot_image_t *img) {
    return img->verified;
}

int boot_get_entry_count(const boot_table_t *bt) {
    return bt->entry_count;
}

uint32_t boot_get_version(const boot_table_t *bt) {
    return bt->version;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C295: Bootloader jump table with function pointers should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C295: Output should not be empty");
    assert!(
        code.contains("fn boot_table_init"),
        "C295: Should contain boot_table_init function"
    );
    assert!(
        code.contains("fn boot_register_entry"),
        "C295: Should contain boot_register_entry function"
    );
    assert!(
        code.contains("fn boot_validate_table"),
        "C295: Should contain boot_validate_table function"
    );
    assert!(
        code.contains("fn boot_verify_image"),
        "C295: Should contain boot_verify_image function"
    );
}

// ============================================================================
// C296-C300: Stack Guard, Memory Pool, Event Flags, Scheduling, HAL
// ============================================================================

#[test]
fn c296_stack_canary_guard_overflow_detection() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define STACK_CANARY_VALUE 0xDEADC0DE
#define MAX_TASK_STACKS 16
#define STACK_WORDS 256

typedef struct {
    uint32_t stack[256];
    uint32_t canary_top;
    uint32_t canary_bottom;
    int stack_pointer;
    int task_id;
    int overflow_detected;
} guarded_stack_t;

typedef struct {
    guarded_stack_t stacks[16];
    int stack_count;
    int total_overflows;
} stack_monitor_t;

void stack_guard_init(guarded_stack_t *gs, int task_id) {
    gs->task_id = task_id;
    gs->canary_top = STACK_CANARY_VALUE;
    gs->canary_bottom = STACK_CANARY_VALUE;
    gs->stack_pointer = 256;
    gs->overflow_detected = 0;
    for (int i = 0; i < 256; i++) {
        gs->stack[i] = 0;
    }
}

int stack_check_canary(const guarded_stack_t *gs) {
    if (gs->canary_top != STACK_CANARY_VALUE) return -1;
    if (gs->canary_bottom != STACK_CANARY_VALUE) return -2;
    return 0;
}

int stack_push(guarded_stack_t *gs, uint32_t value) {
    if (gs->stack_pointer <= 0) {
        gs->overflow_detected = 1;
        return -1;
    }
    gs->stack_pointer--;
    gs->stack[gs->stack_pointer] = value;
    return 0;
}

int stack_pop(guarded_stack_t *gs, uint32_t *value) {
    if (gs->stack_pointer >= 256) return -1;
    *value = gs->stack[gs->stack_pointer];
    gs->stack_pointer++;
    return 0;
}

int stack_usage_words(const guarded_stack_t *gs) {
    return 256 - gs->stack_pointer;
}

int stack_usage_percent(const guarded_stack_t *gs) {
    return ((256 - gs->stack_pointer) * 100) / 256;
}

void monitor_init(stack_monitor_t *mon) {
    mon->stack_count = 0;
    mon->total_overflows = 0;
}

int monitor_add_stack(stack_monitor_t *mon, int task_id) {
    if (mon->stack_count >= 16) return -1;
    int idx = mon->stack_count;
    stack_guard_init(&mon->stacks[idx], task_id);
    mon->stack_count++;
    return idx;
}

int monitor_check_all(stack_monitor_t *mon) {
    int violations = 0;
    for (int i = 0; i < mon->stack_count; i++) {
        if (stack_check_canary(&mon->stacks[i]) != 0) {
            mon->stacks[i].overflow_detected = 1;
            violations++;
        }
    }
    mon->total_overflows += violations;
    return violations;
}

int monitor_max_usage_percent(const stack_monitor_t *mon) {
    int max_pct = 0;
    for (int i = 0; i < mon->stack_count; i++) {
        int pct = stack_usage_percent(&mon->stacks[i]);
        if (pct > max_pct) max_pct = pct;
    }
    return max_pct;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C296: Stack canary/guard overflow detection should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C296: Output should not be empty");
    assert!(
        code.contains("fn stack_guard_init"),
        "C296: Should contain stack_guard_init function"
    );
    assert!(
        code.contains("fn stack_check_canary"),
        "C296: Should contain stack_check_canary function"
    );
    assert!(
        code.contains("fn monitor_check_all"),
        "C296: Should contain monitor_check_all function"
    );
}

#[test]
fn c297_memory_pool_fixed_size_block_allocator() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define POOL_BLOCK_SIZE 64
#define POOL_NUM_BLOCKS 128

typedef struct {
    uint8_t blocks[128][64];
    uint8_t alloc_bitmap[128];
    int free_list[128];
    int free_head;
    int free_count;
    uint32_t alloc_count;
    uint32_t free_op_count;
    uint32_t high_watermark;
} mem_pool_t;

void mpool_init(mem_pool_t *pool) {
    pool->free_count = 128;
    pool->free_head = 0;
    pool->alloc_count = 0;
    pool->free_op_count = 0;
    pool->high_watermark = 0;
    for (int i = 0; i < 128; i++) {
        pool->alloc_bitmap[i] = 0;
        pool->free_list[i] = i + 1;
    }
    pool->free_list[127] = -1;
}

int mpool_alloc(mem_pool_t *pool) {
    if (pool->free_head < 0 || pool->free_count == 0) return -1;
    int idx = pool->free_head;
    pool->free_head = pool->free_list[idx];
    pool->alloc_bitmap[idx] = 1;
    pool->free_count--;
    pool->alloc_count++;
    uint32_t used = 128 - (uint32_t)pool->free_count;
    if (used > pool->high_watermark) {
        pool->high_watermark = used;
    }
    return idx;
}

int mpool_free(mem_pool_t *pool, int block_id) {
    if (block_id < 0 || block_id >= 128) return -1;
    if (!pool->alloc_bitmap[block_id]) return -2;
    pool->alloc_bitmap[block_id] = 0;
    pool->free_list[block_id] = pool->free_head;
    pool->free_head = block_id;
    pool->free_count++;
    pool->free_op_count++;
    return 0;
}

int mpool_write(mem_pool_t *pool, int block_id, const uint8_t *data, int len) {
    if (block_id < 0 || block_id >= 128) return -1;
    if (!pool->alloc_bitmap[block_id]) return -2;
    if (len > 64) len = 64;
    for (int i = 0; i < len; i++) {
        pool->blocks[block_id][i] = data[i];
    }
    return len;
}

int mpool_read(const mem_pool_t *pool, int block_id, uint8_t *data, int len) {
    if (block_id < 0 || block_id >= 128) return -1;
    if (!pool->alloc_bitmap[block_id]) return -2;
    if (len > 64) len = 64;
    for (int i = 0; i < len; i++) {
        data[i] = pool->blocks[block_id][i];
    }
    return len;
}

int mpool_available(const mem_pool_t *pool) {
    return pool->free_count;
}

int mpool_is_allocated(const mem_pool_t *pool, int block_id) {
    if (block_id < 0 || block_id >= 128) return 0;
    return pool->alloc_bitmap[block_id];
}

uint32_t mpool_get_high_watermark(const mem_pool_t *pool) {
    return pool->high_watermark;
}

int mpool_fragmentation_percent(const mem_pool_t *pool) {
    if (pool->free_count == 0) return 0;
    int contiguous = 0;
    int max_contiguous = 0;
    for (int i = 0; i < 128; i++) {
        if (!pool->alloc_bitmap[i]) {
            contiguous++;
            if (contiguous > max_contiguous) max_contiguous = contiguous;
        } else {
            contiguous = 0;
        }
    }
    if (max_contiguous == 0) return 100;
    return 100 - (max_contiguous * 100) / pool->free_count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C297: Memory pool fixed-size block allocator should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C297: Output should not be empty");
    assert!(
        code.contains("fn mpool_init"),
        "C297: Should contain mpool_init function"
    );
    assert!(
        code.contains("fn mpool_alloc"),
        "C297: Should contain mpool_alloc function"
    );
    assert!(
        code.contains("fn mpool_free"),
        "C297: Should contain mpool_free function"
    );
    assert!(
        code.contains("fn mpool_fragmentation_percent"),
        "C297: Should contain mpool_fragmentation_percent function"
    );
}

#[test]
fn c298_event_flag_group_bitwise_signaling() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_EVENT_GROUPS 8
#define EVENT_WAIT_ALL 0
#define EVENT_WAIT_ANY 1

typedef struct {
    uint32_t flags;
    uint32_t set_count;
    uint32_t clear_count;
    uint32_t wait_mask;
    int wait_mode;
} event_group_t;

typedef struct {
    event_group_t groups[8];
    int group_count;
} event_system_t;

void evt_sys_init(event_system_t *sys) {
    sys->group_count = 0;
}

int evt_create_group(event_system_t *sys) {
    if (sys->group_count >= 8) return -1;
    int idx = sys->group_count;
    sys->groups[idx].flags = 0;
    sys->groups[idx].set_count = 0;
    sys->groups[idx].clear_count = 0;
    sys->groups[idx].wait_mask = 0;
    sys->groups[idx].wait_mode = EVENT_WAIT_ANY;
    sys->group_count++;
    return idx;
}

int evt_set_flags(event_system_t *sys, int group_id, uint32_t flags) {
    if (group_id < 0 || group_id >= sys->group_count) return -1;
    sys->groups[group_id].flags |= flags;
    sys->groups[group_id].set_count++;
    return 0;
}

int evt_clear_flags(event_system_t *sys, int group_id, uint32_t flags) {
    if (group_id < 0 || group_id >= sys->group_count) return -1;
    sys->groups[group_id].flags &= ~flags;
    sys->groups[group_id].clear_count++;
    return 0;
}

uint32_t evt_get_flags(const event_system_t *sys, int group_id) {
    if (group_id < 0 || group_id >= sys->group_count) return 0;
    return sys->groups[group_id].flags;
}

int evt_wait_check(const event_system_t *sys, int group_id, uint32_t mask, int mode) {
    if (group_id < 0 || group_id >= sys->group_count) return -1;
    uint32_t matched = sys->groups[group_id].flags & mask;
    if (mode == EVENT_WAIT_ALL) {
        return (matched == mask) ? 1 : 0;
    } else {
        return (matched != 0) ? 1 : 0;
    }
}

int evt_set_wait(event_system_t *sys, int group_id, uint32_t mask, int mode) {
    if (group_id < 0 || group_id >= sys->group_count) return -1;
    sys->groups[group_id].wait_mask = mask;
    sys->groups[group_id].wait_mode = mode;
    return 0;
}

int evt_pulse_flags(event_system_t *sys, int group_id, uint32_t flags) {
    if (group_id < 0 || group_id >= sys->group_count) return -1;
    sys->groups[group_id].flags |= flags;
    sys->groups[group_id].set_count++;
    sys->groups[group_id].flags &= ~flags;
    sys->groups[group_id].clear_count++;
    return 0;
}

int evt_count_set_bits(const event_system_t *sys, int group_id) {
    if (group_id < 0 || group_id >= sys->group_count) return -1;
    uint32_t f = sys->groups[group_id].flags;
    int count = 0;
    while (f) {
        count += (int)(f & 1U);
        f >>= 1;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C298: Event flag group bitwise signaling should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C298: Output should not be empty");
    assert!(
        code.contains("fn evt_sys_init"),
        "C298: Should contain evt_sys_init function"
    );
    assert!(
        code.contains("fn evt_set_flags"),
        "C298: Should contain evt_set_flags function"
    );
    assert!(
        code.contains("fn evt_wait_check"),
        "C298: Should contain evt_wait_check function"
    );
    assert!(
        code.contains("fn evt_count_set_bits"),
        "C298: Should contain evt_count_set_bits function"
    );
}

#[test]
fn c299_deadline_monotonic_scheduling() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define RMS_MAX_TASKS 16

typedef struct {
    int task_id;
    uint32_t period_ms;
    uint32_t deadline_ms;
    uint32_t wcet_ms;
    uint32_t next_release;
    uint32_t next_deadline;
    int active;
    uint32_t completions;
    uint32_t deadline_misses;
} rms_task_t;

typedef struct {
    rms_task_t tasks[16];
    int task_count;
    uint32_t current_tick;
    int running_task;
    uint32_t total_misses;
} rms_scheduler_t;

void rms_init(rms_scheduler_t *s) {
    s->task_count = 0;
    s->current_tick = 0;
    s->running_task = -1;
    s->total_misses = 0;
}

int rms_add_task(rms_scheduler_t *s, uint32_t period, uint32_t deadline, uint32_t wcet) {
    if (s->task_count >= 16) return -1;
    int idx = s->task_count;
    s->tasks[idx].task_id = idx;
    s->tasks[idx].period_ms = period;
    s->tasks[idx].deadline_ms = deadline;
    s->tasks[idx].wcet_ms = wcet;
    s->tasks[idx].next_release = 0;
    s->tasks[idx].next_deadline = deadline;
    s->tasks[idx].active = 1;
    s->tasks[idx].completions = 0;
    s->tasks[idx].deadline_misses = 0;
    s->task_count++;
    return idx;
}

int rms_select_task(const rms_scheduler_t *s) {
    int best = -1;
    uint32_t shortest_period = 0xFFFFFFFF;
    for (int i = 0; i < s->task_count; i++) {
        if (!s->tasks[i].active) continue;
        if (s->current_tick < s->tasks[i].next_release) continue;
        if (s->tasks[i].period_ms < shortest_period) {
            shortest_period = s->tasks[i].period_ms;
            best = i;
        }
    }
    return best;
}

void rms_tick(rms_scheduler_t *s) {
    s->current_tick++;
    for (int i = 0; i < s->task_count; i++) {
        if (!s->tasks[i].active) continue;
        if (s->current_tick >= s->tasks[i].next_deadline &&
            s->current_tick > s->tasks[i].next_release) {
            s->tasks[i].deadline_misses++;
            s->total_misses++;
            s->tasks[i].next_release += s->tasks[i].period_ms;
            s->tasks[i].next_deadline = s->tasks[i].next_release + s->tasks[i].deadline_ms;
        }
    }
}

void rms_complete_task(rms_scheduler_t *s, int task_id) {
    if (task_id < 0 || task_id >= s->task_count) return;
    s->tasks[task_id].completions++;
    s->tasks[task_id].next_release += s->tasks[task_id].period_ms;
    s->tasks[task_id].next_deadline = s->tasks[task_id].next_release + s->tasks[task_id].deadline_ms;
}

int rms_utilization_check(const rms_scheduler_t *s) {
    uint32_t util_num = 0;
    uint32_t util_den = 1;
    for (int i = 0; i < s->task_count; i++) {
        if (!s->tasks[i].active) continue;
        if (s->tasks[i].period_ms == 0) continue;
        util_num += (s->tasks[i].wcet_ms * 1000) / s->tasks[i].period_ms;
    }
    if (util_num > 693 * (uint32_t)s->task_count / (uint32_t)s->task_count) {
        return 0;
    }
    return 1;
}

uint32_t rms_get_miss_count(const rms_scheduler_t *s) {
    return s->total_misses;
}

int rms_get_task_misses(const rms_scheduler_t *s, int task_id) {
    if (task_id < 0 || task_id >= s->task_count) return -1;
    return (int)s->tasks[task_id].deadline_misses;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C299: Deadline monotonic scheduling should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C299: Output should not be empty");
    assert!(
        code.contains("fn rms_init"),
        "C299: Should contain rms_init function"
    );
    assert!(
        code.contains("fn rms_add_task"),
        "C299: Should contain rms_add_task function"
    );
    assert!(
        code.contains("fn rms_select_task"),
        "C299: Should contain rms_select_task function"
    );
    assert!(
        code.contains("fn rms_tick"),
        "C299: Should contain rms_tick function"
    );
}

#[test]
fn c300_hardware_abstraction_layer_vtable() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef int (*hal_init_fn)(void *hw_ctx);
typedef int (*hal_read_fn)(void *hw_ctx, uint8_t *buf, int len);
typedef int (*hal_write_fn)(void *hw_ctx, const uint8_t *buf, int len);
typedef int (*hal_ioctl_fn)(void *hw_ctx, int cmd, uint32_t arg);
typedef void (*hal_close_fn)(void *hw_ctx);

typedef struct {
    hal_init_fn init;
    hal_read_fn read;
    hal_write_fn write;
    hal_ioctl_fn ioctl;
    hal_close_fn close;
} hal_ops_t;

typedef struct {
    const hal_ops_t *ops;
    void *hw_context;
    int is_open;
    uint32_t read_count;
    uint32_t write_count;
    uint32_t error_count;
    int last_error;
} hal_device_t;

#define MAX_DEVICES 8

typedef struct {
    hal_device_t devices[8];
    int device_count;
} hal_registry_t;

void hal_registry_init(hal_registry_t *reg) {
    reg->device_count = 0;
    for (int i = 0; i < 8; i++) {
        reg->devices[i].ops = 0;
        reg->devices[i].hw_context = 0;
        reg->devices[i].is_open = 0;
        reg->devices[i].read_count = 0;
        reg->devices[i].write_count = 0;
        reg->devices[i].error_count = 0;
        reg->devices[i].last_error = 0;
    }
}

int hal_register_device(hal_registry_t *reg, const hal_ops_t *ops, void *ctx) {
    if (reg->device_count >= 8) return -1;
    int idx = reg->device_count;
    reg->devices[idx].ops = ops;
    reg->devices[idx].hw_context = ctx;
    reg->devices[idx].is_open = 0;
    reg->devices[idx].read_count = 0;
    reg->devices[idx].write_count = 0;
    reg->devices[idx].error_count = 0;
    reg->device_count++;
    return idx;
}

int hal_open(hal_registry_t *reg, int dev_id) {
    if (dev_id < 0 || dev_id >= reg->device_count) return -1;
    hal_device_t *dev = &reg->devices[dev_id];
    if (dev->is_open) return -2;
    if (dev->ops && dev->ops->init) {
        int rc = dev->ops->init(dev->hw_context);
        if (rc != 0) {
            dev->last_error = rc;
            dev->error_count++;
            return rc;
        }
    }
    dev->is_open = 1;
    return 0;
}

int hal_read(hal_registry_t *reg, int dev_id, uint8_t *buf, int len) {
    if (dev_id < 0 || dev_id >= reg->device_count) return -1;
    hal_device_t *dev = &reg->devices[dev_id];
    if (!dev->is_open) return -2;
    if (!dev->ops || !dev->ops->read) return -3;
    int rc = dev->ops->read(dev->hw_context, buf, len);
    if (rc >= 0) {
        dev->read_count++;
    } else {
        dev->error_count++;
        dev->last_error = rc;
    }
    return rc;
}

int hal_write(hal_registry_t *reg, int dev_id, const uint8_t *buf, int len) {
    if (dev_id < 0 || dev_id >= reg->device_count) return -1;
    hal_device_t *dev = &reg->devices[dev_id];
    if (!dev->is_open) return -2;
    if (!dev->ops || !dev->ops->write) return -3;
    int rc = dev->ops->write(dev->hw_context, buf, len);
    if (rc >= 0) {
        dev->write_count++;
    } else {
        dev->error_count++;
        dev->last_error = rc;
    }
    return rc;
}

void hal_close(hal_registry_t *reg, int dev_id) {
    if (dev_id < 0 || dev_id >= reg->device_count) return;
    hal_device_t *dev = &reg->devices[dev_id];
    if (!dev->is_open) return;
    if (dev->ops && dev->ops->close) {
        dev->ops->close(dev->hw_context);
    }
    dev->is_open = 0;
}

int hal_device_count(const hal_registry_t *reg) {
    return reg->device_count;
}

int hal_is_open(const hal_registry_t *reg, int dev_id) {
    if (dev_id < 0 || dev_id >= reg->device_count) return 0;
    return reg->devices[dev_id].is_open;
}

uint32_t hal_get_error_count(const hal_registry_t *reg, int dev_id) {
    if (dev_id < 0 || dev_id >= reg->device_count) return 0;
    return reg->devices[dev_id].error_count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C300: Hardware abstraction layer vtable should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C300: Output should not be empty");
    assert!(
        code.contains("fn hal_registry_init"),
        "C300: Should contain hal_registry_init function"
    );
    assert!(
        code.contains("fn hal_register_device"),
        "C300: Should contain hal_register_device function"
    );
    assert!(
        code.contains("fn hal_open"),
        "C300: Should contain hal_open function"
    );
    assert!(
        code.contains("fn hal_read"),
        "C300: Should contain hal_read function"
    );
    assert!(
        code.contains("fn hal_write"),
        "C300: Should contain hal_write function"
    );
    assert!(
        code.contains("fn hal_close"),
        "C300: Should contain hal_close function"
    );
}
