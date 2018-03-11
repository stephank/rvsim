#ifndef _ENV_CUSTOM_H
#define _ENV_CUSTOM_H

#include "../../vendor/riscv-tests/env/p/riscv_test.h"

#undef RVTEST_CODE_BEGIN
#define RVTEST_CODE_BEGIN                                               \
        .text;                                                          \
        .global _start;                                                 \
_start:                                                                 \
        init

#undef RVTEST_DATA_BEGIN
#define RVTEST_DATA_BEGIN                                               \
        EXTRA_DATA                                                      \
        .align 4; .global begin_signature; begin_signature:

#define RVTEST_DATA_END                                                 \
        .align 4; .global end_signature; end_signature:

#undef RVTEST_ENABLE_SUPERVISOR
#define RVTEST_ENABLE_SUPERVISOR

#undef RVTEST_ENABLE_MACHINE
#define RVTEST_ENABLE_MACHINE

#undef RVTEST_FP_ENABLE
#define RVTEST_FP_ENABLE

#undef RISCV_MULTICORE_DISABLE
#define RISCV_MULTICORE_DISABLE

#endif
