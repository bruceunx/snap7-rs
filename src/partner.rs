//
// partner.rs
// Copyright (C) 2022 gmg137 <gmg137 AT live.com>
// Distributed under terms of the GPL-3.0-or-later license.
//
use crate::{ffi::*, model::*};
use anyhow::*;
use std::ffi::*;
use std::os::raw::*;

/// S7 伙伴
///
/// # Examples
/// 创建被动伙伴
/// ```
/// use snap7_rs::S7Partner;
/// use std::ffi::*;
/// use std::os::raw::*;
///
/// // 创建 S7 被动伙伴
/// let partner = S7Partner::create(0);
///
/// // 设置接收回调
/// partner
///     .set_recv_callback(Some(|_, op, r_id, p_data: *mut c_void, size: i32| unsafe {
///         let buff = std::slice::from_raw_parts(p_data as *const u8, size as usize);
///         println!("op: {}, r_id:{}, p_data:{:#x?}", op, r_id, buff);
///     }))
///     .unwrap();
///
/// // 启动伙伴服务
/// if let Err(e) = partner.start_to("0.0.0.0", "127.0.0.1", 0x1002, 0x1002) {
///     dbg!(e);
/// }
///
/// // 业务逻辑
/// //loop {
///     //...
/// //}
///
/// // 停止服务
/// partner.stop().unwrap();
/// ```
///
/// 创建主动伙伴
/// ```ignore
/// use snap7_rs::S7Partner;
/// use std::ffi::*;
/// use std::os::raw::*;
///
/// // 创建 S7 主动伙伴
/// let partner = S7Partner::create(1);
///
/// // 设置发送回调
/// partner
///     .set_send_callback(Some(|_, op| {
///         dbg!(S7Partner::error_text(op));
///     }))
///     .unwrap();
///
/// // 启动伙伴服务
/// if let Err(e) = partner.start_to("0.0.0.0", "127.0.0.1", 0x1002, 0x1002) {
///     dbg!(e);
/// }
///
/// let mut buff = [0x01u8, 0x02, 0x03, 0x04, 0x05, 0x06];
/// if let Err(e) = partner.b_send(1, &mut buff) {
///     dbg!(e);
/// } else {
///     dbg!("同步发送成功!");
/// }
///
/// let mut buff = [0x07u8, 0x08, 0x09, 0x0a, 0x0b, 0x0c];
/// if let Err(e) = partner.as_b_send(1, &mut buff) {
///     dbg!(e);
/// } else {
///     dbg!("异步发送...");
/// }
///
/// dbg!(S7Partner::error_text(partner.wait_as_b_send_completion(10)));
///
/// // 业务逻辑
/// //loop {
///     //...
/// //}
///
/// // 停止服务
/// partner.stop().unwrap();
/// ```
pub struct S7Partner {
    handle: usize,
}

impl Drop for S7Partner {
    fn drop(&mut self) {
        unsafe {
            Par_Destroy(&mut self.handle as *mut S7Object);
        }
    }
}

impl S7Partner {
    /// 创建一个 S7 伙伴
    ///
    /// **输入参数:**
    ///
    ///  - active: 内部参数类型
    ///     - 0: 创建一个被动(连接)伙伴
    ///     - 1: 创建一个主动(连接)伙伴
    ///
    pub fn create(active: i32) -> Self {
        S7Partner {
            handle: unsafe { Par_Create(active as c_int) },
        }
    }

