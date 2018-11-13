const ISA_TESTS: &[(&str, &str)] = &[
    #[cfg(feature = "rv32c")]
    ("rv32uc", "rvc"),

    ("rv32ui", "add"),
    ("rv32ui", "addi"),
    ("rv32ui", "and"),
    ("rv32ui", "andi"),
    ("rv32ui", "auipc"),
    ("rv32ui", "beq"),
    ("rv32ui", "bge"),
    ("rv32ui", "bgeu"),
    ("rv32ui", "blt"),
    ("rv32ui", "bltu"),
    ("rv32ui", "bne"),
    ("rv32ui", "fence_i"),
    ("rv32ui", "jal"),
    ("rv32ui", "jalr"),
    ("rv32ui", "lb"),
    ("rv32ui", "lbu"),
    ("rv32ui", "lh"),
    ("rv32ui", "lhu"),
    ("rv32ui", "lui"),
    ("rv32ui", "lw"),
    ("rv32ui", "or"),
    ("rv32ui", "ori"),
    ("rv32ui", "sb"),
    ("rv32ui", "sh"),
    ("rv32ui", "simple"),
    ("rv32ui", "sll"),
    ("rv32ui", "slli"),
    ("rv32ui", "slt"),
    ("rv32ui", "slti"),
    ("rv32ui", "sltiu"),
    ("rv32ui", "sltu"),
    ("rv32ui", "sra"),
    ("rv32ui", "srai"),
    ("rv32ui", "srl"),
    ("rv32ui", "srli"),
    ("rv32ui", "sub"),
    ("rv32ui", "sw"),
    ("rv32ui", "xor"),
    ("rv32ui", "xori"),

    ("rv32um", "div"),
    ("rv32um", "divu"),
    ("rv32um", "mul"),
    ("rv32um", "mulh"),
    ("rv32um", "mulhsu"),
    ("rv32um", "mulhu"),
    ("rv32um", "rem"),
    ("rv32um", "remu"),

    ("rv32ua", "amoadd_w"),
    ("rv32ua", "amoand_w"),
    ("rv32ua", "amomax_w"),
    ("rv32ua", "amomaxu_w"),
    ("rv32ua", "amomin_w"),
    ("rv32ua", "amominu_w"),
    ("rv32ua", "amoor_w"),
    ("rv32ua", "amoswap_w"),
    ("rv32ua", "amoxor_w"),
    ("rv32ua", "lrsc"),

    #[cfg(feature = "rv32fd")]
    ("rv32uf", "fadd"),
    #[cfg(feature = "rv32fd")]
    ("rv32uf", "fclass"),
    #[cfg(feature = "rv32fd")]
    ("rv32uf", "fcmp"),
    #[cfg(feature = "rv32fd")]
    ("rv32uf", "fcvt"),
    #[cfg(feature = "rv32fd")]
    ("rv32uf", "fcvt_w"),
    #[cfg(feature = "rv32fd")]
    ("rv32uf", "fdiv"),
    #[cfg(feature = "rv32fd")]
    ("rv32uf", "fmadd"),
    #[cfg(feature = "rv32fd")]
    ("rv32uf", "fmin"),
    #[cfg(feature = "rv32fd")]
    ("rv32uf", "ldst"),
    #[cfg(feature = "rv32fd")]
    ("rv32uf", "move"),
    #[cfg(feature = "rv32fd")]
    ("rv32uf", "recoding"),

    #[cfg(feature = "rv32fd")]
    ("rv32ud", "fadd"),
    #[cfg(feature = "rv32fd")]
    ("rv32ud", "fclass"),
    #[cfg(feature = "rv32fd")]
    ("rv32ud", "fcmp"),
    #[cfg(feature = "rv32fd")]
    ("rv32ud", "fcvt"),
    #[cfg(feature = "rv32fd")]
    ("rv32ud", "fcvt_w"),
    #[cfg(feature = "rv32fd")]
    ("rv32ud", "fdiv"),
    #[cfg(feature = "rv32fd")]
    ("rv32ud", "fmadd"),
    #[cfg(feature = "rv32fd")]
    ("rv32ud", "fmin"),
    #[cfg(feature = "rv32fd")]
    ("rv32ud", "ldst"),
    // ("rv32ud", "move"),
    #[cfg(feature = "rv32fd")]
    ("rv32ud", "recoding"),
];
