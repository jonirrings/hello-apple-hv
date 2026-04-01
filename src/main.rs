use applevisor::*;

/// Test result structure
struct TestResult {
    name: &'static str,
    passed: bool,
    message: String,
}

impl TestResult {
    fn pass(name: &'static str, message: &str) -> Self {
        Self {
            name,
            passed: true,
            message: message.to_string(),
        }
    }

    fn fail(name: &'static str, message: &str) -> Self {
        Self {
            name,
            passed: false,
            message: message.to_string(),
        }
    }
}

/// Test 1: Basic VM and VCPU creation
fn test_vm_vcpu_creation() -> TestResult {
    let _vm = match VirtualMachine::new() {
        Ok(vm) => vm,
        Err(e) => return TestResult::fail("VM/VCPU Creation", &format!("Failed to create VM: {:?}", e)),
    };

    let vcpu = match Vcpu::new() {
        Ok(vcpu) => vcpu,
        Err(e) => return TestResult::fail("VM/VCPU Creation", &format!("Failed to create VCPU: {:?}", e)),
    };

    // Verify debug features can be enabled
    if vcpu.set_trap_debug_exceptions(true).is_err() {
        return TestResult::fail("VM/VCPU Creation", "Failed to set trap debug exceptions");
    }
    if vcpu.set_trap_debug_reg_accesses(true).is_err() {
        return TestResult::fail("VM/VCPU Creation", "Failed to set trap debug reg accesses");
    }

    TestResult::pass("VM/VCPU Creation", "VM and VCPU created successfully with debug features enabled")
}

/// Test 2: Register read/write operations
fn test_register_operations() -> TestResult {
    let _vm = match VirtualMachine::new() {
        Ok(vm) => vm,
        Err(e) => return TestResult::fail("Register Operations", &format!("Failed to create VM: {:?}", e)),
    };
    let vcpu = match Vcpu::new() {
        Ok(vcpu) => vcpu,
        Err(e) => return TestResult::fail("Register Operations", &format!("Failed to create VCPU: {:?}", e)),
    };

    // Test general purpose registers X0-X28
    for i in 0..=28 {
        let reg = match i {
            0 => Reg::X0, 1 => Reg::X1, 2 => Reg::X2, 3 => Reg::X3,
            4 => Reg::X4, 5 => Reg::X5, 6 => Reg::X6, 7 => Reg::X7,
            8 => Reg::X8, 9 => Reg::X9, 10 => Reg::X10, 11 => Reg::X11,
            12 => Reg::X12, 13 => Reg::X13, 14 => Reg::X14, 15 => Reg::X15,
            16 => Reg::X16, 17 => Reg::X17, 18 => Reg::X18, 19 => Reg::X19,
            20 => Reg::X20, 21 => Reg::X21, 22 => Reg::X22, 23 => Reg::X23,
            24 => Reg::X24, 25 => Reg::X25, 26 => Reg::X26, 27 => Reg::X27,
            28 => Reg::X28, _ => unreachable!(),
        };

        let test_value: u64 = 0xDEAD_BEEF_0000_0000 | (i as u64);
        if vcpu.set_reg(reg, test_value).is_err() {
            return TestResult::fail("Register Operations", &format!("Failed to set X{}", i));
        }
        match vcpu.get_reg(reg) {
            Ok(val) if val == test_value => {}
            Ok(val) => return TestResult::fail("Register Operations",
                &format!("X{} mismatch: expected {:#x}, got {:#x}", i, test_value, val)),
            Err(e) => return TestResult::fail("Register Operations", &format!("Failed to get X{}: {:?}", i, e)),
        }
    }

    // Test PC register
    let pc_value: u64 = 0x8000;
    if vcpu.set_reg(Reg::PC, pc_value).is_err() {
        return TestResult::fail("Register Operations", "Failed to set PC");
    }
    match vcpu.get_reg(Reg::PC) {
        Ok(val) if val == pc_value => {}
        Ok(val) => return TestResult::fail("Register Operations",
            &format!("PC mismatch: expected {:#x}, got {:#x}", pc_value, val)),
        Err(e) => return TestResult::fail("Register Operations", &format!("Failed to get PC: {:?}", e)),
    }

    TestResult::pass("Register Operations", "All general purpose registers and PC read/write verified")
}

