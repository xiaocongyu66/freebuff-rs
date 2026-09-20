import { getMetaContents } from './snippets/dioxus-cli-config-74c4aa43f52789a0/inline0.js';
import { RawInterpreter } from './snippets/dioxus-interpreter-js-20451f6aefec841d/inline0.js';
import { setAttributeInner } from './snippets/dioxus-interpreter-js-20451f6aefec841d/src/js/set_attribute.js';
import { get_select_data } from './snippets/dioxus-web-eae67878b22d6805/inline0.js';
import { js_schedule_toast, js_show_toast } from './snippets/dioxus-web-eae67878b22d6805/inline1.js';
import { WebDioxusChannel } from './snippets/dioxus-web-eae67878b22d6805/src/js/eval.js';
import * as import1 from "./snippets/dioxus-interpreter-js-20451f6aefec841d/src/js/patch_console.js"


//#region exports

export class JSOwner {
    constructor() {
        throw new Error('cannot invoke `new` directly');
    }
    static __wrap(ptr) {
        const obj = Object.create(JSOwner.prototype);
        obj.__wbg_ptr = ptr;
        JSOwnerFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        JSOwnerFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_jsowner_free(ptr, 0);
    }
}
if (Symbol.dispose) JSOwner.prototype[Symbol.dispose] = JSOwner.prototype.free;

//#endregion

