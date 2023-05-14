//
// server.rs
// Copyright (C) 2021 gmg137 <gmg137 AT live.com>
// snap7-rs is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.
//
use crate::{ffi::*, model::*};
use anyhow::*;
use std::ffi::*;

/// S7 服务端
///
/// # Examples
/// ```
/// use snap7_rs::{AreaCode, InternalParam, InternalParamValue, S7Server, MaskKind};
/// use std::ffi::*;
/// use std::os::raw::*;
///
/// // 创建 S7 服务端
/// let server = S7Server::create();
///
/// // 创建共享内存区
/// let mut db_buff = [0u8; 1024];
///
/// // 添加共享区块
/// assert!(server
///     .register_area(AreaCode::S7AreaDB, 1, &mut db_buff)
///     .is_ok());
///
/// // 过滤读和写
/// assert!(server
///     .set_mask(MaskKind::Event, 0x00020000 | 0x00040000)
///     .is_ok());
///
/// // 设置事件回调
/// assert!(server
///     .set_events_callback(Some(move |_, p_event, _| {
///         if let Ok(text) = S7Server::event_text(p_event) {
///             println!("{:?}", text);
///         }
///     }))
///     .is_ok());
///
/// // 启动服务
/// if let Err(e) = server.start() {
///     dbg!(e);
/// }
///
/// // 处理逻辑
/// //loop {
///    // ......
/// //}
///
/// // 关闭服务
/// assert!(server.stop().is_ok());
/// ```
pub struct S7Server {
    handle: usize,
}

impl Drop for S7Server {
    fn drop(&mut self) {
        unsafe {
            Srv_Destroy(&mut self.handle as *mut S7Object);
        }
    }
}

impl Default for S7Server {
    fn default() -> Self {
        Self::create()
    }
}

impl S7Server {
    /// 创建一个 S7 服务端
    pub fn create() -> Self {
        S7Server {
            handle: unsafe { Srv_Create() },
        }
    }

    ///
    /// 读取一个服务端对象的内部参数。
    ///
    /// **输入参数:**
    ///
    ///  - param: 内部参数类型
    ///  - value: 内部参数值
    ///
    /// **返回值:**
    ///
    ///  - Ok: 设置成功
    ///  - Err: 设置失败
    ///
    pub fn get_param(&self, param: InternalParam, value: &mut InternalParamValue) -> Result<()> {
        match param {
            InternalParam::KeepAliveTime | InternalParam::RecoveryTime => unsafe {
                let mut buff = [0u8; 4];
                let res = Srv_GetParam(
                    self.handle,
                    param as c_int,
                    &mut buff as *mut [u8] as *mut c_void,
                );
                if res == 0 {
                    *value = InternalParamValue::U32(u32::from_le_bytes(buff));
                    return Ok(());
                }
                bail!("{}", Self::error_text(res))
            },
            InternalParam::LocalPort
            | InternalParam::RemotePort
            | InternalParam::DstRef
            | InternalParam::SrcTSap
            | InternalParam::SrcRef => unsafe {
                let mut buff = [0u8; 2];
                let res = Srv_GetParam(
                    self.handle,
                    param as c_int,
                    &mut buff as *mut [u8] as *mut c_void,
                );
                if res == 0 {
                    *value = InternalParamValue::U16(u16::from_le_bytes(buff));
                    return Ok(());
                }
                bail!("{}", Self::error_text(res))
            },
            _ => unsafe {
                let mut buff = [0u8; 4];
                let res = Srv_GetParam(
                    self.handle,
                    param as c_int,
                    &mut buff as *mut [u8] as *mut c_void,
                );
                if res == 0 {
                    *value = InternalParamValue::I32(i32::from_le_bytes(buff));
                    return Ok(());
                }
                bail!("{}", Self::error_text(res))
            },
        }
    }