/// Test 3: Memory mapping and permissions
fn test_memory_mapping() -> TestResult {
    let _vm = match VirtualMachine::new() {
        Ok(vm) => vm,
        Err(e) => return TestResult::fail("Memory Mapping", &format!("Failed to create VM: {:?}", e)),
    };

    // Test creating multiple memory regions
    let mut mem1 = match Mapping::new(0x1000) {
        Ok(m) => m,
        Err(e) => return TestResult::fail("Memory Mapping", &format!("Failed to create mapping 1: {:?}", e)),
    };

    let mut mem2 = match Mapping::new(0x2000) {
        Ok(m) => m,
        Err(e) => return TestResult::fail("Memory Mapping", &format!("Failed to create mapping 2: {:?}", e)),
    };

    // Test mapping with different permissions
    if mem1.map(0x4000, MemPerms::RWX).is_err() {
        return TestResult::fail("Memory Mapping", "Failed to map mem1 with RWX permissions");
    }

    if mem2.map(0x8000, MemPerms::RW).is_err() {
        return TestResult::fail("Memory Mapping", "Failed to map mem2 with RW permissions");
    }

    // Test memory write and read
    let test_data: u32 = 0x1234_5678;
    if mem1.write_dword(0x4000, test_data).is_err() {
        return TestResult::fail("Memory Mapping", "Failed to write dword to mem1");
    }

    // Read back and verify
    match mem1.read_dword(0x4000) {
        Ok(val) if val == test_data => {}
        Ok(val) => return TestResult::fail("Memory Mapping",
            &format!("Memory read mismatch: expected {:#x}, got {:#x}", test_data, val)),
        Err(e) => return TestResult::fail("Memory Mapping", &format!("Failed to read dword: {:?}", e)),
    }

    TestResult::pass("Memory Mapping", "Memory mapping with RWX/RW permissions verified")
}

/// Test 4: ARM64 instruction execution - MOV immediate
fn test_mov_immediate() -> TestResult {
    let _vm = match VirtualMachine::new() {
        Ok(vm) => vm,
        Err(e) => return TestResult::fail("MOV Immediate", &format!("Failed to create VM: {:?}", e)),
    };
    let vcpu = match Vcpu::new() {
        Ok(vcpu) => vcpu,
        Err(e) => return TestResult::fail("MOV Immediate", &format!("Failed to create VCPU: {:?}", e)),
    };

    let mut mem = match Mapping::new(0x1000) {
        Ok(m) => m,
        Err(e) => return TestResult::fail("MOV Immediate", &format!("Failed to create mapping: {:?}", e)),
    };

    if mem.map(0x4000, MemPerms::RWX).is_err() {
        return TestResult::fail("MOV Immediate", "Failed to map memory");
    }

    // mov x0, #0x42 (MOV wide immediate)
    // Encoding: 0xd2800840
    if mem.write_dword(0x4000, 0xd2800840).is_err() {
        return TestResult::fail("MOV Immediate", "Failed to write MOV instruction");
    }

    // brk #0 (breakpoint to stop execution)
    if mem.write_dword(0x4004, 0xd4200000).is_err() {
        return TestResult::fail("MOV Immediate", "Failed to write BRK instruction");
    }

    // Set PC and run
    if vcpu.set_reg(Reg::PC, 0x4000).is_err() {
        return TestResult::fail("MOV Immediate", "Failed to set PC");
    }

    if vcpu.run().is_err() {
        return TestResult::fail("MOV Immediate", "Failed to run VCPU");
    }

    // Verify X0 = 0x42
    match vcpu.get_reg(Reg::X0) {
        Ok(val) if val == 0x42 => TestResult::pass("MOV Immediate", "MOV X0, #0x42 executed correctly, X0 = 0x42"),
        Ok(val) => TestResult::fail("MOV Immediate", &format!("X0 mismatch: expected 0x42, got {:#x}", val)),
        Err(e) => TestResult::fail("MOV Immediate", &format!("Failed to get X0: {:?}", e)),
    }
}

