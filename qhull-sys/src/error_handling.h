#pragma once

#include <setjmp.h>
#include "../qhull/src/libqhull_r/libqhull_r.h"

#define QH_TRY_ERROR 10071

int qhull_sys__try_on_qh(
    qhT* qh,
    void (*fn)(void* data),
    void* data
);

FILE* qhull_sys__stdin();
FILE* qhull_sys__stdout();
FILE* qhull_sys__stderr();