    ///
    /// 读取一个伙伴对象的内部参数。
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
                let res = Par_GetParam(
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
                let res = Par_GetParam(
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
                let res = Par_GetParam(
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
    /// 设置伙伴对象的内部参数。
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
                    let res = Par_SetParam(
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
                    let res = Par_SetParam(
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
                    let res = Par_SetParam(
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
    /// 启动伙伴将其绑定到指定的 IP 地址和 TCP 端口。
    ///
    /// **输入参数:**
    ///
    ///  - local_address: 本地服务器地址
    ///  - remote_address: 远程服务器地址
    ///  - loc_tsap: 本地 TSAP
    ///  - rem_tsap: PLC TSAP
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn start_to(
        &self,
        local_address: &str,
        remote_address: &str,
        loc_tsap: u16,
        rem_tsap: u16,
    ) -> Result<()> {
        let local_address = CString::new(local_address).unwrap();
        let remote_address = CString::new(remote_address).unwrap();
        unsafe {
            let res = Par_StartTo(
                self.handle,
                local_address.as_ptr(),
                remote_address.as_ptr(),
                loc_tsap,
                rem_tsap,
            );
            if res == 0 {
                return Ok(());
            }
            bail!("{}", Self::error_text(res))
        }
    }

    ///
    /// 启动伙伴并使用之前 start_to() 中指定的参数。
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn start(&self) -> Result<()> {
        unsafe {
            let res = Par_Start(self.handle);
            if res == 0 {
                return Ok(());
            }
            bail!("{}", Self::error_text(res))
        }
    }

    ///
    /// 停止伙伴，优雅地断开所有伙伴的连接，销毁所有的 S7 作业，并解除监听器套接字与地址的绑定。
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn stop(&self) -> Result<()> {
        unsafe {
            let res = Par_Stop(self.handle);
            if res == 0 {
                return Ok(());
            }
            bail!("{}", Self::error_text(res))
        }
    }

    ///
    /// 设置用户回调，当异步数据发送完成后伙伴对象将调用该回调。
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
    /// partner.set_send_callback(Some(|_ptr, op_result| {
    ///     if op_result == 0 {
    ///         println!("发送成功!");
    ///     }else{
    ///         println!("发送失败!");
    ///     }
    /// })).unwrap();
    /// ```
    pub fn set_send_callback<F>(&self, callback: Option<F>) -> Result<()>
    where
        F: FnMut(*mut c_void, c_int) + 'static,
    {
        if callback.is_some() {
            unsafe {
                let data = Box::into_raw(Box::new(callback));
                let res = Par_SetSendCallback(
                    self.handle,
                    Some(call_send_closure::<F>),
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
                    Par_SetSendCallback(self.handle, None, std::ptr::null_mut() as *mut c_void);
                if res == 0 {
                    return Ok(());
                }
                bail!("{}", Self::error_text(res))
            }
        }
    }

    ///
    /// 设置用户回调，当有数据包时，伙伴对象将调用该回调。
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
    /// partner.set_recv_callback(Some(|_ptr, op, r_id, p_data: *mut c_void, size: i32| {
    ///     let buff = std::slice::from_raw_parts(p_data, size as usize);
    ///     println!("op: {}, r_id:{}, p_data:{:#x?}", op, r_id, buff);
    /// })).unwrap();
    /// ```
    pub fn set_recv_callback<F>(&self, callback: Option<F>) -> Result<()>
    where
        F: FnMut(*mut c_void, c_int, longword, *mut c_void, c_int) + 'static,
    {
        if callback.is_some() {
            unsafe {
                let data = Box::into_raw(Box::new(callback));
                let res = Par_SetRecvCallback(
                    self.handle,
                    Some(call_recv_closure::<F>),
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
                    Par_SetRecvCallback(self.handle, None, std::ptr::null_mut() as *mut c_void);
                if res == 0 {
                    return Ok(());
                }
                bail!("{}", Self::error_text(res))
            }
        }
    }

    ///
    /// 向伙伴发送一个数据包，这个功能是同步的，即当传输工作（send+ack）完成后它才会返回。
    ///
    /// **输入参数:**
    ///
    ///  - r_id: 路由参数，必须向b_recv 提供相同的值
    ///  - buff: 用户缓冲区
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn b_send(&self, r_id: u32, buff: &mut [u8]) -> Result<()> {
        unsafe {
            let res = Par_BSend(
                self.handle,
                r_id,
                buff as *mut [u8] as *mut c_void,
                buff.len() as i32,
            );
            if res == 0 {
                return Ok(());
            }
            bail!("{}", Self::error_text(res))
        }
    }

    ///
    /// 向伙伴发送一个数据包，这个函数是异步的，也就是说它会立即返回，需要一个检查方法来知道传输何时完成。
    ///
    /// **输入参数:**
    ///
    ///  - r_id: 路由参数，必须向b_recv 提供相同的值
    ///  - buff: 用户缓冲区
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn as_b_send(&self, r_id: u32, buff: &mut [u8]) -> Result<()> {
        unsafe {
            let res = Par_AsBSend(
                self.handle,
                r_id,
                buff as *mut [u8] as *mut c_void,
                buff.len() as i32,
            );
            if res == 0 {
                return Ok(());
            }
            bail!("{}", Self::error_text(res))
        }
    }

    ///
    /// 检查当前的异步发送任务是否完成并立即返回。
    ///
    /// **输入参数:**
    ///
    ///  - op_result: 操作结果
    ///
    /// **返回值:**
    ///  - 0: 已完成
    ///  - 1：任务进行中
    ///  - -2: 提供的处理方式无效
    ///
    ///  `注：如果返回值是 0，则 op_result 包含函数执行结果。`
    ///
    /// # Examples
    /// ```ignore
    /// // 如果不想使用循环，可以考虑使用 wait_as_b_send_completion() 函数;
    /// loop {
    ///     let mut op = -1;
    ///     if partner.check_as_b_send_completion(&mut op) == 0 {
    ///         println!("{}", op);
    ///         break;
    ///     }
    ///     std::thread::sleep(std::time::Duration::from_millis(1));
    /// }
    /// ```
    pub fn check_as_b_send_completion(&self, op_result: &mut i32) -> i32 {
        unsafe { Par_CheckAsBSendCompletion(self.handle, op_result as *mut c_int) }
    }

    ///
    /// 等待直到当前的异步发送任务完成或超时结束。
    ///
    /// **输入参数:**
    ///
    ///  - timeout: 超时，单位 ms
    ///
    /// **返回值:**
    ///  - 0: 已完成
    ///  - 0x00B00000：任务超时
    ///  - 其它值: 见错误代码
    ///
    ///  `注：这个函数使用本地操作系统原语（事件、信号...），以避免浪费CPU时间。`
    ///
    pub fn wait_as_b_send_completion(&self, timeout: u32) -> i32 {
        unsafe { Par_WaitAsBSendCompletion(self.handle, timeout) }
    }

    ///
    /// 从伙伴那里接收一个数据包，这个函数是同步的，它将一直等待，直到收到一个数据包或提供的超时过期。
    ///
    /// **输入参数:**
    ///
    ///  - r_id: 路由参数，远程伙伴 b_send 应提供相同的值
    ///  - buff: 用户缓冲区
    ///  - size: 接收数据长度
    ///  - timeout: 超时，单位 ms
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn b_recv(&self, r_id: u32, buff: &mut [u8], size: &mut i32, timeout: u32) -> Result<()> {
        unsafe {
            let res = Par_BRecv(
                self.handle,
                r_id as *mut u32,
                buff as *mut [u8] as *mut c_void,
                size as *mut c_int,
                timeout,
            );
            if res == 0 {
                return Ok(());
            }
            bail!("{}", Self::error_text(res))
        }
    }

    ///
    /// 检查是否收到数据包。
    ///
    /// **输入参数:**
    ///
    ///  - op_result: 操作结果
    ///  - r_id: 路由参数，远程伙伴 b_send 应提供相同的值
    ///  - p_data: 用户缓冲区
    ///  - size: 接收数据长度
    ///
    /// **返回值:**
    ///  - 0: 已完成
    ///  - 1：数据包处理中
    ///  - -2: 提供的处理方式无效
    ///
    ///  `注：仅当返回值是 0 时，参数结果才有意义。`
    ///
    pub fn check_as_b_recv_completion(
        &self,
        op_result: &mut i32,
        r_id: &mut u32,
        p_data: &mut [u8],
        size: &mut i32,
    ) -> i32 {
        unsafe {
            Par_CheckAsBRecvCompletion(
                self.handle,
                op_result as *mut c_int,
                r_id as *mut u32,
                p_data as *mut [u8] as *mut c_void,
                size as *mut i32,
            )
        }
    }

    ///
    /// 返回最后一次发送和接收作业的执行时间，单位为毫秒。
    ///
    /// **输入参数:**
    ///
    ///  - send_time: 发送时长
    ///  - recv_time: 接收时长
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn get_times(&self, send_time: &mut u32, recv_time: &mut u32) -> Result<()> {
        unsafe {
            let res = Par_GetTimes(self.handle, send_time as *mut u32, recv_time as *mut u32);
            if res == 0 {
                return Ok(());
            }
            bail!("{}", Self::error_text(res))
        }
    }

    ///
    /// 返回一些统计数据。
    ///
    /// **输入参数:**
    ///
    ///  - bytes_sent: 发送的字节数
    ///  - bytes_recv: 接收的字节数
    ///  - send_errors: 发送错误的数量
    ///  - recv_errors: 接收错误的数量
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn get_stats(
        &self,
        bytes_sent: &mut u32,
        bytes_recv: &mut u32,
        send_errors: &mut u32,
        recv_errors: &mut u32,
    ) -> Result<()> {
        unsafe {
            let res = Par_GetStats(
                self.handle,
                bytes_sent as *mut u32,
                bytes_recv as *mut u32,
                send_errors as *mut u32,
                recv_errors as *mut u32,
            );
            if res == 0 {
                return Ok(());
            }
            bail!("{}", Self::error_text(res))
        }
    }

    ///
    /// 返回最后的工作结果。
    ///
    /// **输入参数:**
    ///
    ///  - last_error: 最后一次工作的返回
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn get_last_error(&self, last_error: &mut i32) -> Result<()> {
        unsafe {
            let res = Par_GetLastError(self.handle, last_error as *mut i32);
            if res == 0 {
                return Ok(());
            }
            bail!("{}", Self::error_text(res))
        }
    }

    ///
    /// 返回伙伴服务状态。
    ///
    /// **输入参数:**
    ///
    ///  - status: 状态值
    ///     - 0: 已停止
    ///     - 1: 运行中，处于主动状态，正在尝试连接
    ///     - 2: 运行中，处于被动状态，等待连接
    ///     - 3: 已连接
    ///     - 4: 正在发送数据
    ///     - 5: 正在接收数据
    ///     - 6: 启动被动伙伴出错
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn get_status(&self, status: &mut i32) -> Result<()> {
        unsafe {
            let res = Par_GetStatus(self.handle, status as *mut i32);
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
            Par_ErrorText(error, &mut chars as *mut c_char, 1024);
            CStr::from_ptr(&chars as *const c_char)
                .to_string_lossy()
                .into_owned()
        }
    }
}

unsafe extern "C" fn call_send_closure<F>(usr_ptr: *mut c_void, op_result: c_int)
where
    F: FnMut(*mut c_void, c_int),
{
    let callback_ptr = usr_ptr as *mut F;
    let callback = &mut *callback_ptr;
    callback(usr_ptr, op_result);
}

unsafe extern "C" fn call_recv_closure<F>(
    usr_ptr: *mut c_void,
    op_result: c_int,
    r_id: longword,
    p_data: *mut c_void,
    size: c_int,
) where
    F: FnMut(*mut c_void, c_int, longword, *mut c_void, c_int),
{
    let callback_ptr = usr_ptr as *mut F;
    let callback = &mut *callback_ptr;
    callback(usr_ptr, op_result, r_id, p_data, size);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partner() {
        std::thread::sleep(std::time::Duration::from_secs(1));

        let partner = S7Partner::create(0);

        partner
            .set_recv_callback(Some(|_, op, r_id, p_data: *mut c_void, size: i32| unsafe {
                let buff = std::slice::from_raw_parts(p_data as *const u8, size as usize);
                println!("op: {}, r_id:{}, p_data:{:#x?}", op, r_id, buff);
            }))
            .unwrap();

        if let Err(e) = partner.start_to("0.0.0.0", "127.0.0.1", 0x1002, 0x1002) {
            dbg!(e);
            return;
        }

        std::thread::sleep(std::time::Duration::from_secs(3));

        partner.stop().unwrap();
    }

    #[test]
    fn test_active_partner() {
        // 等待伙伴启动
        std::thread::sleep(std::time::Duration::from_secs(2));

        let partner = S7Partner::create(1);
        // if let Err(e) = partner.set_param(InternalParam::RemotePort, InternalParamValue::U16(102)) {
        //     println!("{}", e);
        // }
        // let mut lp = InternalParamValue::U16(0);
        // partner
        //     .get_param(InternalParam::RemotePort, &mut lp)
        //     .unwrap();
        // println!("{:?}", lp);

        partner
            .set_send_callback(Some(|_, op| {
                println!("callback op: {}", op);
            }))
            .unwrap();

        partner
            .set_recv_callback(Some(|_, op, r_id, p_data: *mut c_void, size: i32| unsafe {
                let buff = std::slice::from_raw_parts(p_data as *const u8, size as usize);
                println!("op: {}, r_id:{}, p_data:{:#x?}", op, r_id, buff);
            }))
            .unwrap();

        partner
            .start_to("0.0.0.0", "127.0.0.1", 0x1002, 0x1002)
            .unwrap();

        let mut buff = [0x01u8, 0x02, 0x03, 0x04, 0x05, 0x06];
        if let Err(e) = partner.b_send(1, &mut buff) {
            dbg!(e);
        } else {
            dbg!("同步发送成功!");
        }

        let mut buff = [0x07u8, 0x08, 0x09, 0x0a, 0x0b, 0x0c];
        if let Err(e) = partner.as_b_send(1, &mut buff) {
            dbg!(e);
        } else {
            dbg!("异步发送...");
        }
        // let mut op = -1;
        // let res = partner.check_as_b_send_completion(&mut op);
        // if res == 0 || res == -2 {
        //     println!("{}", op);
        // }

        dbg!(S7Partner::error_text(partner.wait_as_b_send_completion(10)));

        let mut send_time = 0;
        let mut recv_time = 0;
        partner.get_times(&mut send_time, &mut recv_time).unwrap();
        dbg!(send_time, recv_time);

        let mut bytes_sent = 0;
        let mut bytes_recv = 0;
        let mut send_errors = 0;
        let mut recv_errors = 0;
        partner
            .get_stats(
                &mut bytes_sent,
                &mut bytes_recv,
                &mut send_errors,
                &mut recv_errors,
            )
            .unwrap();
        dbg!(bytes_sent, bytes_recv, send_errors, recv_errors);

        let mut status = 0;
        partner.get_status(&mut status).unwrap();
        dbg!(status);

        partner.stop().unwrap();
    }
}