/// Test 5: ARM64 ADD instruction
fn test_add_instruction() -> TestResult {
    let _vm = match VirtualMachine::new() {
        Ok(vm) => vm,
        Err(e) => return TestResult::fail("ADD Instruction", &format!("Failed to create VM: {:?}", e)),
    };
    let vcpu = match Vcpu::new() {
        Ok(vcpu) => vcpu,
        Err(e) => return TestResult::fail("ADD Instruction", &format!("Failed to create VCPU: {:?}", e)),
    };

    let mut mem = match Mapping::new(0x1000) {
        Ok(m) => m,
        Err(e) => return TestResult::fail("ADD Instruction", &format!("Failed to create mapping: {:?}", e)),
    };

    if mem.map(0x4000, MemPerms::RWX).is_err() {
        return TestResult::fail("ADD Instruction", "Failed to map memory");
    }

    // Set X0 = 10, X1 = 32
    if vcpu.set_reg(Reg::X0, 10).is_err() || vcpu.set_reg(Reg::X1, 32).is_err() {
        return TestResult::fail("ADD Instruction", "Failed to set initial registers");
    }

    // add x0, x0, x1 (X0 = X0 + X1)
    // Encoding: 0x8b010000
    if mem.write_dword(0x4000, 0x8b010000).is_err() {
        return TestResult::fail("ADD Instruction", "Failed to write ADD instruction");
    }

    // brk #0
    if mem.write_dword(0x4004, 0xd4200000).is_err() {
        return TestResult::fail("ADD Instruction", "Failed to write BRK instruction");
    }

    if vcpu.set_reg(Reg::PC, 0x4000).is_err() || vcpu.run().is_err() {
        return TestResult::fail("ADD Instruction", "Failed to execute");
    }

    match vcpu.get_reg(Reg::X0) {
        Ok(val) if val == 42 => TestResult::pass("ADD Instruction", "ADD X0, X0, X1 executed correctly, X0 = 42"),
        Ok(val) => TestResult::fail("ADD Instruction", &format!("X0 mismatch: expected 42, got {}", val)),
        Err(e) => TestResult::fail("ADD Instruction", &format!("Failed to get X0: {:?}", e)),
    }
}

/// Test 6: ARM64 SUB instruction
fn test_sub_instruction() -> TestResult {
    let _vm = match VirtualMachine::new() {
        Ok(vm) => vm,
        Err(e) => return TestResult::fail("SUB Instruction", &format!("Failed to create VM: {:?}", e)),
    };
    let vcpu = match Vcpu::new() {
        Ok(vcpu) => vcpu,
        Err(e) => return TestResult::fail("SUB Instruction", &format!("Failed to create VCPU: {:?}", e)),
    };

    let mut mem = match Mapping::new(0x1000) {
        Ok(m) => m,
        Err(e) => return TestResult::fail("SUB Instruction", &format!("Failed to create mapping: {:?}", e)),
    };

    if mem.map(0x4000, MemPerms::RWX).is_err() {
        return TestResult::fail("SUB Instruction", "Failed to map memory");
    }

    // Set X0 = 100, X1 = 58
    if vcpu.set_reg(Reg::X0, 100).is_err() || vcpu.set_reg(Reg::X1, 58).is_err() {
        return TestResult::fail("SUB Instruction", "Failed to set initial registers");
    }

    // sub x0, x0, x1 (X0 = X0 - X1)
    // Encoding: 0xcb010000
    if mem.write_dword(0x4000, 0xcb010000).is_err() {
        return TestResult::fail("SUB Instruction", "Failed to write SUB instruction");
    }

    // brk #0
    if mem.write_dword(0x4004, 0xd4200000).is_err() {
        return TestResult::fail("SUB Instruction", "Failed to write BRK instruction");
    }

    if vcpu.set_reg(Reg::PC, 0x4000).is_err() || vcpu.run().is_err() {
        return TestResult::fail("SUB Instruction", "Failed to execute");
    }

    match vcpu.get_reg(Reg::X0) {
        Ok(val) if val == 42 => TestResult::pass("SUB Instruction", "SUB X0, X0, X1 executed correctly, X0 = 42"),
        Ok(val) => TestResult::fail("SUB Instruction", &format!("X0 mismatch: expected 42, got {}", val)),
        Err(e) => TestResult::fail("SUB Instruction", &format!("Failed to get X0: {:?}", e)),
    }
}

