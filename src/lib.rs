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
    collections::HashMap,
};

use libc::{
    c_int,
    c_char,
};

use lua51_sys::{
    lua_State,
    lua_tolstring,
    lua_pushnumber,
    lua_pushlstring,
    lua_pushboolean,
    luaL_checklstring,
    luaL_checknumber,
    luaL_openlib,
    luaL_Reg,
};

lazy_static! {
    static ref LAST_ID: AtomicUsize = AtomicUsize::new(0);
}

pub fn get_id() -> u32 {
    LAST_ID.fetch_add(1, Ordering::SeqCst) as u32
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

    let id = get_id();

    {
        let mut outstanding_requests = OUTSTANDING_REQUESTS.lock().unwrap();

        outstanding_requests.insert(id, RequestStatus::InFlight);
    }

    println!("Requesting {}", url);

    thread::spawn(move || {
        match REQWEST_CLIENT.get(&url).send() {
            Ok(mut http_response) => {
                let body = http_response.text().unwrap();

                let mut outstanding_requests = OUTSTANDING_REQUESTS.lock().unwrap();
                outstanding_requests.insert(id, RequestStatus::Success(Response { body }));
            },
            Err(_) => {
                let mut outstanding_requests = OUTSTANDING_REQUESTS.lock().unwrap();
                outstanding_requests.insert(id, RequestStatus::Error("aahh".to_string()));
            }
        }
    });

    lua_pushnumber(state, id as f64);

    1
}

#[no_mangle]
pub unsafe extern "C" fn check_request(state: *mut lua_State) -> c_int {
    let request_id = luaL_checknumber(state, 1) as u32;

    {
        let outstanding_requests = OUTSTANDING_REQUESTS.lock().unwrap();

        match outstanding_requests.get(&request_id) {
            Some(status) => {
                lua_pushboolean(state, 1);

                match status {
                    RequestStatus::InFlight => {
                        lua_pushnumber(state, 0.0);

                        return 2;
                    },
                    RequestStatus::Success(response) => {
                        lua_pushnumber(state, 1.0);

                        let body = CString::new(response.body.as_bytes()).unwrap();
                        lua_pushlstring(state, body.as_ptr(), response.body.len());

                        return 3;
                    },
                    RequestStatus::Error(message) => {
                        lua_pushnumber(state, 2.0);

                        let body = CString::new(message.as_bytes()).unwrap();
                        lua_pushlstring(state, body.as_ptr(), message.len());

                        return 3;
                    },
                }

            },
            None => {
                lua_pushboolean(state, 0);
                return 1;
            },
        }
    }
}

#[no_mangle]
pub extern "C" fn luaopen_async_http(state: *mut lua_State) -> c_int {
    let library_name = CString::new("async_http").unwrap();
    let request_name = CString::new("request").unwrap();
    let check_request_name = CString::new("check_request").unwrap();

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
            name: ptr::null(),
            func: None,
        },
    ];

    unsafe {
        luaL_openlib(state, library_name.as_ptr(), registration.as_ptr(), 0);
    }

    1
}