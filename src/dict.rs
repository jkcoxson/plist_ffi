// Jackson Coxson

use std::{
    ffi::{CStr, CString, c_char},
    ptr::null_mut,
};

use plist::Value;

use crate::{NodeType, PlistWrapper, plist_dict_iter, plist_t};

/// # Safety
/// Don't pass a bad plist >:(
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plist_dict_get_size(node: plist_t) -> u32 {
    if let Value::Dictionary(d) = unsafe { &mut *node }.borrow_self() {
        d.len() as u32
    } else {
        0
    }
}

/// Since the free function accepts an iterator, the root objects
/// has to be iterator compatible.
/// # Safety
/// Don't pass a bad plist >:(
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plist_dict_new_iter(_node: plist_t, iter: *mut plist_dict_iter) {
    let p = PlistWrapper::new_iterator(0).into_ptr();
    unsafe { *iter = p };
}

/// # Safety
/// Don't pass a bad plist >:(
/// Use the system allocator or else
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plist_dict_next_item(
    node: plist_t,
    iter: plist_dict_iter,
    key: *mut *mut c_char,
    item: *mut plist_t,
) {
    let wrapper = unsafe { &mut *node };
    let node = wrapper.borrow_self();

    if let Value::Dictionary(d) = node {
        let iter = unsafe { &mut *iter }.iter_next();

        if iter as usize >= d.len() {
            unsafe { *item = null_mut() };
            return;
        }
        let (p_key, p) = d.iter_mut().nth(iter as usize).unwrap();
        let p_key = p_key.to_string();
        let pc_key = CString::new(p_key.as_str()).unwrap();
        let p = PlistWrapper {
            node: NodeType::Child {
                node: p as *mut Value,
                parent: node as *mut Value,
                index: u32::MAX,
                key: Some(p_key),
            },
            children_wrappers: Vec::new(),
        }
        .into_ptr();
        wrapper.children_wrappers.push(p);
        unsafe {
            *item = p;
            *key = pc_key.into_raw();
        };
    }
}

/// # Safety
/// Don't pass a bad plist >:(
/// Use the system allocator or else
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plist_dict_get_item_key(node: plist_t, k: *mut *mut c_char) {
    let node = unsafe { &mut *node };
    match &node.node {
        NodeType::Node(_) => {}
        NodeType::Child { key, .. } => {
            if let Some(key) = key {
                let key = CString::new(key.as_str()).unwrap().into_raw();
                unsafe { *k = key };
            }
        }
        NodeType::Iterator(_) => panic!("you passed an iterator as a node"),
    };
}

/// # Safety
/// Don't pass a bad plist >:(
pub unsafe extern "C" fn plist_dict_get_item(node: plist_t, key: *const c_char) -> plist_t {
    let key = unsafe { CStr::from_ptr(key) }.to_str().unwrap();
    let wrapper = unsafe { &mut *node };
    let node = wrapper.borrow_self();
    if let Value::Dictionary(d) = node {
        if let Some(v) = d.get_mut(key) {
            let p = PlistWrapper {
                node: NodeType::Child {
                    node: v as *mut Value,
                    parent: node as *mut Value,
                    index: u32::MAX,
                    key: Some(key.to_string()),
                },
                children_wrappers: Vec::new(),
            }
            .into_ptr();
            wrapper.children_wrappers.push(p);
            return p;
        }
    }
    null_mut()
}

/// We don't have a key plist type in the plist crate
/// We'll just assume the caller knows what they're doing.
/// Blazing fast trust 🚀
/// # Safety
/// Don't pass a bad plist >:(
pub unsafe extern "C" fn plist_dict_item_get_key(node: plist_t) -> plist_t {
    let node = unsafe { &mut *node };
    match &node.node {
        NodeType::Node(_) => null_mut(),
        NodeType::Child { key, .. } => {
            if let Some(key) = key {
                let p = Value::String(key.to_string());
                let p = PlistWrapper::new_node(p);
                p.into_ptr()
            } else {
                null_mut()
            }
        }
        NodeType::Iterator(_) => panic!("you passed an iterator as a node"),
    }
}