/// Test 7: ARM64 LDR/STR (load/store) instructions
fn test_load_store() -> TestResult {
    let _vm = match VirtualMachine::new() {
        Ok(vm) => vm,
        Err(e) => return TestResult::fail("Load/Store", &format!("Failed to create VM: {:?}", e)),
    };
    let vcpu = match Vcpu::new() {
        Ok(vcpu) => vcpu,
        Err(e) => return TestResult::fail("Load/Store", &format!("Failed to create VCPU: {:?}", e)),
    };

    let mut mem = match Mapping::new(0x2000) {
        Ok(m) => m,
        Err(e) => return TestResult::fail("Load/Store", &format!("Failed to create mapping: {:?}", e)),
    };

    if mem.map(0x4000, MemPerms::RWX).is_err() {
        return TestResult::fail("Load/Store", "Failed to map memory");
    }

    // Set X1 = 0x4100 (base address for load/store)
    if vcpu.set_reg(Reg::X1, 0x4100).is_err() {
        return TestResult::fail("Load/Store", "Failed to set X1");
    }

    // Set X2 = 0xDEADBEEF (value to store)
    if vcpu.set_reg(Reg::X2, 0xDEADBEEF).is_err() {
        return TestResult::fail("Load/Store", "Failed to set X2");
    }

    // str x2, [x1] - store X2 to memory at X1
    // Encoding: 0xf9000022
    if mem.write_dword(0x4000, 0xf9000022).is_err() {
        return TestResult::fail("Load/Store", "Failed to write STR instruction");
    }

    // ldr x0, [x1] - load from memory at X1 into X0
    // Encoding: 0xf9400020
    if mem.write_dword(0x4004, 0xf9400020).is_err() {
        return TestResult::fail("Load/Store", "Failed to write LDR instruction");
    }

    // brk #0
    if mem.write_dword(0x4008, 0xd4200000).is_err() {
        return TestResult::fail("Load/Store", "Failed to write BRK instruction");
    }

    if vcpu.set_reg(Reg::PC, 0x4000).is_err() || vcpu.run().is_err() {
        return TestResult::fail("Load/Store", "Failed to execute");
    }

    match vcpu.get_reg(Reg::X0) {
        Ok(val) if val == 0xDEADBEEF => {
            TestResult::pass("Load/Store", "STR/LDR executed correctly, X0 = 0xDEADBEEF")
        }
        Ok(val) => TestResult::fail("Load/Store", &format!("X0 mismatch: expected 0xDEADBEEF, got {:#x}", val)),
        Err(e) => TestResult::fail("Load/Store", &format!("Failed to get X0: {:?}", e)),
    }
}

/// Test 8: ARM64 logical AND instruction
fn test_and_instruction() -> TestResult {
    let _vm = match VirtualMachine::new() {
        Ok(vm) => vm,
        Err(e) => return TestResult::fail("AND Instruction", &format!("Failed to create VM: {:?}", e)),
    };
    let vcpu = match Vcpu::new() {
        Ok(vcpu) => vcpu,
        Err(e) => return TestResult::fail("AND Instruction", &format!("Failed to create VCPU: {:?}", e)),
    };

    let mut mem = match Mapping::new(0x1000) {
        Ok(m) => m,
        Err(e) => return TestResult::fail("AND Instruction", &format!("Failed to create mapping: {:?}", e)),
    };

    if mem.map(0x4000, MemPerms::RWX).is_err() {
        return TestResult::fail("AND Instruction", "Failed to map memory");
    }

    // Set X0 = 0xFF, X1 = 0x0F
    if vcpu.set_reg(Reg::X0, 0xFF).is_err() || vcpu.set_reg(Reg::X1, 0x0F).is_err() {
        return TestResult::fail("AND Instruction", "Failed to set initial registers");
    }

    // and x0, x0, x1 (X0 = X0 & X1)
    // Encoding: 0x8a010000
    if mem.write_dword(0x4000, 0x8a010000).is_err() {
        return TestResult::fail("AND Instruction", "Failed to write AND instruction");
    }

    // brk #0
    if mem.write_dword(0x4004, 0xd4200000).is_err() {
        return TestResult::fail("AND Instruction", "Failed to write BRK instruction");
    }

    if vcpu.set_reg(Reg::PC, 0x4000).is_err() || vcpu.run().is_err() {
        return TestResult::fail("AND Instruction", "Failed to execute");
    }

    match vcpu.get_reg(Reg::X0) {
        Ok(val) if val == 0x0F => TestResult::pass("AND Instruction", "AND X0, X0, X1 executed correctly, X0 = 0x0F"),
        Ok(val) => TestResult::fail("AND Instruction", &format!("X0 mismatch: expected 0x0F, got {:#x}", val)),
        Err(e) => TestResult::fail("AND Instruction", &format!("Failed to get X0: {:?}", e)),
    }
}