    ///
    /// 设置服务端的内部参数。
    ///
    /// **输入参数:**
    ///
    ///  - param: 内部参数类型
    ///  - value: 内部参数值
    ///
    /// **返回值:**
    ///
    ///  - Ok: 设置成功
    ///  - Err: 设置失败
    ///
    pub fn set_param(&self, param: InternalParam, value: InternalParamValue) -> Result<()> {
        match param {
            InternalParam::KeepAliveTime | InternalParam::RecoveryTime => unsafe {
                if let InternalParamValue::U32(v) = value {
                    let mut buff = v.to_le_bytes();
                    let res = Srv_SetParam(
                        self.handle,
                        param as c_int,
                        &mut buff as *mut [u8] as *mut c_void,
                    );
                    if res == 0 {
                        return Ok(());
                    }
                    bail!("{}", Self::error_text(res))
                } else {
                    bail!("{}", Self::error_text(-1))
                }
            },
            InternalParam::LocalPort
            | InternalParam::RemotePort
            | InternalParam::DstRef
            | InternalParam::SrcTSap
            | InternalParam::SrcRef => unsafe {
                if let InternalParamValue::U16(v) = value {
                    let mut buff = v.to_le_bytes();
                    let res = Srv_SetParam(
                        self.handle,
                        param as c_int,
                        &mut buff as *mut [u8] as *mut c_void,
                    );
                    if res == 0 {
                        return Ok(());
                    }
                    bail!("{}", Self::error_text(res))
                } else {
                    bail!("{}", Self::error_text(-1))
                }
            },
            _ => unsafe {
                if let InternalParamValue::I32(v) = value {
                    let mut buff = v.to_le_bytes();
                    let res = Srv_SetParam(
                        self.handle,
                        param as c_int,
                        &mut buff as *mut [u8] as *mut c_void,
                    );
                    if res == 0 {
                        return Ok(());
                    }
                    bail!("{}", Self::error_text(res))
                } else {
                    bail!("{}", Self::error_text(-1))
                }
            },
        }
    }

    ///
    /// 启动服务端并将其绑定到指定的 IP 地址和 TCP 端口。
    ///
    /// **输入参数:**
    ///
    ///  - address: 服务器地址
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn start_to(&self, address: &str) -> Result<()> {
        let address = CString::new(address).unwrap();
        unsafe {
            let res = Srv_StartTo(self.handle, address.as_ptr());
            if res == 0 {
                return Ok(());
            }
            bail!("{}", Self::error_text(res))
        }
    }