//#region wasm imports
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg_Error_67e7344beaa85059: function() { return logError(function (arg0, arg1) {
            const ret = Error(getStringFromWasm0(arg0, arg1));
            return ret;
        }, arguments); },
        __wbg_String_8564e559799eccda: function() { return logError(function (arg0, arg1) {
            const ret = String(arg1);
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg___wbindgen_bigint_get_as_i64_b482365c149396c8: function(arg0, arg1) {
            const v = arg1;
            const ret = typeof(v) === 'bigint' ? v : undefined;
            if (!isLikeNone(ret)) {
                _assertBigInt(ret);
            }
            getDataViewMemory0().setBigInt64(arg0 + 8 * 1, isLikeNone(ret) ? BigInt(0) : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        },
        __wbg___wbindgen_boolean_get_7a12af2b3f899c5a: function(arg0) {
            const v = arg0;
            const ret = typeof(v) === 'boolean' ? v : undefined;
            if (!isLikeNone(ret)) {
                _assertBoolean(ret);
            }
            return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
        },
        __wbg___wbindgen_debug_string_0e68cf47c9cbd9b0: function(arg0, arg1) {
            const ret = debugString(arg1);
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_exports_3d3410a3b95d7b41: function() {
            const ret = wasm;
            return ret;
        },
        __wbg___wbindgen_function_table_69595ae5c7f77628: function() {
            const ret = wasm.__wbindgen_export;
            return ret;
        },
        __wbg___wbindgen_in_50072d4d6e45c193: function(arg0, arg1) {
            const ret = arg0 in arg1;
            _assertBoolean(ret);
            return ret;
        },
        __wbg___wbindgen_is_bigint_60fc0336cb14f5d7: function(arg0) {
            const ret = typeof(arg0) === 'bigint';
            _assertBoolean(ret);
            return ret;
        },
        __wbg___wbindgen_is_function_fcda5e3902d732fe: function(arg0) {
            const ret = typeof(arg0) === 'function';
            _assertBoolean(ret);
            return ret;
        },
        __wbg___wbindgen_is_object_edb6b15aa3afe12e: function(arg0) {
            const val = arg0;
            const ret = typeof(val) === 'object' && val !== null;
            _assertBoolean(ret);
            return ret;
        },
        __wbg___wbindgen_is_string_c4f7cb494a2a21f1: function(arg0) {
            const ret = typeof(arg0) === 'string';
            _assertBoolean(ret);
            return ret;
        },
        __wbg___wbindgen_is_undefined_8c687d0b90d5b524: function(arg0) {
            const ret = arg0 === undefined;
            _assertBoolean(ret);
            return ret;
        },
        __wbg___wbindgen_jsval_eq_9fdcd3c0a860dd3b: function(arg0, arg1) {
            const ret = arg0 === arg1;
            _assertBoolean(ret);
            return ret;
        },
        __wbg___wbindgen_jsval_loose_eq_3c30021c243b64cd: function(arg0, arg1) {
            const ret = arg0 == arg1;
            _assertBoolean(ret);
            return ret;
        },
        __wbg___wbindgen_memory_3f8442e22540244f: function() {
            const ret = wasm.memory;
            return ret;
        },
        __wbg___wbindgen_number_get_1dc732b810cb937c: function(arg0, arg1) {
            const obj = arg1;
            const ret = typeof(obj) === 'number' ? obj : undefined;
            if (!isLikeNone(ret)) {
                _assertNum(ret);
            }
            getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        },
        __wbg___wbindgen_string_get_92ab86bb19cbc12f: function(arg0, arg1) {
            const obj = arg1;
            const ret = typeof(obj) === 'string' ? obj : undefined;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_throw_5d9e815e6fdf150f: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg__wbg_cb_unref_997e73d32238e655: function() { return logError(function (arg0) {
            arg0._wbg_cb_unref();
        }, arguments); },
        __wbg_addEventListener_9534e2dc12044ee4: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            arg0.addEventListener(getStringFromWasm0(arg1, arg2), arg3);
        }, arguments); },
        __wbg_altKey_8b8e9b0d63b945bb: function() { return logError(function (arg0) {
            const ret = arg0.altKey;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_altKey_c75bf4903a5db4c2: function() { return logError(function (arg0) {
            const ret = arg0.altKey;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_altKey_fa381deee4b395cd: function() { return logError(function (arg0) {
            const ret = arg0.altKey;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_animationName_5f6e148b26109948: function() { return logError(function (arg0, arg1) {
            const ret = arg1.animationName;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_appendChild_0a61ee9a9da4556f: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.appendChild(arg1);
            return ret;
        }, arguments); },
        __wbg_arrayBuffer_06f3da76f071d37f: function() { return handleError(function (arg0) {
            const ret = arg0.arrayBuffer();
            return ret;
        }, arguments); },
        __wbg_arrayBuffer_432de2b072afe7fb: function() { return logError(function (arg0) {
            const ret = arg0.arrayBuffer();
            return ret;
        }, arguments); },
        __wbg_back_af42e9ac0b21eff5: function() { return handleError(function (arg0) {
            arg0.back();
        }, arguments); },
        __wbg_blockSize_726f0afadf8c1f3f: function() { return logError(function (arg0) {
            const ret = arg0.blockSize;
            return ret;
        }, arguments); },
        __wbg_blur_c712b137baa8b295: function() { return handleError(function (arg0) {
            arg0.blur();
        }, arguments); },
        __wbg_borderBoxSize_62234ce3ef78f33f: function() { return logError(function (arg0) {
            const ret = arg0.borderBoxSize;
            return ret;
        }, arguments); },
        __wbg_boundingClientRect_ea4c4d9e9e5be8cb: function() { return logError(function (arg0) {
            const ret = arg0.boundingClientRect;
            return ret;
        }, arguments); },
        __wbg_bubbles_454fa86c9a7c87f4: function() { return logError(function (arg0) {
            const ret = arg0.bubbles;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_button_f6aa74c65b739064: function() { return logError(function (arg0) {
            const ret = arg0.button;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_buttons_fc06c2cbd043b4af: function() { return logError(function (arg0) {
            const ret = arg0.buttons;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_byteLength_fd3df7acd57c50b3: function() { return logError(function (arg0) {
            const ret = arg0.byteLength;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_call_269c5566fbede3eb: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.call(arg1);
            return ret;
        }, arguments); },
        __wbg_call_6bcf8d3e20937e46: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.call(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_changedTouches_b982f6f6739b3f06: function() { return logError(function (arg0) {
            const ret = arg0.changedTouches;
            return ret;
        }, arguments); },
        __wbg_charCodeAt_90635349fbda9f5f: function() { return logError(function (arg0, arg1) {
            const ret = arg0.charCodeAt(arg1 >>> 0);
            return ret;
        }, arguments); },
        __wbg_checkValidity_530e82b8322fc0b3: function() { return logError(function (arg0) {
            const ret = arg0.checkValidity();
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_checked_ab679020f304032a: function() { return logError(function (arg0) {
            const ret = arg0.checked;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_clearData_20da58828b911102: function() { return handleError(function (arg0) {
            arg0.clearData();
        }, arguments); },
        __wbg_clearData_34c16f21ab9a9d3b: function() { return handleError(function (arg0, arg1, arg2) {
            arg0.clearData(getStringFromWasm0(arg1, arg2));
        }, arguments); },
        __wbg_clearTimeout_113b1cde814ec762: function() { return logError(function (arg0) {
            const ret = clearTimeout(arg0);
            return ret;
        }, arguments); },
        __wbg_clientHeight_8669994caeda015c: function() { return logError(function (arg0) {
            const ret = arg0.clientHeight;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_clientWidth_669677d957e958c7: function() { return logError(function (arg0) {
            const ret = arg0.clientWidth;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_clientX_3cd572a5cd368348: function() { return logError(function (arg0) {
            const ret = arg0.clientX;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_clientX_997500aa23d483d1: function() { return logError(function (arg0) {
            const ret = arg0.clientX;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_clientY_4651eed688551d1c: function() { return logError(function (arg0) {
            const ret = arg0.clientY;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_clientY_52b1b3958eb93e2d: function() { return logError(function (arg0) {
            const ret = arg0.clientY;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_code_6bc89190cbffee9e: function() { return logError(function (arg0) {
            const ret = arg0.code;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_code_de6c620a1a241f59: function() { return logError(function (arg0, arg1) {
            const ret = arg1.code;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_compile_fe2895fcc4d85949: function() { return logError(function (arg0) {
            const ret = WebAssembly.compile(arg0);
            return ret;
        }, arguments); },
        __wbg_contentBoxSize_da8ed0423f37fa73: function() { return logError(function (arg0) {
            const ret = arg0.contentBoxSize;
            return ret;
        }, arguments); },
        __wbg_createComment_2f844d723b100a86: function() { return logError(function (arg0, arg1, arg2) {
            const ret = arg0.createComment(getStringFromWasm0(arg1, arg2));
            return ret;
        }, arguments); },
        __wbg_createElementNS_a45335f2f24f9ec5: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            const ret = arg0.createElementNS(arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
            return ret;
        }, arguments); },
        __wbg_createElement_b9024dc5ba95ac27: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.createElement(getStringFromWasm0(arg1, arg2));
            return ret;
        }, arguments); },
        __wbg_createTextNode_6d2db3073c3d6df6: function() { return logError(function (arg0, arg1, arg2) {
            const ret = arg0.createTextNode(getStringFromWasm0(arg1, arg2));
            return ret;
        }, arguments); },
        __wbg_ctrlKey_785e802b8798af13: function() { return logError(function (arg0) {
            const ret = arg0.ctrlKey;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_ctrlKey_ba07ac4b2ea90600: function() { return logError(function (arg0) {
            const ret = arg0.ctrlKey;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_ctrlKey_df3eaa38d7d12de7: function() { return logError(function (arg0) {
            const ret = arg0.ctrlKey;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_dataTransfer_9d0a7bfb637e95b6: function() { return logError(function (arg0) {
            const ret = arg0.dataTransfer;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_data_3634610ec0d56ca6: function() { return logError(function (arg0, arg1) {
            const ret = arg1.data;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_data_aa7ba49b34c0d883: function() { return logError(function (arg0) {
            const ret = arg0.data;
            return ret;
        }, arguments); },
        __wbg_deltaMode_1f7b0a593ff1ea16: function() { return logError(function (arg0) {
            const ret = arg0.deltaMode;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_deltaX_948c3a4bd666561a: function() { return logError(function (arg0) {
            const ret = arg0.deltaX;
            return ret;
        }, arguments); },
        __wbg_deltaY_635e57899c7ee3a1: function() { return logError(function (arg0) {
            const ret = arg0.deltaY;
            return ret;
        }, arguments); },
        __wbg_deltaZ_9798555025e4da12: function() { return logError(function (arg0) {
            const ret = arg0.deltaZ;
            return ret;
        }, arguments); },
        __wbg_detail_84fdc765141875ba: function() { return logError(function (arg0) {
            const ret = arg0.detail;
            return ret;
        }, arguments); },
        __wbg_documentElement_02547989338a4499: function() { return logError(function (arg0) {
            const ret = arg0.documentElement;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_document_c7f486c52d63d24e: function() { return logError(function (arg0) {
            const ret = arg0.document;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_done_cffed884d87aa22e: function() { return logError(function (arg0) {
            const ret = arg0.done;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_dropEffect_3275a468cd6ac036: function() { return logError(function (arg0, arg1) {
            const ret = arg1.dropEffect;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_effectAllowed_abf0839c8e0d3235: function() { return logError(function (arg0, arg1) {
            const ret = arg1.effectAllowed;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_elapsedTime_e39b78687444eeeb: function() { return logError(function (arg0) {
            const ret = arg0.elapsedTime;
            return ret;
        }, arguments); },
        __wbg_elapsedTime_f5498294beaabde3: function() { return logError(function (arg0) {
            const ret = arg0.elapsedTime;
            return ret;
        }, arguments); },
        __wbg_entries_4936217d46734891: function() { return logError(function (arg0) {
            const ret = arg0.entries();
            return ret;
        }, arguments); },
        __wbg_entries_972a87586902cf87: function() { return logError(function (arg0) {
            const ret = Object.entries(arg0);
            return ret;
        }, arguments); },
        __wbg_error_619b76b6242cd1bc: function() { return logError(function (arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
            }
        }, arguments); },
        __wbg_error_756c5934221e6fee: function() { return logError(function (arg0) {
            console.error(arg0);
        }, arguments); },
        __wbg_exports_89c3f1bc156ad5da: function() { return logError(function (arg0) {
            const ret = arg0.exports;
            return ret;
        }, arguments); },
        __wbg_fetch_079086473aeec73d: function() { return logError(function (arg0, arg1, arg2) {
            const ret = arg0.fetch(getStringFromWasm0(arg1, arg2));
            return ret;
        }, arguments); },
        __wbg_fetch_8d9b732df7467c44: function() { return logError(function (arg0) {
            const ret = fetch(arg0);
            return ret;
        }, arguments); },
        __wbg_files_054ebec12ea09826: function() { return logError(function (arg0) {
            const ret = arg0.files;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_files_06af76c704b59946: function() { return logError(function (arg0) {
            const ret = arg0.files;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_focus_c1b2d4962209010b: function() { return handleError(function (arg0) {
            arg0.focus();
        }, arguments); },
        __wbg_force_a8103dca73d83d62: function() { return logError(function (arg0) {
            const ret = arg0.force;
            return ret;
        }, arguments); },
        __wbg_forward_29d58c4fd6fff512: function() { return handleError(function (arg0) {
            arg0.forward();
        }, arguments); },
        __wbg_getAsFile_e64a2869e682dcc6: function() { return handleError(function (arg0) {
            const ret = arg0.getAsFile();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_getAttribute_b12f8535a2cb5368: function() { return logError(function (arg0, arg1, arg2, arg3) {
            const ret = arg1.getAttribute(getStringFromWasm0(arg2, arg3));
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_getBoundingClientRect_299f40f8641457e6: function() { return logError(function (arg0) {
            const ret = arg0.getBoundingClientRect();
            return ret;
        }, arguments); },
        __wbg_getData_b19c87e3887bc0cb: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = arg1.getData(getStringFromWasm0(arg2, arg3));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_getElementById_ccc92d66acf76819: function() { return logError(function (arg0, arg1, arg2) {
            const ret = arg0.getElementById(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_getMetaContents_7aabefc0d962fc2d: function() { return logError(function (arg0, arg1, arg2) {
            const ret = getMetaContents(getStringFromWasm0(arg1, arg2));
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_getNode_b14ec18ffa4892a9: function() { return logError(function (arg0, arg1) {
            const ret = arg0.getNode(arg1 >>> 0);
            return ret;
        }, arguments); },
        __wbg_get_501ad1d6b5ba20fe: function() { return logError(function (arg0, arg1) {
            const ret = arg0[arg1 >>> 0];
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_get_6cf5a4d4d8ad3c5a: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_get_95e4d462165c92ae: function() { return logError(function (arg0, arg1) {
            const ret = arg0[arg1 >>> 0];
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_get_989d0a1309644f2b: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_get_b1f0ab13c737f856: function() { return logError(function (arg0, arg1) {
            const ret = arg0[arg1 >>> 0];
            return ret;
        }, arguments); },
        __wbg_get_c3db89fbf291e5ab: function() { return logError(function (arg0, arg1) {
            const ret = arg0[arg1 >>> 0];
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_get_select_data_44a585f337de4182: function() { return logError(function (arg0, arg1) {
            const ret = get_select_data(arg1);
            const ptr1 = passArrayJsValueToWasm0(ret, wasm.__wbindgen_malloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_get_unchecked_363572bdd397d473: function() { return logError(function (arg0, arg1) {
            const ret = arg0[arg1 >>> 0];
            return ret;
        }, arguments); },
        __wbg_grow_7200a2fb695bf512: function() { return logError(function (arg0, arg1) {
            const ret = arg0.grow(arg1 >>> 0);
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_grow_fc949adcbe8a9b1d: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.grow(arg1 >>> 0);
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_hash_04b938b4a921e95e: function() { return handleError(function (arg0, arg1) {
            const ret = arg1.hash;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_head_1dae591341fbaa2b: function() { return logError(function (arg0) {
            const ret = arg0.head;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_height_24586a23e4789d28: function() { return logError(function (arg0) {
            const ret = arg0.height;
            return ret;
        }, arguments); },
        __wbg_height_d011730144e088ea: function() { return logError(function (arg0) {
            const ret = arg0.height;
            return ret;
        }, arguments); },
        __wbg_height_f390cf417f3cf99d: function() { return logError(function (arg0) {
            const ret = arg0.height;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_history_9d7bf9676b7c736c: function() { return handleError(function (arg0) {
            const ret = arg0.history;
            return ret;
        }, arguments); },
        __wbg_host_1cf263d95b06dac3: function() { return handleError(function (arg0, arg1) {
            const ret = arg1.host;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_identifier_33e3b1ed04ae35c8: function() { return logError(function (arg0) {
            const ret = arg0.identifier;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_initialize_6d1129af221f598b: function() { return logError(function (arg0, arg1, arg2) {
            arg0.initialize(arg1, arg2);
        }, arguments); },
        __wbg_inlineSize_199278605b83587a: function() { return logError(function (arg0) {
            const ret = arg0.inlineSize;
            return ret;
        }, arguments); },
        __wbg_instanceof_AnimationEvent_fd2fa1d231fd0d23: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof AnimationEvent;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_ArrayBuffer_d4ff01f8247925ae: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof ArrayBuffer;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_Blob_38df052cf775f822: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof Blob;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_CompositionEvent_179872ea787b2ebe: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof CompositionEvent;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_CustomEvent_f65ee8ab12ac87f9: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof CustomEvent;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_Document_9284a3da8a2e4796: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof Document;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_DragEvent_0aaee068b6b4ba41: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof DragEvent;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_Element_c95c45635d03047e: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof Element;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_Error_fe6fa771c78ee4cf: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof Error;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_Event_a4e4dbe20b01838b: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof Event;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_File_388508da6a606d0d: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof File;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_FocusEvent_1322b267a999bc0d: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof FocusEvent;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_HtmlElement_bd89ce6abf9dad44: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof HTMLElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_HtmlFormElement_de556433be5fcd15: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof HTMLFormElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_HtmlInputElement_086894dc9c868a64: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof HTMLInputElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_HtmlSelectElement_2be52e2158c4f399: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof HTMLSelectElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_HtmlTextAreaElement_632129eba25e4367: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof HTMLTextAreaElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_KeyboardEvent_e7ef7527c166dc71: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof KeyboardEvent;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_Map_1ff6a2b54c899f0d: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof Map;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_MouseEvent_7b6ebd01ef5907f7: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof MouseEvent;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_Node_5b8bea1a7005fbb9: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof Node;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_PointerEvent_4269e92f16379aea: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof PointerEvent;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_Response_6366a785e400039b: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof Response;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_TouchEvent_19242dc3c8d02052: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof TouchEvent;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_TransitionEvent_12d35df2a2af4c7c: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof TransitionEvent;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_Uint8Array_598adc0fef426aa8: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof Uint8Array;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_WheelEvent_82ec56ef53ac3877: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof WheelEvent;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instanceof_Window_a3b8566f0a9c5d1a: function() { return logError(function (arg0) {
            let result;
            try {
                result = arg0 instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_instantiate_884602f4730130a7: function() { return logError(function (arg0, arg1) {
            const ret = WebAssembly.instantiate(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_intersectionRatio_dcd7062de83ff1ca: function() { return logError(function (arg0) {
            const ret = arg0.intersectionRatio;
            return ret;
        }, arguments); },
        __wbg_intersectionRect_574cd4412c445618: function() { return logError(function (arg0) {
            const ret = arg0.intersectionRect;
            return ret;
        }, arguments); },
        __wbg_isArray_5674713bb7b79043: function() { return logError(function (arg0) {
            const ret = Array.isArray(arg0);
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_isComposing_2ab72d608a0a8d0e: function() { return logError(function (arg0) {
            const ret = arg0.isComposing;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_isIntersecting_4b154fb6df9d9a6d: function() { return logError(function (arg0) {
            const ret = arg0.isIntersecting;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_isPrimary_6a50a5efd7086e89: function() { return logError(function (arg0) {
            const ret = arg0.isPrimary;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_isSafeInteger_8f51c743827d1ec5: function() { return logError(function (arg0) {
            const ret = Number.isSafeInteger(arg0);
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_item_5ad6a68f51ea4b33: function() { return logError(function (arg0, arg1) {
            const ret = arg0.item(arg1 >>> 0);
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_items_a8fdc4cf728ef922: function() { return logError(function (arg0) {
            const ret = arg0.items;
            return ret;
        }, arguments); },
        __wbg_iterator_22ddeb808cf55a6f: function() { return logError(function () {
            const ret = Symbol.iterator;
            return ret;
        }, arguments); },
        __wbg_js_schedule_toast_b015f534acbfa9a4: function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg4;
                deferred0_1 = arg5;
                js_schedule_toast(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3), getStringFromWasm0(arg4, arg5), arg6 >>> 0);
            } finally {
                wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
            }
        }, arguments); },
        __wbg_js_show_toast_9de6e85dcb7c27b0: function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg4;
                deferred0_1 = arg5;
                js_show_toast(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3), getStringFromWasm0(arg4, arg5), arg6 >>> 0);
            } finally {
                wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
            }
        }, arguments); },
        __wbg_key_d29d670923b81510: function() { return logError(function (arg0, arg1) {
            const ret = arg1.key;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_keys_6efc298980178da1: function() { return logError(function (arg0) {
            const ret = Object.keys(arg0);
            return ret;
        }, arguments); },
        __wbg_kind_ed2088ba17535d91: function() { return logError(function (arg0, arg1) {
            const ret = arg1.kind;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_lastModified_a90bb7a34b384061: function() { return logError(function (arg0) {
            const ret = arg0.lastModified;
            return ret;
        }, arguments); },
        __wbg_left_644de747afd08e57: function() { return logError(function (arg0) {
            const ret = arg0.left;
            return ret;
        }, arguments); },
        __wbg_length_2a004c1681a133a1: function() { return logError(function (arg0) {
            const ret = arg0.length;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_length_31bdaf014f5fbde2: function() { return logError(function (arg0) {
            const ret = arg0.length;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_length_4e1adc0d42e23620: function() { return logError(function (arg0) {
            const ret = arg0.length;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_length_b9633f9f50fac3c7: function() { return logError(function (arg0) {
            const ret = arg0.length;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_length_c0ad739f2efc455b: function() { return logError(function (arg0) {
            const ret = arg0.length;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_length_d52ee452c5be0c99: function() { return logError(function (arg0) {
            const ret = arg0.length;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_length_f97c9822ab91d4f4: function() { return logError(function (arg0) {
            const ret = arg0.length;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_location_382075cf6a393ad4: function() { return logError(function (arg0) {
            const ret = arg0.location;
            return ret;
        }, arguments); },
        __wbg_location_4cfd4b7727d1ad11: function() { return logError(function (arg0) {
            const ret = arg0.location;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_log_1f8cbb01c83d06c2: function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.log(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3), getStringFromWasm0(arg4, arg5), getStringFromWasm0(arg6, arg7));
            } finally {
                wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
            }
        }, arguments); },
        __wbg_log_a54ca6b45e09078a: function() { return logError(function (arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.log(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
            }
        }, arguments); },
        __wbg_mark_6b7f03786f5e4d61: function() { return logError(function (arg0, arg1) {
            performance.mark(getStringFromWasm0(arg0, arg1));
        }, arguments); },
        __wbg_measure_0e21b33a1c6e3a29: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            let deferred0_0;
            let deferred0_1;
            let deferred1_0;
            let deferred1_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                deferred1_0 = arg2;
                deferred1_1 = arg3;
                performance.measure(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
            } finally {
                wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
                wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
            }
        }, arguments); },
        __wbg_message_1cbc5bc03dcf1dee: function() { return logError(function (arg0) {
            const ret = arg0.message;
            return ret;
        }, arguments); },
        __wbg_metaKey_3f9183dfd5e1b726: function() { return logError(function (arg0) {
            const ret = arg0.metaKey;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_metaKey_c085a7ea7d9587dd: function() { return logError(function (arg0) {
            const ret = arg0.metaKey;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_metaKey_feb71e77f26b1a4c: function() { return logError(function (arg0) {
            const ret = arg0.metaKey;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_name_2290eb6a0f9bb82e: function() { return logError(function (arg0, arg1) {
            const ret = arg1.name;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_name_9240af9c79ef35aa: function() { return logError(function (arg0, arg1) {
            const ret = arg1.name;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_name_b9d8f2ea16b22045: function() { return logError(function (arg0) {
            const ret = arg0.name;
            return ret;
        }, arguments); },
        __wbg_new_0afe64b4dc16ab74: function() { return handleError(function () {
            const ret = new Headers();
            return ret;
        }, arguments); },
        __wbg_new_1da3429bc3c4541c: function() { return logError(function (arg0) {
            const ret = new Uint8Array(arg0);
            return ret;
        }, arguments); },
        __wbg_new_2fa8dd8f40f14923: function() { return handleError(function (arg0, arg1) {
            const ret = new URL(getStringFromWasm0(arg0, arg1));
            return ret;
        }, arguments); },
        __wbg_new_557b2c3ddb611852: function() { return logError(function () {
            const ret = new Error();
            return ret;
        }, arguments); },
        __wbg_new_82926b2a4d30e175: function() { return handleError(function (arg0, arg1) {
            const ret = new WebSocket(getStringFromWasm0(arg0, arg1));
            return ret;
        }, arguments); },
        __wbg_new_8d36e20aa758e411: function() { return logError(function () {
            const ret = new Map();
            return ret;
        }, arguments); },
        __wbg_new_8f72ed7c652bf5a2: function() { return logError(function (arg0, arg1) {
            try {
                var state0 = {a: arg0, b: arg1};
                var cb0 = (arg0, arg1) => {
                    const a = state0.a;
                    state0.a = 0;
                    try {
                        return _RINvNvNtNtCsiXDZd1WmBpW_12wasm_bindgen7convert8closuress4_1__6invokeNtCs4ySu89kE9MV_6js_sys8FunctionB14_uKb1_EB16_(a, state0.b, arg0, arg1);
                    } finally {
                        state0.a = a;
                    }
                };
                const ret = new Promise(cb0);
                return ret;
            } finally {
                state0.a = 0;
            }
        }, arguments); },
        __wbg_new_9952c6712c73a634: function() { return handleError(function () {
            const ret = new DataTransfer();
            return ret;
        }, arguments); },
        __wbg_new_bebc3f4757acf305: function() { return logError(function () {
            const ret = new Object();
            return ret;
        }, arguments); },
        __wbg_new_c5dc7c0518fdb0ba: function() { return logError(function (arg0) {
            const ret = new RawInterpreter(arg0 >>> 0);
            return ret;
        }, arguments); },
        __wbg_new_e60365bfd84fce6d: function() { return handleError(function () {
            const ret = new URLSearchParams();
            return ret;
        }, arguments); },
        __wbg_new_e942d9e3968e46f8: function() { return logError(function (arg0) {
            const ret = new WebDioxusChannel(JSOwner.__wrap(arg0));
            return ret;
        }, arguments); },
        __wbg_new_e9fffa3c467d7786: function() { return handleError(function () {
            const ret = new FileReader();
            return ret;
        }, arguments); },
        __wbg_new_eba495809764b79b: function() { return handleError(function (arg0, arg1) {
            const ret = new WebAssembly.Global(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_new_ffa92086ea89f79c: function() { return logError(function () {
            const ret = new Array();
            return ret;
        }, arguments); },
        __wbg_new_with_args_6cd2cdadf5e23940: function() { return logError(function (arg0, arg1, arg2, arg3) {
            const ret = new Function(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
            return ret;
        }, arguments); },
        __wbg_new_with_form_dbba564808b01fa6: function() { return handleError(function (arg0) {
            const ret = new FormData(arg0);
            return ret;
        }, arguments); },
        __wbg_new_with_str_3827d1a2319e0426: function() { return handleError(function (arg0, arg1) {
            const ret = new Request(getStringFromWasm0(arg0, arg1));
            return ret;
        }, arguments); },
        __wbg_new_with_str_and_init_2f31a77deda127fd: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = new Request(getStringFromWasm0(arg0, arg1), arg2);
            return ret;
        }, arguments); },
        __wbg_next_95053e306b1c3aed: function() { return logError(function (arg0) {
            const ret = arg0.next;
            return ret;
        }, arguments); },
        __wbg_next_f31ecb8646d2c605: function() { return handleError(function (arg0) {
            const ret = arg0.next();
            return ret;
        }, arguments); },
        __wbg_offsetX_5b637acbf4fc40e8: function() { return logError(function (arg0) {
            const ret = arg0.offsetX;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_offsetY_ad9ca8949500ce96: function() { return logError(function (arg0) {
            const ret = arg0.offsetY;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_ok_0d8e8fee3573b7ce: function() { return logError(function (arg0) {
            const ret = arg0.ok;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_open_060ef0a925d5962b: function() { return logError(function (arg0, arg1, arg2, arg3) {
            window.open(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
        }, arguments); },
        __wbg_ownerDocument_5e91fd7a8aefc7cc: function() { return logError(function (arg0) {
            const ret = arg0.ownerDocument;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_pageX_6b1a2c1b3f3be830: function() { return logError(function (arg0) {
            const ret = arg0.pageX;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_pageX_7794a7bd22e64abf: function() { return logError(function (arg0) {
            const ret = arg0.pageX;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_pageY_353ef292c5f0d0cd: function() { return logError(function (arg0) {
            const ret = arg0.pageY;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_pageY_b268c6e19acefd4e: function() { return logError(function (arg0) {
            const ret = arg0.pageY;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_parentElement_c0b090e8023d3df2: function() { return logError(function (arg0) {
            const ret = arg0.parentElement;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_pathname_1c74e1a4da38e765: function() { return handleError(function (arg0, arg1) {
            const ret = arg1.pathname;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_pointerId_b8c496f0172b1473: function() { return logError(function (arg0) {
            const ret = arg0.pointerId;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_pointerType_753a9c4f0e3018cc: function() { return logError(function (arg0, arg1) {
            const ret = arg1.pointerType;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_pressure_1021f185ca9ffe6b: function() { return logError(function (arg0) {
            const ret = arg0.pressure;
            return ret;
        }, arguments); },
        __wbg_preventDefault_4235a4ccf8540533: function() { return logError(function (arg0) {
            arg0.preventDefault();
        }, arguments); },
        __wbg_propertyName_cf392fa6f240f867: function() { return logError(function (arg0, arg1) {
            const ret = arg1.propertyName;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_protocol_0b0f691870228b69: function() { return handleError(function (arg0, arg1) {
            const ret = arg1.protocol;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_prototypesetcall_ae9f5e7459250748: function() { return logError(function (arg0, arg1, arg2) {
            Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
        }, arguments); },
        __wbg_pseudoElement_262f4289f77f4f4b: function() { return logError(function (arg0, arg1) {
            const ret = arg1.pseudoElement;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_pseudoElement_2ae0d1302ef6948a: function() { return logError(function (arg0, arg1) {
            const ret = arg1.pseudoElement;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_pushState_1eded2f6852c83e7: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.pushState(arg1, getStringFromWasm0(arg2, arg3), arg4 === 0 ? undefined : getStringFromWasm0(arg4, arg5));
        }, arguments); },
        __wbg_push_bfdf956ba476f65b: function() { return logError(function (arg0, arg1) {
            const ret = arg0.push(arg1);
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_querySelectorAll_fdbf93e11103921d: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.querySelectorAll(getStringFromWasm0(arg1, arg2));
            return ret;
        }, arguments); },
        __wbg_queueMicrotask_85c90f6987555d65: function() { return logError(function (arg0) {
            const ret = arg0.queueMicrotask;
            return ret;
        }, arguments); },
        __wbg_queueMicrotask_f6a1fa10b81d1fc0: function() { return logError(function (arg0) {
            queueMicrotask(arg0);
        }, arguments); },
        __wbg_radiusX_4f13056638241bcc: function() { return logError(function (arg0) {
            const ret = arg0.radiusX;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_radiusY_43aba04cd6f66d1d: function() { return logError(function (arg0) {
            const ret = arg0.radiusY;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_random_7bce4e9e78e2e0d0: function() { return logError(function () {
            const ret = Math.random();
            return ret;
        }, arguments); },
        __wbg_readAsArrayBuffer_ef9c26393f5c58b1: function() { return handleError(function (arg0, arg1) {
            arg0.readAsArrayBuffer(arg1);
        }, arguments); },
        __wbg_readAsText_6aee528cd111ea76: function() { return handleError(function (arg0, arg1) {
            arg0.readAsText(arg1);
        }, arguments); },
        __wbg_reload_80e06bd80f18101c: function() { return handleError(function (arg0) {
            arg0.reload();
        }, arguments); },
        __wbg_repeat_01e85ccb1bcef326: function() { return logError(function (arg0) {
            const ret = arg0.repeat;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_replaceState_6542bba3c2e92e5e: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.replaceState(arg1, getStringFromWasm0(arg2, arg3), arg4 === 0 ? undefined : getStringFromWasm0(arg4, arg5));
        }, arguments); },
        __wbg_requestAnimationFrame_4988887658fcee2e: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.requestAnimationFrame(arg1);
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_resolve_35ec7e0c6af4c82c: function() { return logError(function (arg0) {
            const ret = Promise.resolve(arg0);
            return ret;
        }, arguments); },
        __wbg_result_6a6fab012d5f9cc1: function() { return handleError(function (arg0) {
            const ret = arg0.result;
            return ret;
        }, arguments); },
        __wbg_rootBounds_73318b4dba5bec4a: function() { return logError(function (arg0) {
            const ret = arg0.rootBounds;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_rotationAngle_c94da25554ec8611: function() { return logError(function (arg0) {
            const ret = arg0.rotationAngle;
            return ret;
        }, arguments); },
        __wbg_run_005df3744a5aa7cf: function() { return logError(function (arg0) {
            arg0.run();
        }, arguments); },
        __wbg_run_f19543b086e90d43: function() { return logError(function (arg0, arg1, arg2) {
            try {
                var state0 = {a: arg1, b: arg2};
                var cb0 = () => {
                    const a = state0.a;
                    state0.a = 0;
                    try {
                        return _RINvNvNtNtCsiXDZd1WmBpW_12wasm_bindgen7convert8closuress0_1__6invokebKb1_ECs4ySu89kE9MV_6js_sys(a, state0.b, );
                    } finally {
                        state0.a = a;
                    }
                };
                const ret = arg0.run(cb0);
                _assertBoolean(ret);
                return ret;
            } finally {
                state0.a = 0;
            }
        }, arguments); },
        __wbg_rustRecv_c9dcac227ecc00d9: function() { return logError(function (arg0) {
            const ret = arg0.rustRecv();
            return ret;
        }, arguments); },
        __wbg_rustSend_56478f16de514546: function() { return logError(function (arg0, arg1) {
            arg0.rustSend(arg1);
        }, arguments); },
        __wbg_saveTemplate_e3c1a1e2ac405343: function() { return logError(function (arg0, arg1, arg2, arg3) {
            var v0 = getArrayJsValueFromWasm0(arg1, arg2);
            wasm.__wbindgen_free(arg1, arg2 * 4, 4);
            arg0.saveTemplate(v0, arg3);
        }, arguments); },
        __wbg_screenX_174b66ca193d04ef: function() { return logError(function (arg0) {
            const ret = arg0.screenX;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_screenX_ae667756b820f7c4: function() { return logError(function (arg0) {
            const ret = arg0.screenX;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_screenY_5af62c383e06b59b: function() { return logError(function (arg0) {
            const ret = arg0.screenY;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_screenY_dd328b61a5d3fe3f: function() { return logError(function (arg0) {
            const ret = arg0.screenY;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_scrollHeight_b1a7772dca8769f0: function() { return logError(function (arg0) {
            const ret = arg0.scrollHeight;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_scrollIntoView_326d8d17701e2108: function() { return logError(function (arg0, arg1) {
            arg0.scrollIntoView(arg1);
        }, arguments); },
        __wbg_scrollLeft_5bebc8ba042c2371: function() { return logError(function (arg0) {
            const ret = arg0.scrollLeft;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_scrollTo_7819e59a92b20e23: function() { return logError(function (arg0, arg1, arg2) {
            arg0.scrollTo(arg1, arg2);
        }, arguments); },
        __wbg_scrollTop_77a6c2418331d800: function() { return logError(function (arg0) {
            const ret = arg0.scrollTop;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_scrollWidth_a9140c22eb2b68c3: function() { return logError(function (arg0) {
            const ret = arg0.scrollWidth;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_scrollX_8ded7e341bd8c2c2: function() { return handleError(function (arg0) {
            const ret = arg0.scrollX;
            return ret;
        }, arguments); },
        __wbg_scrollY_d14087fece6de342: function() { return handleError(function (arg0) {
            const ret = arg0.scrollY;
            return ret;
        }, arguments); },
        __wbg_scroll_ced8afbb54bc6212: function() { return logError(function (arg0, arg1) {
            arg0.scroll(arg1);
        }, arguments); },
        __wbg_search_685c8e05aa5dd9ba: function() { return handleError(function (arg0, arg1) {
            const ret = arg1.search;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_search_6b8de53571c80faa: function() { return logError(function (arg0, arg1) {
            const ret = arg1.search;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_setAttributeInner_a29bef1cf5d53ffe: function() { return logError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            setAttributeInner(arg0, getStringFromWasm0(arg1, arg2), arg3, arg4 === 0 ? undefined : getStringFromWasm0(arg4, arg5));
        }, arguments); },
        __wbg_setAttribute_8c84e9351986b1f6: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_setData_f099ea2a3a271149: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.setData(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_setTimeout_7b1dfbeaccbf9bb6: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.setTimeout(arg1, arg2);
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_setTimeout_c202321c38d85688: function() { return logError(function (arg0, arg1) {
            window.setTimeout(arg0, arg1);
        }, arguments); },
        __wbg_setTimeout_ef24d2fc3ad97385: function() { return handleError(function (arg0, arg1) {
            const ret = setTimeout(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_set_13d25b81ab403f5e: function() { return logError(function (arg0, arg1, arg2) {
            arg0[arg1 >>> 0] = arg2;
        }, arguments); },
        __wbg_set_6be42768c690e380: function() { return logError(function (arg0, arg1, arg2) {
            arg0[arg1] = arg2;
        }, arguments); },
        __wbg_set_7923e5ea63b41e6b: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.set(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_set_a377297433dfea63: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = Reflect.set(arg0, arg1, arg2);
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_set_behavior_7ea9ff4d9986a2a1: function() { return logError(function (arg0, arg1) {
            arg0.behavior = __wbindgen_enum_ScrollBehavior[arg1];
        }, arguments); },
        __wbg_set_behavior_fc7a62e8f5217b27: function() { return logError(function (arg0, arg1) {
            arg0.behavior = __wbindgen_enum_ScrollBehavior[arg1];
        }, arguments); },
        __wbg_set_bf6dde4923b9b059: function() { return logError(function (arg0, arg1, arg2) {
            const ret = arg0.set(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_set_block_6882e6bcd1f9b921: function() { return logError(function (arg0, arg1) {
            arg0.block = __wbindgen_enum_ScrollLogicalPosition[arg1];
        }, arguments); },
        __wbg_set_body_f39cee72c74a5b02: function() { return logError(function (arg0, arg1) {
            arg0.body = arg1;
        }, arguments); },
        __wbg_set_dropEffect_9a560c6fdae25981: function() { return logError(function (arg0, arg1, arg2) {
            arg0.dropEffect = getStringFromWasm0(arg1, arg2);
        }, arguments); },
        __wbg_set_effectAllowed_8add52099a6a9b26: function() { return logError(function (arg0, arg1, arg2) {
            arg0.effectAllowed = getStringFromWasm0(arg1, arg2);
        }, arguments); },
        __wbg_set_headers_31d373f68f28bf76: function() { return logError(function (arg0, arg1) {
            arg0.headers = arg1;
        }, arguments); },
        __wbg_set_href_af814f650e329fca: function() { return handleError(function (arg0, arg1, arg2) {
            arg0.href = getStringFromWasm0(arg1, arg2);
        }, arguments); },
        __wbg_set_inline_4f25c8826ef77a01: function() { return logError(function (arg0, arg1) {
            arg0.inline = __wbindgen_enum_ScrollLogicalPosition[arg1];
        }, arguments); },
        __wbg_set_left_63bbb8fe4ecd10d4: function() { return logError(function (arg0, arg1) {
            arg0.left = arg1;
        }, arguments); },
        __wbg_set_method_82e6c3c083da0734: function() { return logError(function (arg0, arg1, arg2) {
            arg0.method = getStringFromWasm0(arg1, arg2);
        }, arguments); },
        __wbg_set_onclose_97aab74a1c2662f7: function() { return logError(function (arg0, arg1) {
            arg0.onclose = arg1;
        }, arguments); },
        __wbg_set_onload_b3d8ff8ccbd7017c: function() { return logError(function (arg0, arg1) {
            arg0.onload = arg1;
        }, arguments); },
        __wbg_set_onmessage_0f0d391465909a45: function() { return logError(function (arg0, arg1) {
            arg0.onmessage = arg1;
        }, arguments); },
        __wbg_set_onopen_e91e64af99b7966d: function() { return logError(function (arg0, arg1) {
            arg0.onopen = arg1;
        }, arguments); },
        __wbg_set_scrollRestoration_94338f6cad660cfc: function() { return handleError(function (arg0, arg1) {
            arg0.scrollRestoration = __wbindgen_enum_ScrollRestoration[arg1];
        }, arguments); },
        __wbg_set_search_4282ef5d8c36f833: function() { return logError(function (arg0, arg1, arg2) {
            arg0.search = getStringFromWasm0(arg1, arg2);
        }, arguments); },
        __wbg_set_textContent_729c78ca859aa5a5: function() { return logError(function (arg0, arg1, arg2) {
            arg0.textContent = arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2);
        }, arguments); },
        __wbg_set_top_330d6e8d1d2f9427: function() { return logError(function (arg0, arg1) {
            arg0.top = arg1;
        }, arguments); },
        __wbg_shiftKey_1daef5d8d06d4247: function() { return logError(function (arg0) {
            const ret = arg0.shiftKey;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_shiftKey_2b651c3e5a9c07f5: function() { return logError(function (arg0) {
            const ret = arg0.shiftKey;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_shiftKey_9dd20e0fb9da6eb5: function() { return logError(function (arg0) {
            const ret = arg0.shiftKey;
            _assertBoolean(ret);
            return ret;
        }, arguments); },
        __wbg_size_338f1717fbf84c4e: function() { return logError(function (arg0) {
            const ret = arg0.size;
            return ret;
        }, arguments); },
        __wbg_stack_fb16a27564f12021: function() { return logError(function (arg0, arg1) {
            const ret = arg1.stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_state_a31aaa3b93bc1c7b: function() { return handleError(function (arg0) {
            const ret = arg0.state;
            return ret;
        }, arguments); },
        __wbg_static_accessor_CREATE_TASK_04be768c473932c0: function() { return logError(function () {
            const ret = typeof console === 'undefined' ? null : console?.createTask;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_static_accessor_GLOBAL_8eb4cd83130a11a0: function() { return logError(function () {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_static_accessor_GLOBAL_THIS_1e7044f654e934db: function() { return logError(function () {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_static_accessor_SELF_d8b50611246a6d92: function() { return logError(function () {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_static_accessor_WINDOW_fd0bc376bf0f8b42: function() { return logError(function () {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_statusText_d25b9b3ccc0d4617: function() { return logError(function (arg0, arg1) {
            const ret = arg1.statusText;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_stringify_54b3d9b61602aee6: function() { return handleError(function (arg0) {
            const ret = JSON.stringify(arg0);
            return ret;
        }, arguments); },
        __wbg_tangentialPressure_55610d0a052445d6: function() { return logError(function (arg0) {
            const ret = arg0.tangentialPressure;
            return ret;
        }, arguments); },
        __wbg_targetTouches_55d7e91b9ddab16f: function() { return logError(function (arg0) {
            const ret = arg0.targetTouches;
            return ret;
        }, arguments); },
        __wbg_target_3e80dc66a5936e87: function() { return logError(function (arg0) {
            const ret = arg0.target;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_textContent_d7e4a707af82657f: function() { return logError(function (arg0, arg1) {
            const ret = arg1.textContent;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_text_b00323a194b10fa8: function() { return handleError(function (arg0) {
            const ret = arg0.text();
            return ret;
        }, arguments); },
        __wbg_then_7a850dae4493f353: function() { return logError(function (arg0, arg1, arg2) {
            const ret = arg0.then(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_then_b830475380919203: function() { return logError(function (arg0, arg1) {
            const ret = arg0.then(arg1);
            return ret;
        }, arguments); },
        __wbg_tiltX_1ce87af3e8da3cde: function() { return logError(function (arg0) {
            const ret = arg0.tiltX;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_tiltY_0479d3e1658b4b17: function() { return logError(function (arg0) {
            const ret = arg0.tiltY;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_time_cc91f0f6671cd512: function() { return logError(function (arg0) {
            const ret = arg0.time;
            return ret;
        }, arguments); },
        __wbg_toString_794a30e85c994f2a: function() { return logError(function (arg0) {
            const ret = arg0.toString();
            return ret;
        }, arguments); },
        __wbg_toString_beee8ad71195e063: function() { return logError(function (arg0) {
            const ret = arg0.toString();
            return ret;
        }, arguments); },
        __wbg_top_319e8c7a40daacf1: function() { return logError(function (arg0) {
            const ret = arg0.top;
            return ret;
        }, arguments); },
        __wbg_touches_fcb578b82332de47: function() { return logError(function (arg0) {
            const ret = arg0.touches;
            return ret;
        }, arguments); },
        __wbg_twist_1a2feaa75e84cd1e: function() { return logError(function (arg0) {
            const ret = arg0.twist;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_type_9a9a7042e88baa94: function() { return logError(function (arg0, arg1) {
            const ret = arg1.type;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_type_de0ef83ccf892533: function() { return logError(function (arg0, arg1) {
            const ret = arg1.type;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_type_e9e1e62b9a2c4682: function() { return logError(function (arg0, arg1) {
            const ret = arg1.type;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_update_memory_4060a0ab30633318: function() { return logError(function (arg0, arg1) {
            arg0.update_memory(arg1);
        }, arguments); },
        __wbg_url_8d180a4a6996e22d: function() { return logError(function (arg0, arg1) {
            const ret = arg1.url;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_value_000080a4486e3ed3: function() { return logError(function (arg0, arg1) {
            const ret = arg1.value;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_value_8215ae6d5fec3d34: function() { return logError(function (arg0, arg1) {
            const ret = arg1.value;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_value_88dbf3c61c816e23: function() { return logError(function (arg0, arg1) {
            const ret = arg1.value;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_value_c227f843d21da141: function() { return logError(function (arg0) {
            const ret = arg0.value;
            return ret;
        }, arguments); },
        __wbg_weak_76ec38e642028ca7: function() { return logError(function (arg0) {
            const ret = arg0.weak();
            return ret;
        }, arguments); },
        __wbg_width_66ae60ec9345f56c: function() { return logError(function (arg0) {
            const ret = arg0.width;
            return ret;
        }, arguments); },
        __wbg_width_71b671f940e3eb8c: function() { return logError(function (arg0) {
            const ret = arg0.width;
            return ret;
        }, arguments); },
        __wbg_width_8d6f7dacd4cd7c61: function() { return logError(function (arg0) {
            const ret = arg0.width;
            _assertNum(ret);
            return ret;
        }, arguments); },
        __wbg_write_text_82e57241a0088f14: function() { return logError(function (arg0, arg1) {
            window.navigator.clipboard.write_text(getStringFromWasm0(arg0, arg1));
        }, arguments); },
        __wbg_x_67554e02a6b67f0e: function() { return logError(function (arg0) {
            const ret = arg0.x;
            return ret;
        }, arguments); },
        __wbg_y_8c87002a78bcd08f: function() { return logError(function (arg0) {
            const ret = arg0.y;
            return ret;
        }, arguments); },
        __wbindgen_generic_0000000000000001: function() { return logError(function (arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 2024, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, _RINvNvNtNtCsiXDZd1WmBpW_12wasm_bindgen7convert8closuress2_1__6invokeNtB8_7JsValueINtNtCsknUcikIyyBm_4core6result6ResultuNtB8_7JsErrorEKb1_ECs4ySu89kE9MV_6js_sys);
            return ret;
        }, arguments); },
        __wbindgen_generic_0000000000000002: function() { return logError(function (arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("CloseEvent")], shim_idx: 1414, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, _RINvNvNtNtCsiXDZd1WmBpW_12wasm_bindgen7convert8closuress2_1__6invokeNtNtNtCslsL5eFSWZQN_7web_sys8features14gen_CloseEvent10CloseEventuKb1_ECslPD5AW6YMoR_10dioxus_web);
            return ret;
        }, arguments); },
        __wbindgen_generic_0000000000000003: function() { return logError(function (arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("Event")], shim_idx: 1418, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, _RINvNvNtNtCsiXDZd1WmBpW_12wasm_bindgen7convert8closuress2_1__6invokeNtNtNtCslsL5eFSWZQN_7web_sys8features9gen_Event5EventuKb1_ECslPD5AW6YMoR_10dioxus_web);
            return ret;
        }, arguments); },
        __wbindgen_generic_0000000000000004: function() { return logError(function (arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("MessageEvent")], shim_idx: 1416, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, _RINvNvNtNtCsiXDZd1WmBpW_12wasm_bindgen7convert8closuress2_1__6invokeNtNtNtCslsL5eFSWZQN_7web_sys8features16gen_MessageEvent12MessageEventuKb1_ECslPD5AW6YMoR_10dioxus_web);
            return ret;
        }, arguments); },
        __wbindgen_generic_0000000000000005: function() { return logError(function (arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Ref(NamedExternref("Event"))], shim_idx: 1412, ret: Unit, inner_ret: Some(Unit) }, mutable: false }) -> Externref`.
            const ret = makeClosure(arg0, arg1, _RINvNvNvNtNtCsiXDZd1WmBpW_12wasm_bindgen7convert8closures1__1__6invokeNtNtNtCslsL5eFSWZQN_7web_sys8features9gen_Event5EventuKb1_ECslPD5AW6YMoR_10dioxus_web);
            return ret;
        }, arguments); },
        __wbindgen_generic_0000000000000006: function() { return logError(function (arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [], shim_idx: 1597, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, _RINvNvNtNtCsiXDZd1WmBpW_12wasm_bindgen7convert8closuress0_1__6invokeuKb1_ECsllE8nXdAxO6_11gloo_timers);
            return ret;
        }, arguments); },
        __wbindgen_generic_0000000000000007: function() { return logError(function (arg0) {
            // Cast intrinsic for `F64 -> Externref`.
            const ret = arg0;
            return ret;
        }, arguments); },
        __wbindgen_generic_0000000000000008: function() { return logError(function (arg0) {
            // Cast intrinsic for `I64 -> Externref`.
            const ret = arg0;
            return ret;
        }, arguments); },
        __wbindgen_generic_0000000000000009: function() { return logError(function (arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        }, arguments); },
        __wbindgen_generic_000000000000000a: function() { return logError(function (arg0) {
            // Cast intrinsic for `U64 -> Externref`.
            const ret = BigInt.asUintN(64, arg0);
            return ret;
        }, arguments); },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./freebuff-admin-ui_bg.js": import0,
        "./snippets/dioxus-interpreter-js-20451f6aefec841d/src/js/patch_console.js": import1,
    };
}


//#endregion
function _RINvNvNtNtCsiXDZd1WmBpW_12wasm_bindgen7convert8closuress0_1__6invokeuKb1_ECsllE8nXdAxO6_11gloo_timers(arg0, arg1) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._RINvNvNtNtCsiXDZd1WmBpW_12wasm_bindgen7convert8closuress0_1__6invokeuKb1_ECsllE8nXdAxO6_11gloo_timers(arg0, arg1);
}

function _RINvNvNtNtCsiXDZd1WmBpW_12wasm_bindgen7convert8closuress0_1__6invokebKb1_ECs4ySu89kE9MV_6js_sys(arg0, arg1) {
    _assertNum(arg0);
    _assertNum(arg1);
    const ret = wasm._RINvNvNtNtCsiXDZd1WmBpW_12wasm_bindgen7convert8closuress0_1__6invokebKb1_ECs4ySu89kE9MV_6js_sys(arg0, arg1);
    return ret !== 0;
}

function _RINvNvNtNtCsiXDZd1WmBpW_12wasm_bindgen7convert8closuress2_1__6invokeNtNtNtCslsL5eFSWZQN_7web_sys8features14gen_CloseEvent10CloseEventuKb1_ECslPD5AW6YMoR_10dioxus_web(arg0, arg1, arg2) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._RINvNvNtNtCsiXDZd1WmBpW_12wasm_bindgen7convert8closuress2_1__6invokeNtNtNtCslsL5eFSWZQN_7web_sys8features14gen_CloseEvent10CloseEventuKb1_ECslPD5AW6YMoR_10dioxus_web(arg0, arg1, arg2);
}

function _RINvNvNtNtCsiXDZd1WmBpW_12wasm_bindgen7convert8closuress2_1__6invokeNtNtNtCslsL5eFSWZQN_7web_sys8features9gen_Event5EventuKb1_ECslPD5AW6YMoR_10dioxus_web(arg0, arg1, arg2) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._RINvNvNtNtCsiXDZd1WmBpW_12wasm_bindgen7convert8closuress2_1__6invokeNtNtNtCslsL5eFSWZQN_7web_sys8features9gen_Event5EventuKb1_ECslPD5AW6YMoR_10dioxus_web(arg0, arg1, arg2);
}

function _RINvNvNtNtCsiXDZd1WmBpW_12wasm_bindgen7convert8closuress2_1__6invokeNtNtNtCslsL5eFSWZQN_7web_sys8features16gen_MessageEvent12MessageEventuKb1_ECslPD5AW6YMoR_10dioxus_web(arg0, arg1, arg2) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._RINvNvNtNtCsiXDZd1WmBpW_12wasm_bindgen7convert8closuress2_1__6invokeNtNtNtCslsL5eFSWZQN_7web_sys8features16gen_MessageEvent12MessageEventuKb1_ECslPD5AW6YMoR_10dioxus_web(arg0, arg1, arg2);
}

function _RINvNvNvNtNtCsiXDZd1WmBpW_12wasm_bindgen7convert8closures1__1__6invokeNtNtNtCslsL5eFSWZQN_7web_sys8features9gen_Event5EventuKb1_ECslPD5AW6YMoR_10dioxus_web(arg0, arg1, arg2) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._RINvNvNvNtNtCsiXDZd1WmBpW_12wasm_bindgen7convert8closures1__1__6invokeNtNtNtCslsL5eFSWZQN_7web_sys8features9gen_Event5EventuKb1_ECslPD5AW6YMoR_10dioxus_web(arg0, arg1, arg2);
}

function _RINvNvNtNtCsiXDZd1WmBpW_12wasm_bindgen7convert8closuress2_1__6invokeNtB8_7JsValueINtNtCsknUcikIyyBm_4core6result6ResultuNtB8_7JsErrorEKb1_ECs4ySu89kE9MV_6js_sys(arg0, arg1, arg2) {
    _assertNum(arg0);
    _assertNum(arg1);
    const ret = wasm._RINvNvNtNtCsiXDZd1WmBpW_12wasm_bindgen7convert8closuress2_1__6invokeNtB8_7JsValueINtNtCsknUcikIyyBm_4core6result6ResultuNtB8_7JsErrorEKb1_ECs4ySu89kE9MV_6js_sys(arg0, arg1, arg2);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function _RINvNvNtNtCsiXDZd1WmBpW_12wasm_bindgen7convert8closuress4_1__6invokeNtCs4ySu89kE9MV_6js_sys8FunctionB14_uKb1_EB16_(arg0, arg1, arg2, arg3) {
    _assertNum(arg0);
    _assertNum(arg1);
    wasm._RINvNvNtNtCsiXDZd1WmBpW_12wasm_bindgen7convert8closuress4_1__6invokeNtCs4ySu89kE9MV_6js_sys8FunctionB14_uKb1_EB16_(arg0, arg1, arg2, arg3);
}


const __wbindgen_enum_ScrollBehavior = ["auto", "instant", "smooth"];


const __wbindgen_enum_ScrollLogicalPosition = ["start", "center", "end", "nearest"];


const __wbindgen_enum_ScrollRestoration = ["auto", "manual"];
const JSOwnerFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_jsowner_free(ptr, 1));


//#region intrinsics
function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

function _assertBigInt(n) {
    if (typeof(n) !== 'bigint') throw new Error(`expected a bigint argument, found ${typeof(n)}`);
}

function _assertBoolean(n) {
    if (typeof(n) !== 'boolean') {
        throw new Error(`expected a boolean argument, found ${typeof(n)}`);
    }
}

function _assertNum(n) {
    if (typeof(n) !== 'number') throw new Error(`expected a number argument, found ${typeof(n)}`);
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => wasm.__wbindgen_destroy_closure(state.a, state.b));

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function getArrayJsValueFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getDataViewMemory0();
    const result = [];
    for (let i = ptr; i < ptr + 4 * len; i += 4) {
        result.push(wasm.__wbindgen_externrefs.get(mem.getUint32(i, true)));
    }
    wasm.__externref_drop_slice(ptr, len);
    return result;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function logError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        let error = (function () {
            try {
                return e instanceof Error ? `${e.message}\n\nStack:\n${e.stack}` : e.toString();
            } catch(_) {
                return "<failed to stringify thrown value>";
            }
        }());
        console.error("wasm-bindgen: imported JS function that was not marked as `catch` threw an error:", error);
        throw e;
    }
}

function makeClosure(arg0, arg1, f) {
    const state = { a: arg0, b: arg1, cnt: 1 };
    const real = (...args) => {

        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        try {
            return f(state.a, state.b, ...args);
        } finally {
            real._wbg_cb_unref();
        }
    };
    real._wbg_cb_unref = () => {
        if (--state.cnt === 0) {
            wasm.__wbindgen_destroy_closure(state.a, state.b);
            state.a = 0;
            CLOSURE_DTORS.unregister(state);
        }
    };
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function makeMutClosure(arg0, arg1, f) {
    const state = { a: arg0, b: arg1, cnt: 1 };
    const real = (...args) => {

        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            state.a = a;
            real._wbg_cb_unref();
        }
    };
    real._wbg_cb_unref = () => {
        if (--state.cnt === 0) {
            wasm.__wbindgen_destroy_closure(state.a, state.b);
            state.a = 0;
            CLOSURE_DTORS.unregister(state);
        }
    };
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function passArrayJsValueToWasm0(array, malloc) {
    const ptr = malloc(array.length * 4, 4) >>> 0;
    for (let i = 0; i < array.length; i++) {
        const add = addToExternrefTable0(array[i]);
        getDataViewMemory0().setUint32(ptr + 4 * i, add, true);
    }
    WASM_VECTOR_LEN = array.length;
    return ptr;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (typeof(arg) !== 'string') throw new Error(`expected a string argument, found ${typeof(arg)}`);
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);
        if (ret.read !== arg.length) throw new Error('failed to pass whole string');
        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;


//#endregion

//#region wasm loading
let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (!module.ok) {
            throw new Error(`failed to fetch Wasm: ${module.status} ${module.statusText} fetching '${module.url}'`);
        }

        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('freebuff-admin-ui_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
//#endregion
export { wasm as __wasm }

globalThis.__wasm_split_main_initSync = initSync;

// Actually perform the load
__wbg_init({module_or_path: "/./wasm/freebuff-admin-ui_bg.wasm"}).then((wasm) => {
    // assign this module to be accessible globally
    globalThis.__dx_mainWasm = wasm;
    globalThis.__dx_mainInit = __wbg_init;
    globalThis.__dx_mainInitSync = initSync;
    globalThis.__dx___wbg_get_imports = __wbg_get_imports;

    if (wasm.__wbindgen_start == undefined) {
        wasm.main();
    }
});

