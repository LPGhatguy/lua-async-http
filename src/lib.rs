#[macro_use] extern crate lazy_static;
extern crate lua51_sys;
extern crate libc;
extern crate reqwest;

use std::{
    ffi::{CString, CStr},
    ptr,
    thread,
    sync::{
        Mutex,
        atomic::{
            AtomicUsize,
            Ordering,
        },
    },
    time::Duration,
    collections::HashMap,
};

use libc::{
    c_int,
    c_char,
};

use lua51_sys::{
    lua_State,
    lua_pushnumber,
    lua_pushlstring,
    lua_pushboolean,
    luaL_checklstring,
    luaL_checknumber,
    luaL_openlib,
    luaL_Reg,
};

fn get_id() -> u32 {
    lazy_static! {
        static ref LAST_ID: AtomicUsize = AtomicUsize::new(0);
    }

    LAST_ID.fetch_add(1, Ordering::SeqCst) as u32
}

unsafe fn push_str(state: *mut lua_State, value: &str) {
    lua_pushlstring(state, value.as_ptr() as *const c_char, value.len());
}

#[derive(Debug)]
struct Response {
    body: String,
}

#[derive(Debug)]
enum RequestStatus {
    InFlight,
    Success(Response),
    Error(String),
}

lazy_static! {
    static ref REQWEST_CLIENT: reqwest::Client = reqwest::Client::new();
    static ref OUTSTANDING_REQUESTS: Mutex<HashMap<u32, RequestStatus>> = Mutex::new(HashMap::new());
}

#[no_mangle]
pub unsafe extern "C" fn request(state: *mut lua_State) -> c_int {
    let url_source = luaL_checklstring(state, 1, ptr::null_mut());
    let url = CStr::from_ptr(url_source).to_str().unwrap().to_string();

    println!("Requesting {}", url);

    let request = match REQWEST_CLIENT.get(&url).build() {
        Ok(request) => request,
        Err(_) => {
            lua_pushboolean(state, 0);

            push_str(state, "Invalid request parameters");

            return 2;
        },
    };

    let id = get_id();

    {
        let mut outstanding_requests = OUTSTANDING_REQUESTS.lock().unwrap();

        outstanding_requests.insert(id, RequestStatus::InFlight);
    }

    thread::spawn(move || {
        match REQWEST_CLIENT.execute(request) {
            Ok(mut http_response) => {
                let body = http_response.text().unwrap();

                let mut outstanding_requests = OUTSTANDING_REQUESTS.lock().unwrap();
                outstanding_requests.insert(id, RequestStatus::Success(Response { body }));
            },
            Err(_) => {
                let mut outstanding_requests = OUTSTANDING_REQUESTS.lock().unwrap();
                outstanding_requests.insert(id, RequestStatus::Error("HTTP error".to_string()));
            },
        }
    });

    lua_pushboolean(state, 1);
    lua_pushnumber(state, id as f64);

    2
}

#[no_mangle]
pub unsafe extern "C" fn check_request(state: *mut lua_State) -> c_int {
    let request_id = luaL_checknumber(state, 1) as u32;

    {
        let outstanding_requests = OUTSTANDING_REQUESTS.lock().unwrap();

        match outstanding_requests.get(&request_id) {
            Some(status) => {
                match status {
                    RequestStatus::InFlight => {
                        push_str(state, "in-flight");

                        return 1;
                    },
                    RequestStatus::Success(response) => {
                        push_str(state, "success");

                        let body = response.body.as_bytes();
                        lua_pushlstring(state, body.as_ptr() as *const i8, body.len());

                        return 2;
                    },
                    RequestStatus::Error(message) => {
                        push_str(state, "error");

                        push_str(state, &message);

                        return 2;
                    },
                }

            },
            None => {
                push_str(state, "error");
                push_str(state, "Unknown request ID");

                return 2;
            },
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn cleanup_request(state: *mut lua_State) -> c_int {
    let request_id = luaL_checknumber(state, 1) as u32;

    let mut outstanding_requests = OUTSTANDING_REQUESTS.lock().unwrap();

    match outstanding_requests.remove(&request_id) {
        Some(_) => lua_pushboolean(state, 1),
        None => lua_pushboolean(state, 0),
    }

    1
}

#[no_mangle]
pub unsafe extern "C" fn sleep_ms(state: *mut lua_State) -> c_int {
    let sleep_amount = luaL_checknumber(state, 1) as u64;

    thread::sleep(Duration::from_millis(sleep_amount));

    0
}

#[no_mangle]
pub extern "C" fn luaopen_async_http(state: *mut lua_State) -> c_int {
    let library_name = CString::new("async_http").unwrap();
    let request_name = CString::new("request").unwrap();
    let check_request_name = CString::new("check_request").unwrap();
    let cleanup_request_name = CString::new("cleanup_request").unwrap();
    let sleep_ms_name = CString::new("sleep_ms").unwrap();

    let registration = &[
        luaL_Reg {
            name: request_name.as_ptr(),
            func: Some(request),
        },
        luaL_Reg {
            name: check_request_name.as_ptr(),
            func: Some(check_request),
        },
        luaL_Reg {
            name: cleanup_request_name.as_ptr(),
            func: Some(cleanup_request),
        },
        luaL_Reg {
            name: sleep_ms_name.as_ptr(),
            func: Some(sleep_ms),
        },
        luaL_Reg {
            name: ptr::null(),
            func: None,
        },
    ];

    unsafe {
        luaL_openlib(state, library_name.as_ptr(), registration.as_ptr(), 0);
    }

    1
}