//! Falsification tests for Virtual Machine / CPU emulation patterns in C.
//! Tests C1401-C1425: register files, instruction decode, memory systems, I/O, VM management.
//! Append-only per Popperian falsification methodology.

use decy_core::transpile;

// =============================================================================
// Category 1: CPU Emulation (C1401-C1405)
// =============================================================================

#[test]
fn c1401_register_file() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef struct { uint32_t regs[16]; uint32_t pc; uint32_t flags; } vm2_cpu_t;
void vm2_cpu_init(vm2_cpu_t *c) { int i; for(i=0;i<16;i++) c->regs[i]=0; c->pc=0; c->flags=0; }
uint32_t vm2_cpu_get(vm2_cpu_t *c, int r) { return (r>=0 && r<16) ? c->regs[r] : 0; }
void vm2_cpu_set(vm2_cpu_t *c, int r, uint32_t v) { if(r>=0 && r<16) c->regs[r]=v; }
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1401 failed: {:?}", result.err());
}

#[test]
fn c1402_alu_operations() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef struct { uint32_t a; uint32_t b; uint32_t result; int zero; int carry; } vm2_alu_t;
void vm2_alu_add(vm2_alu_t *a) { a->result = a->a + a->b; a->zero = (a->result == 0); a->carry = (a->result < a->a); }
void vm2_alu_sub(vm2_alu_t *a) { a->result = a->a - a->b; a->zero = (a->result == 0); a->carry = (a->a < a->b); }
void vm2_alu_and(vm2_alu_t *a) { a->result = a->a & a->b; a->zero = (a->result == 0); a->carry = 0; }
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1402 failed: {:?}", result.err());
}

#[test]
fn c1403_branch_jump() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef struct { uint32_t pc; uint32_t flags; } vm2_ctrl_t;
void vm2_jump(vm2_ctrl_t *c, uint32_t addr) { c->pc = addr; }
void vm2_branch_if_zero(vm2_ctrl_t *c, uint32_t addr) { if(c->flags == 0) c->pc = addr; else c->pc += 4; }
void vm2_call(vm2_ctrl_t *c, uint32_t *sp, uint32_t addr) { *sp -= 4; c->pc = addr; }
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1403 failed: {:?}", result.err());
}

#[test]
fn c1404_memory_bus() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;
typedef struct { uint8_t data[4096]; int size; } vm2_bus_t;
void vm2_bus_init(vm2_bus_t *b) { int i; for(i=0;i<4096;i++) b->data[i]=0; b->size=4096; }
uint8_t vm2_bus_read(vm2_bus_t *b, uint32_t addr) { return (addr < 4096) ? b->data[addr] : 0; }
void vm2_bus_write(vm2_bus_t *b, uint32_t addr, uint8_t val) { if(addr < 4096) b->data[addr] = val; }
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1404 failed: {:?}", result.err());
}

#[test]
fn c1405_interrupt_controller() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef struct { uint32_t pending; uint32_t mask; int enabled; } vm2_intc_t;
void vm2_intc_init(vm2_intc_t *ic) { ic->pending=0; ic->mask=0; ic->enabled=0; }
void vm2_intc_raise(vm2_intc_t *ic, int irq) { if(irq>=0 && irq<32) ic->pending |= (1u << irq); }
int vm2_intc_poll(vm2_intc_t *ic) { uint32_t active; int i; if(!ic->enabled) return -1; active = ic->pending & ~ic->mask; for(i=0;i<32;i++) if(active & (1u<<i)) return i; return -1; }
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1405 failed: {:?}", result.err());
}

// =============================================================================
// Category 2: Instruction Decode (C1406-C1410)
// =============================================================================

#[test]
fn c1406_opcode_decode() {
    let c_code = r#"
typedef unsigned int uint32_t;
int vm2_decode_op(uint32_t instr) { return (int)((instr >> 24) & 0xFF); }
int vm2_decode_rd(uint32_t instr) { return (int)((instr >> 20) & 0x0F); }
int vm2_decode_rs(uint32_t instr) { return (int)((instr >> 16) & 0x0F); }
int vm2_decode_rt(uint32_t instr) { return (int)((instr >> 12) & 0x0F); }
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1406 failed: {:?}", result.err());
}

#[test]
fn c1407_immediate_extraction() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef int int32_t;
uint32_t vm2_imm16(uint32_t instr) { return instr & 0xFFFF; }
int32_t vm2_simm16(uint32_t instr) { uint32_t v = instr & 0xFFFF; return (v & 0x8000) ? (int32_t)(v | 0xFFFF0000u) : (int32_t)v; }
uint32_t vm2_imm26(uint32_t instr) { return instr & 0x03FFFFFF; }
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1407 failed: {:?}", result.err());
}