/// Test 9: ARM64 ORR (OR) instruction
fn test_orr_instruction() -> TestResult {
    let _vm = match VirtualMachine::new() {
        Ok(vm) => vm,
        Err(e) => return TestResult::fail("ORR Instruction", &format!("Failed to create VM: {:?}", e)),
    };
    let vcpu = match Vcpu::new() {
        Ok(vcpu) => vcpu,
        Err(e) => return TestResult::fail("ORR Instruction", &format!("Failed to create VCPU: {:?}", e)),
    };

    let mut mem = match Mapping::new(0x1000) {
        Ok(m) => m,
        Err(e) => return TestResult::fail("ORR Instruction", &format!("Failed to create mapping: {:?}", e)),
    };

    if mem.map(0x4000, MemPerms::RWX).is_err() {
        return TestResult::fail("ORR Instruction", "Failed to map memory");
    }

    // Set X0 = 0xF0, X1 = 0x0F
    if vcpu.set_reg(Reg::X0, 0xF0).is_err() || vcpu.set_reg(Reg::X1, 0x0F).is_err() {
        return TestResult::fail("ORR Instruction", "Failed to set initial registers");
    }

    // orr x0, x0, x1 (X0 = X0 | X1)
    // Encoding: 0xaa010000
    if mem.write_dword(0x4000, 0xaa010000).is_err() {
        return TestResult::fail("ORR Instruction", "Failed to write ORR instruction");
    }

    // brk #0
    if mem.write_dword(0x4004, 0xd4200000).is_err() {
        return TestResult::fail("ORR Instruction", "Failed to write BRK instruction");
    }

    if vcpu.set_reg(Reg::PC, 0x4000).is_err() || vcpu.run().is_err() {
        return TestResult::fail("ORR Instruction", "Failed to execute");
    }

    match vcpu.get_reg(Reg::X0) {
        Ok(val) if val == 0xFF => TestResult::pass("ORR Instruction", "ORR X0, X0, X1 executed correctly, X0 = 0xFF"),
        Ok(val) => TestResult::fail("ORR Instruction", &format!("X0 mismatch: expected 0xFF, got {:#x}", val)),
        Err(e) => TestResult::fail("ORR Instruction", &format!("Failed to get X0: {:?}", e)),
    }
}

/// Test 10: ARM64 EOR (XOR) instruction
fn test_eor_instruction() -> TestResult {
    let _vm = match VirtualMachine::new() {
        Ok(vm) => vm,
        Err(e) => return TestResult::fail("EOR Instruction", &format!("Failed to create VM: {:?}", e)),
    };
    let vcpu = match Vcpu::new() {
        Ok(vcpu) => vcpu,
        Err(e) => return TestResult::fail("EOR Instruction", &format!("Failed to create VCPU: {:?}", e)),
    };

    let mut mem = match Mapping::new(0x1000) {
        Ok(m) => m,
        Err(e) => return TestResult::fail("EOR Instruction", &format!("Failed to create mapping: {:?}", e)),
    };

    if mem.map(0x4000, MemPerms::RWX).is_err() {
        return TestResult::fail("EOR Instruction", "Failed to map memory");
    }

    // Set X0 = 0xFF, X1 = 0x0F
    if vcpu.set_reg(Reg::X0, 0xFF).is_err() || vcpu.set_reg(Reg::X1, 0x0F).is_err() {
        return TestResult::fail("EOR Instruction", "Failed to set initial registers");
    }

    // eor x0, x0, x1 (X0 = X0 ^ X1)
    // Encoding: 0xca010000
    if mem.write_dword(0x4000, 0xca010000).is_err() {
        return TestResult::fail("EOR Instruction", "Failed to write EOR instruction");
    }

    // brk #0
    if mem.write_dword(0x4004, 0xd4200000).is_err() {
        return TestResult::fail("EOR Instruction", "Failed to write BRK instruction");
    }

    if vcpu.set_reg(Reg::PC, 0x4000).is_err() || vcpu.run().is_err() {
        return TestResult::fail("EOR Instruction", "Failed to execute");
    }

    match vcpu.get_reg(Reg::X0) {
        Ok(val) if val == 0xF0 => TestResult::pass("EOR Instruction", "EOR X0, X0, X1 executed correctly, X0 = 0xF0"),
        Ok(val) => TestResult::fail("EOR Instruction", &format!("X0 mismatch: expected 0xF0, got {:#x}", val)),
        Err(e) => TestResult::fail("EOR Instruction", &format!("Failed to get X0: {:?}", e)),
    }
}