    ///
    /// 启动服务端并将其绑定到 start_to() 中指定的 IP 地址。
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：如果 start_to() 之前未被调用，则绑定 IP 到 0.0.0.0。`
    ///
    pub fn start(&self) -> Result<()> {
        unsafe {
            let res = Srv_Start(self.handle);
            if res == 0 {
                return Ok(());
            }
            bail!("{}", Self::error_text(res))
        }
    }

    ///
    /// 停止服务端，优雅地断开所有客户端的连接，销毁所有的 S7 作业，并解除监听器套接字与地址的绑定。
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn stop(&self) -> Result<()> {
        unsafe {
            let res = Srv_Stop(self.handle);
            if res == 0 {
                return Ok(());
            }
            bail!("{}", Self::error_text(res))
        }
    }

    ///
    /// 共享一个内存区域，该内存块将被客户端看到。
    ///
    /// **输入参数:**
    ///
    ///  - area_code: 区块类型
    ///  - index: 要分享的数据块(DB)编号。如果 area_code != S7AreaDB 则被忽略，值为 0。
    ///  - buff: 要分享的内存缓冲区
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn register_area(&self, area_code: AreaCode, index: u16, buff: &mut [u8]) -> Result<()> {
        unsafe {
            let res = Srv_RegisterArea(
                self.handle,
                area_code as c_int,
                index,
                buff as *mut [u8] as *mut c_void,
                buff.len() as c_int,
            );
            if res == 0 {
                return Ok(());
            }
            bail!("{}", Self::error_text(res))
        }
    }

    ///
    /// 解除先前 register_area() 共享的内存区域，该内存块将不再被客户端看到。
    ///
    /// **输入参数:**
    ///
    ///  - area_code: 区块类型
    ///  - index: 要解除的数据块(DB)编号。如果 area_code != S7AreaDB 则被忽略，值为 0。
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn unregister_area(&self, area_code: AreaCode, index: u16) -> Result<()> {
        unsafe {
            let res = Srv_UnregisterArea(self.handle, area_code as c_int, index);
            if res == 0 {
                return Ok(());
            }
            bail!("{}", Self::error_text(res))
        }
    }

    ///
    /// 锁定一个共享内存区域。
    ///
    /// **输入参数:**
    ///
    ///  - area_code: 区块类型
    ///  - index: 要解除的数据块(DB)编号。如果 area_code != S7AreaDB 则被忽略，值为 0。
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn lock_area(&self, area_code: AreaCode, index: u16) -> Result<()> {
        unsafe {
            let res = Srv_LockArea(self.handle, area_code as c_int, index);
            if res == 0 {
                return Ok(());
            }
            bail!("{}", Self::error_text(res))
        }
    }

    ///
    /// 解锁先前锁定的共享内存区域。
    ///
    /// **输入参数:**
    ///
    ///  - area_code: 区块类型
    ///  - index: 要解除的数据块(DB)编号。如果 area_code != S7AreaDB 则被忽略，值为 0。
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn unlock_area(&self, area_code: AreaCode, index: u16) -> Result<()> {
        unsafe {
            let res = Srv_UnlockArea(self.handle, area_code as c_int, index);
            if res == 0 {
                return Ok(());
            }
            bail!("{}", Self::error_text(res))
        }
    }

    ///
    /// 设置服务器对象在创建事件时要调用的用户回调。
    ///
    /// **输入参数:**
    ///
    ///  - callback: 回调函数
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// # Examples
    /// ```ignore
    /// use std::sync::{Arc,Mutex};
    ///
    /// let num = Arc::new(Mutex::new(32));
    /// let num_clone = te.clone();
    /// server.set_events_callback(Some(move |_, p_event, _| {
    ///     let mut num = num_clone.lock().unwrap();
    ///     *num += 100;
    ///     println!("num: {}", num);
    ///     if let Some(text) = S7Server::event_text(p_event) {
    ///         println!("{:?}", text);
    ///     }
    /// })).unwrap();
    /// println!("num:{}", num.lock().unwrap());
    /// ```
    pub fn set_events_callback<F>(&self, callback: Option<F>) -> Result<()>
    where
        F: FnMut(*mut c_void, PSrvEvent, c_int) + 'static,
    {
        if callback.is_some() {
            unsafe {
                let data = Box::into_raw(Box::new(callback));
                let res = Srv_SetEventsCallback(
                    self.handle,
                    Some(call_events_closure::<F>),
                    data as *mut c_void,
                );
                if res == 0 {
                    return Ok(());
                }
                bail!("{}", Self::error_text(res))
            }
        } else {
            unsafe {
                let res =
                    Srv_SetEventsCallback(self.handle, None, std::ptr::null_mut() as *mut c_void);
                if res == 0 {
                    return Ok(());
                }
                bail!("{}", Self::error_text(res))
            }
        }
    }