#[test]
fn c1408_addressing_modes() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef struct { uint32_t regs[16]; } vm2_regs_t;
uint32_t vm2_addr_imm(uint32_t base, uint32_t offset) { return base + offset; }
uint32_t vm2_addr_reg(vm2_regs_t *r, int rb, int ri) { return r->regs[rb] + r->regs[ri]; }
uint32_t vm2_addr_scaled(vm2_regs_t *r, int rb, int ri, int shift) { return r->regs[rb] + (r->regs[ri] << shift); }
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1408 failed: {:?}", result.err());
}

#[test]
fn c1409_instruction_cache() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef struct { uint32_t tag; uint32_t data[4]; int valid; } vm2_icline_t;
typedef struct { vm2_icline_t lines[64]; int hits; int misses; } vm2_icache_t;
void vm2_icache_init(vm2_icache_t *c) { int i; for(i=0;i<64;i++) c->lines[i].valid=0; c->hits=0; c->misses=0; }
int vm2_icache_lookup(vm2_icache_t *c, uint32_t addr) { int idx = (int)((addr >> 4) & 63); if(c->lines[idx].valid && c->lines[idx].tag==(addr>>10)) { c->hits++; return 1; } c->misses++; return 0; }
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1409 failed: {:?}", result.err());
}

#[test]
fn c1410_pipeline_stages() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef struct { uint32_t fetch; uint32_t decode; uint32_t execute; uint32_t writeback; int stall; } vm2_pipe_t;
void vm2_pipe_init(vm2_pipe_t *p) { p->fetch=0; p->decode=0; p->execute=0; p->writeback=0; p->stall=0; }
void vm2_pipe_advance(vm2_pipe_t *p, uint32_t next) { if(!p->stall) { p->writeback=p->execute; p->execute=p->decode; p->decode=p->fetch; p->fetch=next; } }
void vm2_pipe_flush(vm2_pipe_t *p) { p->decode=0; p->execute=0; p->writeback=0; p->stall=0; }
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1410 failed: {:?}", result.err());
}

// =============================================================================
// Category 3: Memory Systems (C1411-C1415)
// =============================================================================

#[test]
fn c1411_mmu_page_table() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef struct { uint32_t phys; int present; int writable; } vm2_pte_t;
typedef struct { vm2_pte_t entries[256]; } vm2_ptable_t;
void vm2_pt_init(vm2_ptable_t *pt) { int i; for(i=0;i<256;i++) { pt->entries[i].phys=0; pt->entries[i].present=0; pt->entries[i].writable=0; } }
int vm2_pt_translate(vm2_ptable_t *pt, uint32_t virt, uint32_t *phys) { int idx=(int)((virt>>12)&0xFF); if(!pt->entries[idx].present) return -1; *phys=pt->entries[idx].phys|(virt&0xFFF); return 0; }
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1411 failed: {:?}", result.err());
}

#[test]
fn c1412_tlb() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef struct { uint32_t vpn; uint32_t ppn; int valid; } vm2_tlb_entry_t;
typedef struct { vm2_tlb_entry_t entries[16]; int hits; int misses; } vm2_tlb_t;
void vm2_tlb_init(vm2_tlb_t *t) { int i; for(i=0;i<16;i++) t->entries[i].valid=0; t->hits=0; t->misses=0; }
int vm2_tlb_lookup(vm2_tlb_t *t, uint32_t vpn, uint32_t *ppn) { int i; for(i=0;i<16;i++) { if(t->entries[i].valid && t->entries[i].vpn==vpn) { *ppn=t->entries[i].ppn; t->hits++; return 1; } } t->misses++; return 0; }
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1412 failed: {:?}", result.err());
}

#[test]
fn c1413_cache_line() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;
typedef struct { uint32_t tag; uint8_t data[32]; int valid; int dirty; } vm2_cline_t;
void vm2_cline_init(vm2_cline_t *cl) { int i; cl->tag=0; cl->valid=0; cl->dirty=0; for(i=0;i<32;i++) cl->data[i]=0; }
int vm2_cline_match(vm2_cline_t *cl, uint32_t tag) { return cl->valid && cl->tag == tag; }
void vm2_cline_fill(vm2_cline_t *cl, uint32_t tag, uint8_t *src) { int i; cl->tag=tag; cl->valid=1; cl->dirty=0; for(i=0;i<32;i++) cl->data[i]=src[i]; }
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1413 failed: {:?}", result.err());
}

