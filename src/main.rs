
use std::io;

const MEMSIZE: usize = 65536;               // memory size 64k
const RESET_VECTOR_LOBYTE: usize = 0xfffc;  // reset vector memory location
const RESET_VECTOR_HIBYTE: usize = 0xfffd;
const BREAK_VECTOR_LOBYTE: usize = 0xfffe;  // break vector memory location
const BREAK_VECTOR_HIBYTE: usize = 0xffff;
const STATUS_BIT_INT_DIS: u8 = 0x04;        // interrup disable status bit
const STATUS_FLAGS_BREAK: u8 = 0x10;        // break status bit
const STATUS_FLAGS_UNUSED: u8 = 0x20;       // unused status bit

// instruction text by opcode
const INSTRUCTION_TEXT: [&str; 256] = [
	"BRK",  "ORA","",     "","",        "ORA","ASL",     "","PHP","ORA", "ASL", "","",       "ORA", "ASL", "", // 00
	"BPL",  "ORA","",     "","",        "ORA","ASL",     "","CLC","ORA", "",    "","",       "ORA", "ASL", "", // 10
	"JSR",  "AND","",     "","BIT",     "AND","ROL",     "","PLP","AND", "ROL", "","BIT",    "AND", "ROL", "", // 20
	"BMI",  "AND","",     "","",        "AND","ROL",     "","SEC","AND", "",    "","",       "AND", "ROL", "", // 30
	"RTI",  "EOR","",     "","",        "EOR","LSR",     "","PHA","EOR", "LSR", "","JMP",    "EOR", "LSR", "", // 40
	"BVC",  "EOR","",     "","",        "EOR","LSR",     "","CLI","EOR", "",    "","",       "EOR", "LSR", "", // 50
	"RTS",  "ADC","",     "","",        "ADC","ROR",     "","PLA","ADC", "ROR", "","JMP",    "ADC", "ROR", "", // 60
	"BVS",  "ADC","",     "","",        "ADC","ROR",     "","SEI","ADC", "",    "","",       "ADC", "ROR", "", // 70
	"",     "STA","",     "","STY",     "STA","STX",     "","DEY","",    "TXA", "","STY",    "STA", "STX", "", // 80
	"BCC",  "STA","",     "","STY",     "STA","STX",     "","TYA","STA", "TXS", "","",       "STA", "",    "", // 90
	"LDY",  "LDA","LDX",  "","LDY",     "LDA","LDX",     "","TAY","LDA", "TAX", "","LDY",    "LDA", "LDX", "", // 0a
	"BCS",  "LDA","",     "","LDY",     "LDA","LDX",     "","CLV","LDA", "TSX", "","LDY",    "LDA", "LDX", "", // 0b
	"CPY",  "CMP","",     "","CPY",     "CMP","DEC",     "","INY","CMP", "DEX", "","CPY",    "CMP", "DEC", "", // 0c
	"BNE",  "CMP","",     "","",        "CMP","DEC",     "","CLD","CMP", "",    "","",       "CMP", "DEC", "", // 0d
	"CPX",  "SBC","",     "","CPX",     "SBC","INC",     "","INX","SBC", "NOP", "","CPX",    "SBC", "INC", "", // 0e
	"BEQ",  "SBC","",     "","",        "SBX","INC",     "","SED","SBC", "",    "","",       "SBX", "INC", ""  // 0f
];

// CPU
struct Cpu {
    pc: u16,
    sp: u8,
    ac: u8,
    xr: u8,
    yr: u8,
    st: u8,
}

// MEMORY
struct Memory {
    mem: Vec<u8>,
}

// convert two bytes (hi and lo) to a word
fn byte_to_word(lobyte: u8, hibyte: u8) -> u16 {
    ((hibyte as u16) << 8) | lobyte as u16
}

// initialize memory with zero's
fn init_memory(mem: &mut Memory) {
    for i in 0..MEMSIZE {
        mem.mem[i] = 0x00;
    }
}