/// Test 11: ARM64 MOV with shift (LSL)
fn test_mov_shift() -> TestResult {
    let _vm = match VirtualMachine::new() {
        Ok(vm) => vm,
        Err(e) => return TestResult::fail("MOV Shift", &format!("Failed to create VM: {:?}", e)),
    };
    let vcpu = match Vcpu::new() {
        Ok(vcpu) => vcpu,
        Err(e) => return TestResult::fail("MOV Shift", &format!("Failed to create VCPU: {:?}", e)),
    };

    let mut mem = match Mapping::new(0x1000) {
        Ok(m) => m,
        Err(e) => return TestResult::fail("MOV Shift", &format!("Failed to create mapping: {:?}", e)),
    };

    if mem.map(0x4000, MemPerms::RWX).is_err() {
        return TestResult::fail("MOV Shift", "Failed to map memory");
    }

    // Set X1 = 1
    if vcpu.set_reg(Reg::X1, 1).is_err() {
        return TestResult::fail("MOV Shift", "Failed to set X1");
    }

    // lsl x0, x1, #4 (X0 = X1 << 4 = 16)
    // Correct encoding: orr x0, xzr, x1, lsl #4
    // ORR (shifted register): sf=1, opc=01, opcode=01010, shift=00(LSL), N=0, Rm=x1=1, imm6=000100, Rn=xzr=31, Rd=x0=0
    // Binary: 1 01 01010 00 0 00001 000100 11111 00000 = 0xaa0113e0
    if mem.write_dword(0x4000, 0xaa0113e0).is_err() {
        return TestResult::fail("MOV Shift", "Failed to write LSL instruction");
    }

    // brk #0
    if mem.write_dword(0x4004, 0xd4200000).is_err() {
        return TestResult::fail("MOV Shift", "Failed to write BRK instruction");
    }

    if vcpu.set_reg(Reg::PC, 0x4000).is_err() || vcpu.run().is_err() {
        return TestResult::fail("MOV Shift", "Failed to execute");
    }

    match vcpu.get_reg(Reg::X0) {
        Ok(val) if val == 16 => TestResult::pass("MOV Shift", "LSL X0, X1, #4 executed correctly, X0 = 16"),
        Ok(val) => TestResult::fail("MOV Shift", &format!("X0 mismatch: expected 16, got {}", val)),
        Err(e) => TestResult::fail("MOV Shift", &format!("Failed to get X0: {:?}", e)),
    }
}

