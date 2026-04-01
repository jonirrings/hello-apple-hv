# hello-hv

Apple Hypervisor Framework ARM64 测试程序

## 编译

```shell
cargo build --release
```

## 签名

运行前需要对二进制文件签名以获取 Hypervisor 权限：

```shell
codesign --sign - --entitlements hv.entitlements.xml --deep --force target/release/hello-hv
```

## 运行

```shell
./target/release/hello-hv
```

## 测试结果

```
========================================
Apple Hypervisor Framework ARM64 Tests
========================================

[✓ PASS] VM/VCPU Creation
       VM and VCPU created successfully with debug features enabled

[✓ PASS] Register Operations
       All general purpose registers and PC read/write verified

[✓ PASS] Memory Mapping
       Memory mapping with RWX/RW permissions verified

[✓ PASS] MOV Immediate
       MOV X0, #0x42 executed correctly, X0 = 0x42

[✓ PASS] ADD Instruction
       ADD X0, X0, X1 executed correctly, X0 = 42

[✓ PASS] SUB Instruction
       SUB X0, X0, X1 executed correctly, X0 = 42

[✓ PASS] Load/Store
       STR/LDR executed correctly, X0 = 0xDEADBEEF

[✓ PASS] AND Instruction
       AND X0, X0, X1 executed correctly, X0 = 0x0F

[✓ PASS] ORR Instruction
       ORR X0, X0, X1 executed correctly, X0 = 0xFF

[✓ PASS] EOR Instruction
       EOR X0, X0, X1 executed correctly, X0 = 0xF0

[✓ PASS] MOV Shift
       LSL X0, X1, #4 executed correctly, X0 = 16

[✓ PASS] Instruction Sequence
       Multi-instruction ADD sequence executed correctly, X0 = 50

[✓ PASS] Exit Info
       Exit information retrieved: reason=EXCEPTION, exception=hv_vcpu_exit_exception_t { syndrome: 2181038087, virtual_address: 1024, physical_address: 1024 }

========================================
Results: 13 passed, 0 failed, 13 total
========================================
```

## 测试覆盖

| 测试名称 | 说明 |
|---------|------|
| VM/VCPU Creation | 验证虚拟机和虚拟 CPU 创建，以及调试功能启用 |
| Register Operations | 测试所有通用寄存器 X0-X28 和 PC 的读写 |
| Memory Mapping | 测试内存映射创建、权限设置 (RWX/RW) 和读写 |
| MOV Immediate | 验证 MOV 立即数指令执行 |
| ADD Instruction | 验证加法指令 |
| SUB Instruction | 验证减法指令 |
| Load/Store | 验证 LDR/STR 内存访问指令 |
| AND Instruction | 验证逻辑与指令 |
| ORR Instruction | 验证逻辑或指令 |
| EOR Instruction | 验证逻辑异或指令 |
| MOV Shift | 验证逻辑左移操作 (LSL) |
| Instruction Sequence | 验证多指令序列执行 |
| Exit Info | 验证 VCPU 退出信息获取 |

## 调试

```shell
lldb target/debug/hello-hv
```

### 使用 RustRover 调试

1. Menu-Run-Attach to an Unstarted Process
2. Select target/debug/hello-hv
3. run the executable in another terminal