// reset cpu
// set stack pointer to 0xff
// set program counter to reset vector
// set unused bit on status flag (assuming starts at zero)
fn reset_cpu(cpu: &mut Cpu, mem: &Memory) {
    cpu.sp = 0xff;
    cpu.pc = byte_to_word(mem.mem[RESET_VECTOR_LOBYTE], mem.mem[RESET_VECTOR_HIBYTE]);
    cpu.st = cpu.st | STATUS_FLAGS_UNUSED;
}

// pushes a u8 to the stack
fn push_to_stack(b:u8, cpu: &mut Cpu, mem: &mut Memory)
{
    let stack_base:usize = 0x0100;
    let memloc:usize = stack_base + cpu.sp as usize;
	mem.mem[memloc] = b;
    cpu.sp -= 1;
}

// pulls a u8 from the stack
fn pull_from_stack(cpu: &mut Cpu, mem: &Memory) -> u8
{
    cpu.sp += 1;
    let stack_base:usize = 0x0100;
    let memloc:usize = stack_base + cpu.sp as usize;
    mem.mem[memloc]
}

// prototype for cpu operation (opcode)
type CpuOp = fn(cpu: &mut Cpu, mem: &mut Memory);

// for unused op codes just do nothing
fn ixx(_cpu: &mut Cpu, _mem: &mut Memory) {
    // place holder for op codes not implemented
}

// BRK (00)
fn i00(cpu: &mut Cpu, mem: &mut Memory) {
    cpu.st |= STATUS_FLAGS_BREAK|STATUS_FLAGS_UNUSED;
    cpu.pc += 2;
    push_to_stack((cpu.pc >> 8) as u8, cpu, mem);
    push_to_stack((cpu.pc & 0xff) as u8, cpu, mem);
    push_to_stack(cpu.st, cpu, mem);
    cpu.st |= STATUS_BIT_INT_DIS;
    cpu.pc = mem.mem[BREAK_VECTOR_LOBYTE] as u16 + ((mem.mem[BREAK_VECTOR_HIBYTE] as u16) << 8);
	//cpu->pending_cycles += 7;
}

// NOP (EA)
fn iea(cpu: &mut Cpu, _mem: &mut Memory) {
    cpu.pc += 1;
    //cpu->pending_cycles += 2;
}

// op code array
const CPU_OPS: [CpuOp; 256] = [
    //0    1    2    3    4    5    6    7    8    9    a    b    c    d    e    f
    i00, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx,     // 00
    ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx,     // 10
    ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx,     // 20
    ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx,     // 30
    ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx,     // 40
    ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx,     // 50
    ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx,     // 60
    ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx,     // 70
    ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx,     // 80
    ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx,     // 90
    ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx,     // a0
    ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx,     // b0
    ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx,     // c0
    ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx,     // d0
    ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, iea, ixx, ixx, ixx, ixx, ixx,     // e0
    ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx, ixx,     // f0
];

fn main() {
    let mut cpu = Cpu {
        pc: 0,
        sp: 0,
        ac: 0,
        xr: 0,
        yr: 0,
        st: 0,
    };
    let mut mem: Memory = Memory {
        mem: vec![0; MEMSIZE],
    };
    let pause_on_exec_instr: u8 = 1;
    let print_output: u8 = 1;

    // initialize memory
    init_memory(&mut mem);

    // for debugging; start at 0x400
    mem.mem[0xfffc] = 0x00;
    mem.mem[0xfffd] = 0x04;
    mem.mem[0x0400] = 0xea;
    
    // initialize cpu
    reset_cpu(&mut cpu, &mem);

    let stdin = io::stdin();

    // main loop
    loop {
        // get keys for 0xC000 (keyboard)

        if print_output == 1 {
            // TODO implement g_instruction_text
            let memloc:usize = cpu.pc as usize;
            let instrloc:usize = mem.mem[memloc] as usize;
            print!("\t${:04x}\t{}", cpu.pc, INSTRUCTION_TEXT[instrloc]);
        }

        // execute the opcode
        let opcode = mem.mem[cpu.pc as usize];
        let opcode_handler = CPU_OPS[opcode as usize];
        opcode_handler(&mut cpu, &mut mem);

        if print_output == 1 {
            println!();
        }

        if pause_on_exec_instr == 1 {
            // get user input
            let mut user_input = String::new();
            let _result = stdin.read_line(&mut user_input);
        }

    }
}
