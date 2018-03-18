
            /* tslint:disable */
            import * as wasm from './simple_vm_wasm_bg'; // imports from wasm file
            

            
            let cachedDecoder = null;
            function textDecoder() {
                if (cachedDecoder)
                    return cachedDecoder;
                cachedDecoder = new TextDecoder('utf-8');
                return cachedDecoder;
            }
        
            let cachedUint8Memory = null;
            function getUint8Memory() {
                if (cachedUint8Memory === null ||
                    cachedUint8Memory.buffer !== wasm.memory.buffer)
                    cachedUint8Memory = new Uint8Array(wasm.memory.buffer);
                return cachedUint8Memory;
            }
        
                function getStringFromWasmBrowser(ptr, len) {
                    const mem = getUint8Memory();
                    const slice = mem.slice(ptr, ptr + len);
                    const ret = textDecoder().decode(slice);
                    return ret;
                }
            const getStringFromWasm = getStringFromWasmBrowser;

                    const __wbg_s_console_log_target = console.log;
                export function __wbg_s_console_log(ptr0, len0) {

                        let arg0 = getStringFromWasm(ptr0, len0);
                    __wbg_s_console_log_target(arg0)
}

                    const __wbg_s_console_warn_target = console.warn;
                export function __wbg_s_console_warn(ptr0, len0) {

                        let arg0 = getStringFromWasm(ptr0, len0);
                    __wbg_s_console_warn_target(arg0)
}

            let cachedEncoder = null;
            function textEncoder() {
                if (cachedEncoder)
                    return cachedEncoder;
                cachedEncoder = new TextEncoder('utf-8');
                return cachedEncoder;
            }
        
                function passStringToWasmBrowser(arg) {
                    if (typeof(arg) !== 'string')
                        throw new Error('expected a string argument');
                    const buf = textEncoder().encode(arg);
                    const len = buf.length;
                    const ptr = wasm.__wbindgen_malloc(len);
                    getUint8Memory().set(buf, ptr);
                    return [ptr, len];
                }
            const passStringToWasm = passStringToWasmBrowser;

            let cachedUint32Memory = null;
            function getUint32Memory() {
                if (cachedUint32Memory === null ||
                    cachedUint32Memory.buffer !== wasm.memory.buffer)
                    cachedUint32Memory = new Uint32Array(wasm.memory.buffer);
                return cachedUint32Memory;
            }
        
            function getArrayU32FromWasm(ptr, len) {
                const mem = getUint32Memory();
                const slice = mem.slice(ptr / 4, ptr / 4 + len);
                return new Uint32Array(slice);
            }
        
            let stack = [];
        let slab = [];
            function getObject(idx) {
                if ((idx & 1) === 1) {
                    return stack[idx >> 1];
                } else {
                    const val = slab[idx >> 1];
                    
                return val.obj;
            
                }
            }
        
                function getArrayJsValueFromWasm(ptr, len) {
                    const mem = getUint32Memory();
                    const slice = mem.slice(ptr / 4, ptr / 4 + len);
                    const result = []
                    for (ptr in slice) {
                        result.push(getObject(ptr))
                    }
                    return result;
                }
            export function run(arg0, arg1) {
        const [ptr0, len0] = passStringToWasm(arg0);
                    const [ptr1, len1] = passStringToWasm(arg1);
                    try {
                    const ret = wasm.run(ptr0, len0, ptr1, len1);
                    
                    const ptr = wasm.__wbindgen_boxed_str_ptr(ret);
                    const len = wasm.__wbindgen_boxed_str_len(ret);
                    const realRet = getArrayJsValueFromWasm(ptr, len);
                    wasm.__wbindgen_boxed_str_free(ret);
                    return realRet;
                
                } finally {
                    
wasm.__wbindgen_free(ptr0, len0);

wasm.__wbindgen_free(ptr1, len1);

                }
            }

            let slab_next = 0;
        
            function addHeapObject(obj) {
                if (slab_next == slab.length)
                    slab.push(slab.length + 1);
                const idx = slab_next;
                const next = slab[idx];
                
                slab_next = next;
            
                slab[idx] = { obj, cnt: 1 };
                return idx << 1;
            }
        export function __wbindgen_object_clone_ref (idx) {
                        // If this object is on the stack promote it to the heap.
                        if ((idx & 1) === 1)
                            return addHeapObject(getObject(idx));

                        // Otherwise if the object is on the heap just bump the
                        // refcount and move on
                        const val = slab[idx >> 1];
                        val.cnt += 1;
                        return idx;
                    }

            function dropRef(idx) {
                

                let obj = slab[idx >> 1];
                
                obj.cnt -= 1;
                if (obj.cnt > 0)
                    return;
            

                // If we hit 0 then free up our space in the slab
                slab[idx >> 1] = slab_next;
                slab_next = idx >> 1;
            }
        export function __wbindgen_object_drop_ref (i) { dropRef(i); }
export function __wbindgen_string_new (p, l) {
                    return addHeapObject(getStringFromWasm(p, l));
                }
export function __wbindgen_number_new (i) { return addHeapObject(i); }
export function __wbindgen_number_get (n, invalid) {
                        let obj = getObject(n);
                        if (typeof(obj) === 'number')
                            return obj;
                        getUint8Memory()[invalid] = 1;
                        return 0;
                    }
export function __wbindgen_undefined_new () { return addHeapObject(undefined); }
export function __wbindgen_null_new () {
                    return addHeapObject(null);
                }
export function __wbindgen_is_null (idx) {
                    return getObject(idx) === null ? 1 : 0;
                }
export function __wbindgen_is_undefined (idx) {
                    return getObject(idx) === undefined ? 1 : 0;
                }
export function __wbindgen_boolean_new (v) {
                    return addHeapObject(v == 1);
                }
export function __wbindgen_boolean_get (i) {
                    let v = getObject(i);
                    if (typeof(v) == 'boolean') {
                        return v ? 1 : 0;
                    } else {
                        return 2;
                    }
                }
export function __wbindgen_symbol_new (ptr, len) {
                    let a;
                    console.log(ptr, len);
                    if (ptr === 0) {
                        a = Symbol();
                    } else {
                        a = Symbol(getStringFromWasm(ptr, len));
                    }
                    return addHeapObject(a);
                }
export function __wbindgen_is_symbol (i) {
                    return typeof(getObject(i)) == 'symbol' ? 1 : 0;
                }
export function __wbindgen_throw (ptr, len) {
                        throw new Error(getStringFromWasm(ptr, len));
                    }
export function __wbindgen_string_get (i, len_ptr) {
                    let obj = getObject(i);
                    if (typeof(obj) !== 'string')
                        return 0;
                    const [ptr, len] = passStringToWasm(obj);
                    getUint32Memory()[len_ptr / 4] = len;
                    return ptr;
                }

        