    ///
    /// 设置服务端对象在客户请求读/写时要调用的用户回调。。
    ///
    /// **输入参数:**
    ///
    ///  - callback: 回调函数
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// # Examples
    /// ```ignore
    /// use std::os::raw::*;
    ///
    /// server.set_rw_area_callback(Some(
    ///     move |usr_ptr, sender, operation, ps7tag: PS7Tag, p_usr_data: *mut c_void| {
    ///         unsafe {
    ///             let pbuff = p_usr_data as *mut u8;
    ///             if operation == 0 {
    ///                 println!("读请求!");
    ///             } else {
    ///                 println!("写请求!");
    ///             }
    ///             let p7 = *ps7tag;
    ///             match p7.Area {
    ///                 0x81 => println!("Area: PE"),
    ///                 0x82 => println!("Area: PA"),
    ///                 0x83 => println!("Area: MK"),
    ///                 0x1c => println!("Area: CT"),
    ///                 0x1d => println!("Area: TM"),
    ///                 0x84 => println!("Area: DB{}", p7.DBNumber as i32),
    ///                 _ => println!("未定义的 Area"),
    ///             }
    ///             println!("Strat: {}", p7.Start as i32);
    ///             println!("Size: {}", p7.Size as i32);
    ///             if operation == 1 {
    ///                 let buff = std::slice::from_raw_parts(pbuff, p7.Size as usize);
    ///                 println!("pUsrData: {:#x?}", buff);
    ///             } else {
    ///                 //
    ///             }
    ///         }
    ///     }
    /// )).unwrap();
    /// ```
    pub fn set_rw_area_callback<F>(&self, callback: Option<F>) -> Result<()>
    where
        F: FnMut(*mut c_void, c_int, c_int, PS7Tag, *mut c_void),
    {
        if callback.is_some() {
            unsafe {
                let data = Box::into_raw(Box::new(callback));
                let res = Srv_SetRWAreaCallback(
                    self.handle,
                    Some(call_rw_area_closure::<F>),
                    data as *mut c_void,
                );
                if res == 0 {
                    return Ok(());
                }
                bail!("{}", Self::error_text(res))
            }
        } else {
            unsafe {
                let res =
                    Srv_SetRWAreaCallback(self.handle, None, std::ptr::null_mut() as *mut c_void);
                if res == 0 {
                    return Ok(());
                }
                bail!("{}", Self::error_text(res))
            }
        }
    }

    ///
    /// 设置服务端对象在创建读取事件时要调用的用户回调。
    ///
    /// **输入参数:**
    ///
    ///  - callback: 回调函数
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// # Examples
    /// ```ignore
    /// server.set_read_events_callback(Some(|ptr, p_event, size| {
    ///     println!("ptr: {:?}, size: {}", ptr, size);
    ///     if let Some(text) = S7Server::event_text(p_event) {
    ///         println!("{:?}", text);
    ///     }
    /// })).unwrap();
    /// ```
    pub fn set_read_events_callback<F>(&self, callback: Option<F>) -> Result<()>
    where
        F: FnMut(*mut c_void, PSrvEvent, c_int) + 'static,
    {
        if callback.is_some() {
            unsafe {
                let data = Box::into_raw(Box::new(callback));
                let res = Srv_SetReadEventsCallback(
                    self.handle,
                    Some(call_events_closure::<F>),
                    data as *mut c_void,
                );
                if res == 0 {
                    return Ok(());
                }
                bail!("{}", Self::error_text(res))
            }
        } else {
            unsafe {
                let res =
                    Srv_SetEventsCallback(self.handle, None, std::ptr::null_mut() as *mut c_void);
                if res == 0 {
                    return Ok(());
                }
                bail!("{}", Self::error_text(res))
            }
        }
    }

    ///
    /// 读取指定的过滤器掩码。
    ///
    /// **输入参数:**
    ///
    ///  - mask_kind: 掩码类型
    ///  - mask: 掩码值
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn get_mask(&self, mask_kind: MaskKind, mask: &mut u32) -> Result<()> {
        unsafe {
            let res = Srv_GetMask(self.handle, mask_kind as c_int, mask as *mut c_uint);
            if res == 0 {
                return Ok(());
            }
            bail!("{}", Self::error_text(res))
        }
    }

    ///
    /// 写入指定的过滤掩码。
    ///
    /// **输入参数:**
    ///
    ///  - mask_kind: 掩码类型
    ///  - mask: 掩码值
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn set_mask(&self, mask_kind: MaskKind, mask: u32) -> Result<()> {
        unsafe {
            let res = Srv_SetMask(self.handle, mask_kind as c_int, mask);
            if res == 0 {
                return Ok(());
            }
            bail!("{}", Self::error_text(res))
        }
    }

    ///
    /// 从事件队列中提取一个事件（如果有的话）。
    ///
    /// **输入参数:**
    ///
    ///  - event: 事件变量
    ///  - evt_ready: 提取是否成功，返回 1 代表提取成功，0 代表无事件。
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn pick_event(&self, event: &mut TSrvEvent, evt_ready: &mut i32) -> Result<()> {
        unsafe {
            let res = Srv_PickEvent(
                self.handle,
                event as *mut TSrvEvent,
                evt_ready as *mut c_int,
            );
            if res == 0 {
                return Ok(());
            }
            bail!("{}", Self::error_text(res))
        }
    }