#[test]
fn c1414_write_buffer() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef struct { uint32_t addr; uint32_t data; int pending; } vm2_wbuf_entry_t;
typedef struct { vm2_wbuf_entry_t slots[8]; int head; int count; } vm2_wbuf_t;
void vm2_wbuf_init(vm2_wbuf_t *wb) { int i; wb->head=0; wb->count=0; for(i=0;i<8;i++) wb->slots[i].pending=0; }
int vm2_wbuf_push(vm2_wbuf_t *wb, uint32_t addr, uint32_t data) { int idx; if(wb->count>=8) return -1; idx=(wb->head+wb->count)%8; wb->slots[idx].addr=addr; wb->slots[idx].data=data; wb->slots[idx].pending=1; wb->count++; return 0; }
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1414 failed: {:?}", result.err());
}

#[test]
fn c1415_memory_protection() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef struct { uint32_t base; uint32_t limit; int readable; int writable; int executable; } vm2_mprot_t;
void vm2_mprot_set(vm2_mprot_t *m, uint32_t base, uint32_t limit, int r, int w, int x) { m->base=base; m->limit=limit; m->readable=r; m->writable=w; m->executable=x; }
int vm2_mprot_check_read(vm2_mprot_t *m, uint32_t addr) { return (addr>=m->base && addr<m->base+m->limit && m->readable) ? 1 : 0; }
int vm2_mprot_check_write(vm2_mprot_t *m, uint32_t addr) { return (addr>=m->base && addr<m->base+m->limit && m->writable) ? 1 : 0; }
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1415 failed: {:?}", result.err());
}

// =============================================================================
// Category 4: I/O Emulation (C1416-C1420)
// =============================================================================

#[test]
fn c1416_port_io() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;
typedef struct { uint8_t ports[256]; } vm2_pio_t;
void vm2_pio_init(vm2_pio_t *p) { int i; for(i=0;i<256;i++) p->ports[i]=0; }
uint8_t vm2_pio_in(vm2_pio_t *p, uint8_t port) { return p->ports[port]; }
void vm2_pio_out(vm2_pio_t *p, uint8_t port, uint8_t val) { p->ports[port] = val; }
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1416 failed: {:?}", result.err());
}

#[test]
fn c1417_dma_controller() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef struct { uint32_t src; uint32_t dst; uint32_t count; int active; } vm2_dma_ch_t;
typedef struct { vm2_dma_ch_t channels[4]; } vm2_dma_t;
void vm2_dma_init(vm2_dma_t *d) { int i; for(i=0;i<4;i++) { d->channels[i].src=0; d->channels[i].dst=0; d->channels[i].count=0; d->channels[i].active=0; } }
void vm2_dma_setup(vm2_dma_t *d, int ch, uint32_t src, uint32_t dst, uint32_t cnt) { if(ch>=0&&ch<4) { d->channels[ch].src=src; d->channels[ch].dst=dst; d->channels[ch].count=cnt; d->channels[ch].active=1; } }
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1417 failed: {:?}", result.err());
}

#[test]
fn c1418_timer_counter() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef struct { uint32_t counter; uint32_t reload; int running; int irq_pending; } vm2_timer_t;
void vm2_timer_init(vm2_timer_t *t, uint32_t reload) { t->counter=reload; t->reload=reload; t->running=0; t->irq_pending=0; }
void vm2_timer_tick(vm2_timer_t *t) { if(!t->running) return; if(t->counter==0) { t->counter=t->reload; t->irq_pending=1; } else { t->counter--; } }
void vm2_timer_start(vm2_timer_t *t) { t->running=1; }
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1418 failed: {:?}", result.err());
}

#[test]
fn c1419_uart_emulator() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef struct { uint8_t txbuf[16]; uint8_t rxbuf[16]; int tx_head; int tx_count; int rx_head; int rx_count; } vm2_uart_t;
void vm2_uart_init(vm2_uart_t *u) { u->tx_head=0; u->tx_count=0; u->rx_head=0; u->rx_count=0; }
int vm2_uart_send(vm2_uart_t *u, uint8_t byte) { int idx; if(u->tx_count>=16) return -1; idx=(u->tx_head+u->tx_count)%16; u->txbuf[idx]=byte; u->tx_count++; return 0; }
int vm2_uart_recv(vm2_uart_t *u, uint8_t *byte) { if(u->rx_count==0) return -1; *byte=u->rxbuf[u->rx_head]; u->rx_head=(u->rx_head+1)%16; u->rx_count--; return 0; }
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1419 failed: {:?}", result.err());
}

