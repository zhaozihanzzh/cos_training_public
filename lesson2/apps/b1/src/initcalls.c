/* Helper for getting init_calls_[start/end] */

extern unsigned long init_calls_start;
unsigned long *initcalls_start() {
    return &init_calls_start;
}

extern unsigned long init_calls_end;
unsigned long *initcalls_end() {
    return &init_calls_end;
}