    ///
    /// 清空事件队列。
    ///
    /// **返回值:**
    ///  - true: 操作成功
    ///  - false: 操作失败
    ///
    pub fn clear_events(&self) -> bool {
        unsafe { Srv_ClearEvents(self.handle) == 0 }
    }

    ///
    /// 读取服务器状态、虚拟 CPU 状态和连接的客户端数量。
    ///
    /// **输入参数:**
    ///
    ///  - server_status: 服务端状态
    ///     - 0: 服务停止
    ///     - 1: 服务运行
    ///     - 2: 服务错误
    ///  - cpu_status: CPU 状态
    ///     - 0x00: 状态未知
    ///     - 0x08: CPU Run
    ///     - 0x04: CPU Stop
    ///  - client_count: 客户端连接数
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    ///  `注：CPU 的状态可以由客户端调用相关的 S7 控制功能（冷启动/热启动/停止）或以编程方式，在服务器端调用函数 set_cpu_status() 来改变。`
    ///
    pub fn get_status(
        &self,
        server_status: &mut i32,
        cpu_status: &mut i32,
        client_count: &mut i32,
    ) -> Result<()> {
        unsafe {
            let res = Srv_GetStatus(
                self.handle,
                server_status as *mut c_int,
                cpu_status as *mut c_int,
                client_count as *mut c_int,
            );
            if res == 0 {
                return Ok(());
            }
            bail!("{}", Self::error_text(res))
        }
    }

    ///
    /// 设置虚拟 CPU 状态。
    ///
    /// **输入参数:**
    ///
    ///  - cpu_status: CPU 状态
    ///     - 0x00: 状态未知
    ///     - 0x08: CPU Run
    ///     - 0x04: CPU Stop
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn set_cpu_status(&self, cpu_status: i32) -> Result<()> {
        unsafe {
            let res = Srv_SetCpuStatus(self.handle, cpu_status as c_int);
            if res == 0 {
                return Ok(());
            }
            bail!("{}", Self::error_text(res))
        }
    }

    ///
    /// 返回一个给定错误的文本解释。
    ///
    /// **输入参数:**
    ///
    ///  - error: 错误代码
    ///
    pub fn error_text(error: i32) -> String {
        let mut chars = [0i8; 1024];
        unsafe {
            Srv_ErrorText(error, &mut chars as *mut c_char, 1024);
            CStr::from_ptr(&chars as *const c_char)
                .to_string_lossy()
                .into_owned()
        }
    }

    ///
    /// 返回一个给定事件的文本解释。
    ///
    /// **输入参数:**
    ///
    ///  - event: 事件
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn event_text(event: *mut TSrvEvent) -> Result<String> {
        let mut chars = [0i8; 1024];
        unsafe {
            let res = Srv_EventText(event, &mut chars as *mut c_char, 1024);
            if res == 0 {
                Ok(CStr::from_ptr(&chars as *const c_char)
                    .to_string_lossy()
                    .into_owned())
            } else {
                Ok("".to_owned())
            }
        }
    }
}

unsafe extern "C" fn call_events_closure<F>(usr_ptr: *mut c_void, p_event: PSrvEvent, size: c_int)
where
    F: FnMut(*mut c_void, PSrvEvent, c_int),
{
    let callback_ptr = usr_ptr as *mut F;
    let callback = &mut *callback_ptr;
    callback(usr_ptr, p_event, size);
}