#[test]
fn c1420_interrupt_priority() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef struct { int priority[8]; uint32_t pending; int serving; } vm2_ipri_t;
void vm2_ipri_init(vm2_ipri_t *ip) { int i; for(i=0;i<8;i++) ip->priority[i]=i; ip->pending=0; ip->serving=-1; }
void vm2_ipri_raise(vm2_ipri_t *ip, int irq) { if(irq>=0 && irq<8) ip->pending |= (1u << irq); }
int vm2_ipri_next(vm2_ipri_t *ip) { int best=-1; int best_pri=99; int i; for(i=0;i<8;i++) { if((ip->pending & (1u<<i)) && ip->priority[i]<best_pri) { best=i; best_pri=ip->priority[i]; } } return best; }
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1420 failed: {:?}", result.err());
}

// =============================================================================
// Category 5: VM Management (C1421-C1425)
// =============================================================================

#[test]
fn c1421_snapshot_restore() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef struct { uint32_t regs[8]; uint32_t pc; uint32_t flags; } vm2_state_t;
typedef struct { vm2_state_t saved; int valid; } vm2_snap_t;
void vm2_snap_save(vm2_snap_t *s, vm2_state_t *st) { int i; for(i=0;i<8;i++) s->saved.regs[i]=st->regs[i]; s->saved.pc=st->pc; s->saved.flags=st->flags; s->valid=1; }
int vm2_snap_restore(vm2_snap_t *s, vm2_state_t *st) { int i; if(!s->valid) return -1; for(i=0;i<8;i++) st->regs[i]=s->saved.regs[i]; st->pc=s->saved.pc; st->flags=s->saved.flags; return 0; }
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1421 failed: {:?}", result.err());
}

#[test]
fn c1422_breakpoint_manager() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef struct { uint32_t addr; int enabled; } vm2_bp_t;
typedef struct { vm2_bp_t bps[16]; int count; } vm2_bpmgr_t;
void vm2_bpmgr_init(vm2_bpmgr_t *m) { m->count=0; }
int vm2_bpmgr_add(vm2_bpmgr_t *m, uint32_t addr) { if(m->count>=16) return -1; m->bps[m->count].addr=addr; m->bps[m->count].enabled=1; m->count++; return m->count-1; }
int vm2_bpmgr_check(vm2_bpmgr_t *m, uint32_t pc) { int i; for(i=0;i<m->count;i++) { if(m->bps[i].enabled && m->bps[i].addr==pc) return 1; } return 0; }
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1422 failed: {:?}", result.err());
}

#[test]
fn c1423_trace_logging() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef struct { uint32_t pc; uint32_t instr; uint32_t result; } vm2_trace_t;
typedef struct { vm2_trace_t log[128]; int head; int count; } vm2_tracer_t;
void vm2_tracer_init(vm2_tracer_t *t) { t->head=0; t->count=0; }
void vm2_tracer_record(vm2_tracer_t *t, uint32_t pc, uint32_t instr, uint32_t res) { int idx=(t->head+t->count)%128; t->log[idx].pc=pc; t->log[idx].instr=instr; t->log[idx].result=res; if(t->count<128) t->count++; else t->head=(t->head+1)%128; }
int vm2_tracer_count(vm2_tracer_t *t) { return t->count; }
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1423 failed: {:?}", result.err());
}

#[test]
fn c1424_performance_counters() {
    let c_code = r#"
typedef unsigned long uint64_t;
typedef struct { uint64_t cycles; uint64_t instrs; uint64_t cache_hits; uint64_t cache_misses; } vm2_perf_t;
void vm2_perf_init(vm2_perf_t *p) { p->cycles=0; p->instrs=0; p->cache_hits=0; p->cache_misses=0; }
void vm2_perf_tick(vm2_perf_t *p) { p->cycles++; }
void vm2_perf_instr(vm2_perf_t *p) { p->instrs++; }
uint64_t vm2_perf_ipc_x100(vm2_perf_t *p) { return (p->cycles>0) ? (p->instrs*100)/p->cycles : 0; }
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1424 failed: {:?}", result.err());
}

#[test]
fn c1425_device_registry() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef struct { uint32_t base; uint32_t size; int type; int active; } vm2_dev_t;
typedef struct { vm2_dev_t devs[8]; int count; } vm2_devreg_t;
void vm2_devreg_init(vm2_devreg_t *r) { r->count=0; }
int vm2_devreg_add(vm2_devreg_t *r, uint32_t base, uint32_t size, int type) { if(r->count>=8) return -1; r->devs[r->count].base=base; r->devs[r->count].size=size; r->devs[r->count].type=type; r->devs[r->count].active=1; r->count++; return r->count-1; }
int vm2_devreg_find(vm2_devreg_t *r, uint32_t addr) { int i; for(i=0;i<r->count;i++) { if(r->devs[i].active && addr>=r->devs[i].base && addr<r->devs[i].base+r->devs[i].size) return i; } return -1; }
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1425 failed: {:?}", result.err());
}
