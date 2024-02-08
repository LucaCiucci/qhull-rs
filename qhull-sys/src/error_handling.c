
#include "./error_handling.h"

int qhull_sys__try_on_qh(
    qhT* qh,
    void (*fn)(qhT* qh, void* data),
    void* data
) {
    // See definition of QH_TRY_NO_THROW_ for reference

    if (!qh || !fn) {
        printf("qh or fn is NULL\n");
        return QH_TRY_ERROR;
    }

    // this is the error status, 0 means no error
    int try_status = 0;

    if (qh->NOerrexit) {
        qh->NOerrexit = False;
        try_status = setjmp(qh->errexit);
    } else {
        // try_on_qh was nested
        printf("try_on_qh was nested\n");
        try_status = QH_TRY_ERROR;
    }

    // do not execute the function if an error occurred and we
    // jumped back to the setjmp point
    if (try_status == 0) {
        fn(qh, data);
    }

    qh->NOerrexit = True;

    return try_status;
}

FILE* qhull_sys__stdin() {
    return stdin;
}

FILE* qhull_sys__stdout() {
    return stdout;
}

FILE* qhull_sys__stderr() {
    return stderr;
}