unsafe extern "C" fn call_rw_area_closure<F>(
    usr_ptr: *mut c_void,
    sender: c_int,
    operation: c_int,
    p_tag: PS7Tag,
    p_usr_data: *mut c_void,
) where
    F: FnMut(*mut c_void, c_int, c_int, PS7Tag, *mut c_void),
{
    let callback_ptr = usr_ptr as *mut F;
    let callback = &mut *callback_ptr;
    callback(usr_ptr, sender, operation, p_tag, p_usr_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::result::Result::Ok;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_server() {
        let server = S7Server::create();
        let mut db_buff = [0u8; 1024];
        let mut ab_buff = [0u8; 1024];
        let mut eb_buff = [0u8; 1024];
        let mut mb_buff = [0u8; 1024];

        assert!(server
            .register_area(AreaCode::S7AreaDB, 1, &mut db_buff)
            .is_ok());
        assert!(server
            .register_area(AreaCode::S7AreaPA, 1, &mut ab_buff)
            .is_ok());
        assert!(server
            .register_area(AreaCode::S7AreaPE, 1, &mut eb_buff)
            .is_ok());
        assert!(server
            .register_area(AreaCode::S7AreaMK, 1, &mut mb_buff)
            .is_ok());
        // 过滤读和写
        assert!(server
            .set_mask(MaskKind::Event, 0x00020000 | 0x00040000)
            .is_ok());

        // assert!(server.set_rw_area_callback(Some(
        //     move |usr_ptr, sender, operation, ps7tag: PS7Tag, p_usr_data: *mut c_void| {
        //         unsafe {
        //             let pbuff = p_usr_data as *mut u8;
        //             if operation == 0 {
        //                 println!("读请求!");
        //             } else {
        //                 println!("写请求!");
        //             }
        //             let p7 = *ps7tag;
        //             match p7.Area {
        //                 0x81 => println!("Area: PE"),
        //                 0x82 => println!("Area: PA"),
        //                 0x83 => println!("Area: MK"),
        //                 0x1c => println!("Area: CT"),
        //                 0x1d => println!("Area: TM"),
        //                 0x84 => println!("Area: DB{}", p7.DBNumber as i32),
        //                 _ => println!("未定义的 Area"),
        //             }
        //             println!("Strat: {}", p7.Start as i32);
        //             println!("Size: {}", p7.Size as i32);
        //             if operation == 1 {
        //                 let buff = std::slice::from_raw_parts(p_usr_data, p7.Size as usize);
        //                 println!("pUsrData: {:#x?}", buff);
        //             } else {
        //                 *pbuff = 0x08;
        //             }
        //         }
        //     }
        // )).is_ok());
        let te = Arc::new(Mutex::new(32));
        let tee = te.clone();
        assert!(server
            .set_events_callback(Some(move |_, p_event, _| {
                let mut data = tee.lock().unwrap();
                *data += 100;
                println!("te: {}", data);
                if let Ok(text) = S7Server::event_text(p_event) {
                    println!("{:?}", text);
                }
            }))
            .is_ok());
        println!("te1:{}", te.lock().unwrap());

        assert!(server
            .set_read_events_callback(Some(|_ptr, p_event, _size| {
                if let Ok(text) = S7Server::event_text(p_event) {
                    println!("{:?}", text);
                }
            }))
            .is_ok());

        server
            .set_param(InternalParam::LocalPort, InternalParamValue::U16(7878))
            .unwrap();
        assert!(server.start().is_ok());

        let mut value = InternalParamValue::U16(0);
        assert!(server
            .get_param(InternalParam::WorkInterval, &mut value)
            .is_ok());
        println!("WorkInterval: {:?}", value);
        let mut value = InternalParamValue::I32(0);
        assert!(server
            .get_param(InternalParam::MaxClients, &mut value)
            .is_ok());
        println!("MaxClients: {:?}", value);

        let mut event = TSrvEvent::default();
        let mut ready = 0;
        assert!(server.pick_event(&mut event, &mut ready).is_ok());
        if let Ok(text) = S7Server::event_text(&mut event as *mut TSrvEvent) {
            println!("{:?}", text);
        }
        println!("{:?}", event);
        println!("{:?}", ready);
        println!("{:?}", &db_buff[0..24]);
        std::thread::sleep(std::time::Duration::from_secs(3));
        println!("{:?}", &db_buff[0..24]);

        let (mut server_status, mut cpu_status, mut client_count) = (0, 0, 0);
        assert!(server
            .get_status(&mut server_status, &mut cpu_status, &mut client_count)
            .is_ok());
        println!(
            "server_status:{}, cpu_status:{},client_count:{}",
            server_status, cpu_status, client_count
        );
        server.stop().unwrap();
    }
}