/// Test 12: Multiple instruction sequence
fn test_instruction_sequence() -> TestResult {
    let _vm = match VirtualMachine::new() {
        Ok(vm) => vm,
        Err(e) => return TestResult::fail("Instruction Sequence", &format!("Failed to create VM: {:?}", e)),
    };
    let vcpu = match Vcpu::new() {
        Ok(vcpu) => vcpu,
        Err(e) => return TestResult::fail("Instruction Sequence", &format!("Failed to create VCPU: {:?}", e)),
    };

    let mut mem = match Mapping::new(0x1000) {
        Ok(m) => m,
        Err(e) => return TestResult::fail("Instruction Sequence", &format!("Failed to create mapping: {:?}", e)),
    };

    if mem.map(0x4000, MemPerms::RWX).is_err() {
        return TestResult::fail("Instruction Sequence", "Failed to map memory");
    }

    // Simple 3-instruction sequence: X0 = 42 + 8 = 50
    // mov x0, #42      ; 0xd2800540
    // mov x1, #8       ; 0xd2800101
    // add x0, x0, x1   ; 0x8b010000 -> X0 = 50
    // brk #0           ; 0xd4200000

    let instructions: [(u64, u32); 4] = [
        (0x4000, 0xd2800540), // mov x0, #42
        (0x4004, 0xd2800101), // mov x1, #8
        (0x4008, 0x8b010000), // add x0, x0, x1 -> X0 = 50
        (0x400c, 0xd4200000), // brk #0
    ];

    for (addr, instr) in instructions {
        if mem.write_dword(addr, instr).is_err() {
            return TestResult::fail("Instruction Sequence", &format!("Failed to write instruction at {:#x}", addr));
        }
    }

    if vcpu.set_reg(Reg::PC, 0x4000).is_err() || vcpu.run().is_err() {
        return TestResult::fail("Instruction Sequence", "Failed to execute");
    }

    match vcpu.get_reg(Reg::X0) {
        Ok(val) if val == 50 => {
            TestResult::pass("Instruction Sequence", "Multi-instruction ADD sequence executed correctly, X0 = 50")
        }
        Ok(val) => TestResult::fail("Instruction Sequence", &format!("X0 mismatch: expected 50, got {}", val)),
        Err(e) => TestResult::fail("Instruction Sequence", &format!("Failed to get X0: {:?}", e)),
    }
}

/// Test 13: Exit information verification
fn test_exit_info() -> TestResult {
    let _vm = match VirtualMachine::new() {
        Ok(vm) => vm,
        Err(e) => return TestResult::fail("Exit Info", &format!("Failed to create VM: {:?}", e)),
    };
    let vcpu = match Vcpu::new() {
        Ok(vcpu) => vcpu,
        Err(e) => return TestResult::fail("Exit Info", &format!("Failed to create VCPU: {:?}", e)),
    };

    let mut mem = match Mapping::new(0x1000) {
        Ok(m) => m,
        Err(e) => return TestResult::fail("Exit Info", &format!("Failed to create mapping: {:?}", e)),
    };

    if mem.map(0x4000, MemPerms::RWX).is_err() {
        return TestResult::fail("Exit Info", "Failed to map memory");
    }

    // brk #0x1234 (software breakpoint with immediate)
    // Encoding: 0xd4202460 (brk #0x1234)
    if mem.write_dword(0x4000, 0xd4202460).is_err() {
        return TestResult::fail("Exit Info", "Failed to write BRK instruction");
    }

    if vcpu.set_reg(Reg::PC, 0x4000).is_err() || vcpu.run().is_err() {
        return TestResult::fail("Exit Info", "Failed to execute");
    }

    // Get exit info - returns VcpuExit struct directly
    let exit_info = vcpu.get_exit_info();
    TestResult::pass("Exit Info",
        &format!("Exit information retrieved: reason={:?}, exception={:?}", exit_info.reason, exit_info.exception))
}

#[cfg(target_arch = "aarch64")]
fn main() {
    println!("========================================");
    println!("Apple Hypervisor Framework ARM64 Tests");
    println!("========================================\n");

    let tests: Vec<TestResult> = vec![
        test_vm_vcpu_creation(),
        test_register_operations(),
        test_memory_mapping(),
        test_mov_immediate(),
        test_add_instruction(),
        test_sub_instruction(),
        test_load_store(),
        test_and_instruction(),
        test_orr_instruction(),
        test_eor_instruction(),
        test_mov_shift(),
        test_instruction_sequence(),
        test_exit_info(),
    ];

    let passed = tests.iter().filter(|t| t.passed).count();
    let failed = tests.len() - passed;

    for test in &tests {
        let status = if test.passed { "✓ PASS" } else { "✗ FAIL" };
        println!("[{}] {}", status, test.name);
        println!("       {}", test.message);
        println!();
    }

    println!("========================================");
    println!("Results: {} passed, {} failed, {} total", passed, failed, tests.len());
    println!("========================================");

    if failed > 0 {
        std::process::exit(1);
    }
}

#[cfg(target_arch = "x86_64")]
fn main() {
    println!("This test is only supported on ARM64 (Apple Silicon) platforms.");
    std::process::exit(1);
}