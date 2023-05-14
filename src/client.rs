//
// snap.rs
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

/// S7 客户端
///
/// # Examples
/// ```ignore
/// use snap7_rs::S7Client;
/// use std::ffi::*;
/// use std::os::raw::*;
///
/// // 创建 S7 客户端
/// let client = S7Client::create();
/// // 连接到 PLC
/// if client.connect_to("192.168.1.123", 0, 1).is_ok() {
///     // 创建一个数据缓冲区
///     let mut buff = [0u8; 2];
///     // 从 PLC 读取 DB1.WDB20 的值
///     if client.db_read(1, 20, 2, &mut buff).is_ok() {
///         println!("DB1.W20: {}", u16::from_be_bytes([buff[0], buff[1]]));
///     } else {
///         println!("读取 DB 失败！");
///     }
/// } else {
///     println!("连接 PLC 失败！");
/// }
/// ```
pub struct S7Client {
    handle: usize,
}

impl Drop for S7Client {
    fn drop(&mut self) {
        unsafe {
            Cli_Destroy(&mut self.handle as *mut S7Object);
        }
    }
}

impl Default for S7Client {
    fn default() -> Self {
        Self::create()
    }
}

impl S7Client {
    /// 创建 S7 客户端。
    pub fn create() -> S7Client {
        S7Client {
            handle: unsafe { Cli_Create() },
        }
    }

    ///
    /// 设置客户端连接参数。
    ///
    /// **输入参数:**
    ///
    ///  - value: 连接类型
    ///
    /// **返回值:**
    ///
    ///  - Ok: 设置成功
    ///  - Err: 设置失败
    ///
    pub fn set_connection_type(&self, value: ConnType) -> Result<()> {
        let value = match value {
            ConnType::PG => 0x01,
            ConnType::OP => 0x02,
            ConnType::S7Basic(v) => v,
        };
        unsafe {
            let res = Cli_SetConnectionType(self.handle, value);
            if res == 0 {
                return Ok(());
            }
            bail!("{}", Self::error_text(res))
        };
    }

    ///
    /// 通过指定 IP 和机架号、插槽号连接到 PLC。
    ///
    /// **输入参数:**
    ///
    ///  - address: PLC 地址
    ///  - rack: 机架号(0..7)
    ///  - slot: 插槽号(1..31)
    ///
    /// **返回值:**
    ///
    ///  - Ok: 设置成功
    ///  - Err: 设置失败
    ///
    /// **机架号和插槽号规则：**
    ///
    /// |    CPU    | 机架 | 插槽 |
    /// | --------- | ---- | ---- |
    /// |   S7 200  |   0  |   1  |
    /// |   S7 300  |   0  |   2  |
    /// |   S7 1200 |   0  | 0(1) |
    /// |   S7 1500 |   0  | 0(1) |
    ///
    /// `注：其它 CPU 按硬件配置设置`
    ///
    pub fn connect_to(&self, address: &str, rack: i32, slot: i32) -> Result<()> {
        let address = CString::new(address).unwrap();
        let res =
            unsafe { Cli_ConnectTo(self.handle, address.as_ptr(), rack as c_int, slot as c_int) };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 设置内部(IP，本地TSAP，远程TSAP)地址。
    ///
    /// **输入参数:**
    ///
    ///  - address: PLC 地址
    ///  - local_tsap: 本地 TSAP
    ///  - remote_tsap: 远程 TSAP
    ///
    /// **返回值:**
    ///
    ///  - Ok: 设置成功
    ///  - Err: 设置失败
    ///
    /// `注：此函数必须在 connect() 之前调用。`
    ///
    pub fn set_connection_params(
        &self,
        address: &str,
        local_tsap: u16,
        remote_tsap: u16,
    ) -> Result<()> {
        let address = CString::new(address).unwrap();
        let res = unsafe {
            Cli_SetConnectionParams(self.handle, address.as_ptr(), local_tsap, remote_tsap)
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 通过调用 connect_to() 或 set_connection_params() 中指定的参数，将客户端连接到PLC。
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注: 只有在调用 connect_to() 或 set_connection_params() 后才能调用该函数。`
    ///
    pub fn connect(&self) -> Result<()> {
        let res = unsafe { Cli_Connect(self.handle) };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// "优雅地"从 PLC 上断开客户端的连接。
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注: 如果客户端参数是一个有效的句柄，这个函数总是返回 true，它可以被安全地多次调用。这个函数在 S7Client drop 时也会被调用。`
    ///
    pub fn disconnect(&self) -> Result<()> {
        let res = unsafe { Cli_Disconnect(self.handle) };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 读取客户端的内部参数。
    ///
    /// **输入参数:**
    ///
    ///  - param: 内部参数类型
    ///  - value: 参数值
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn get_param(&self, param: InternalParam, value: &mut InternalParamValue) -> Result<()> {
        match param {
            InternalParam::KeepAliveTime | InternalParam::RecoveryTime => unsafe {
                let mut buff = [0u8; 4];
                let res = Cli_GetParam(
                    self.handle,
                    param as c_int,
                    &mut buff as *mut [u8] as *mut c_void,
                );
                if res == 0 {
                    *value = InternalParamValue::U32(u32::from_le_bytes(buff));
                    Ok(())
                } else {
                    bail!("{}", Self::error_text(res))
                }
            },
            InternalParam::LocalPort
            | InternalParam::RemotePort
            | InternalParam::DstRef
            | InternalParam::SrcTSap
            | InternalParam::SrcRef => unsafe {
                let mut buff = [0u8; 2];
                let res = Cli_GetParam(
                    self.handle,
                    param as c_int,
                    &mut buff as *mut [u8] as *mut c_void,
                );
                if res == 0 {
                    *value = InternalParamValue::U16(u16::from_le_bytes(buff));
                    Ok(())
                } else {
                    bail!("{}", Self::error_text(res))
                }
            },
            _ => unsafe {
                let mut buff = [0u8; 4];
                let res = Cli_GetParam(
                    self.handle,
                    param as c_int,
                    &mut buff as *mut [u8] as *mut c_void,
                );
                if res == 0 {
                    *value = InternalParamValue::I32(i32::from_le_bytes(buff));
                    Ok(())
                } else {
                    bail!("{}", Self::error_text(res))
                }
            },
        }
    }

    ///
    /// 设置客户端的内部参数。
    ///
    /// **输入参数:**
    ///
    ///  - param: 内部参数类型
    ///  - value: 内部参数值
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn set_param(&self, param: InternalParam, value: InternalParamValue) -> Result<()> {
        match param {
            InternalParam::KeepAliveTime | InternalParam::RecoveryTime => unsafe {
                if let InternalParamValue::U32(v) = value {
                    let mut buff = v.to_le_bytes();
                    let res = Cli_SetParam(
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
                    let res = Cli_SetParam(
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
                    let res = Cli_SetParam(
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
    /// 从 PLC 中读取数据, 你可以读取数据块(DB)、输入、输出、内部标志位(Merkers)、定时器和计数器。
    ///
    /// **输入参数:**
    ///
    ///  - area: 要读取的区域
    ///  - db_number: 要读取的数据块(DB)编号。如果区域不为 S7AreaDB 则被忽略，值为 0。
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待读取数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：
    /// (1) 如果你需要传输一个大的数据，你可以考虑使用异步的 as_read_area()。
    /// (2) 当 word_len=S7WLBit 时，Offset(start) 必须以比特表示。
    /// 示例: DB4.DBX 10.3 的起点是 (10*8)+3=83`
    ///
    pub fn read_area(
        &self,
        area: AreaTable,
        db_number: i32,
        start: i32,
        size: i32,
        word_len: WordLenTable,
        buff: &mut [u8],
    ) -> Result<()> {
        let res = unsafe {
            Cli_ReadArea(
                self.handle,
                area as c_int,
                db_number as c_int,
                start as c_int,
                size as c_int,
                word_len as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 将数据写入到 PLC, 这是 read_area() 的补充函数。
    ///
    /// **输入参数:**
    ///
    ///  - area: 要读取的区域
    ///  - db_number: 要读取的数据块(DB)编号。如果区域不为 S7AreaDB 则被忽略，值为 0。
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待写入数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：
    /// (1) 如果你需要传输一个大的数据，你可以考虑使用异步的 as_write_area()。
    /// (2) 当 word_len=S7WLBit 时，Offset(start) 必须以比特表示。
    /// 示例: DB4.DBX 10.3 的起点是 (10*8)+3=83`
    ///
    pub fn write_area(
        &self,
        area: AreaTable,
        db_number: i32,
        start: i32,
        size: i32,
        word_len: WordLenTable,
        buff: &mut [u8],
    ) -> Result<()> {
        let res = unsafe {
            Cli_WriteArea(
                self.handle,
                area as c_int,
                db_number as c_int,
                start as c_int,
                size as c_int,
                word_len as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 从 PLC DB 区读取数据。
    ///
    /// ```text
    /// 这是 read_area() 的一个精简函数，它从内部调用了 read_area(), 其内容为:
    ///     area = S7AreaDB.
    ///     word_len = S7WLBytes.
    /// ```
    ///
    /// **输入参数:**
    ///
    ///  - db_number: 要读取的数据块(DB)编号
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待写入数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：如果你需要传输一个大的数据，你可以考虑使用异步的 as_db_read()。`
    ///
    pub fn db_read(&self, db_number: i32, start: i32, size: i32, buff: &mut [u8]) -> Result<()> {
        let res = unsafe {
            Cli_DBRead(
                self.handle,
                db_number as c_int,
                start as c_int,
                size as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 向 PLC DB 区写入数据。
    ///
    /// ```markdown
    /// 这是 write_area() 的一个精简函数，它从内部调用了 write_area(), 其内容为:
    ///     area = S7AreaDB.
    ///     word_len = S7WLBytes.
    /// ```
    ///
    /// **输入参数:**
    ///
    ///  - db_number: 要读取的数据块(DB)编号
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待写入数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：如果你需要传输一个大的数据，你可以考虑使用异步的 as_db_write()。`
    ///
    pub fn db_write(&self, db_number: i32, start: i32, size: i32, buff: &mut [u8]) -> Result<()> {
        let res = unsafe {
            Cli_DBWrite(
                self.handle,
                db_number as c_int,
                start as c_int,
                size as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 从 PLC 输出区读取数据。
    ///
    /// ```text
    /// 这是 read_area() 的一个精简函数，它从内部调用了 read_area(), 其内容为:
    ///     area = S7AreaPA.
    ///     word_len = S7WLBytes.
    /// ```
    ///
    /// **输入参数:**
    ///
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待写入数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：如果你需要传输一个大的数据，你可以考虑使用异步的 as_ab_read()。`
    ///
    pub fn ab_read(&self, start: i32, size: i32, buff: &mut [u8]) -> Result<()> {
        let res = unsafe {
            Cli_ABRead(
                self.handle,
                start as c_int,
                size as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 向 PLC 输出区写入数据。
    ///
    /// ```markdown
    /// 这是 write_area() 的一个精简函数，它从内部调用了 write_area(), 其内容为:
    ///     area = S7AreaPA.
    ///     word_len = S7WLBytes.
    /// ```
    ///
    /// **输入参数:**
    ///
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待写入数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：如果你需要传输一个大的数据，你可以考虑使用异步的 as_ab_write()。`
    ///
    pub fn ab_write(&self, start: i32, size: i32, buff: &mut [u8]) -> Result<()> {
        let res = unsafe {
            Cli_ABWrite(
                self.handle,
                start as c_int,
                size as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 从 PLC 输入区读取数据。
    ///
    /// ```text
    /// 这是 read_area() 的一个精简函数，它从内部调用了 read_area(), 其内容为:
    ///     area = S7AreaPE.
    ///     word_len = S7WLBytes.
    /// ```
    ///
    /// **输入参数:**
    ///
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待写入数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：如果你需要传输一个大的数据，你可以考虑使用异步的 as_eb_read()。`
    ///
    pub fn eb_read(&self, start: i32, size: i32, buff: &mut [u8]) -> Result<()> {
        let res = unsafe {
            Cli_EBRead(
                self.handle,
                start as c_int,
                size as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 向 PLC 输入区写入数据。
    ///
    /// ```markdown
    /// 这是 write_area() 的一个精简函数，它从内部调用了 write_area(), 其内容为:
    ///     area = S7AreaPE.
    ///     word_len = S7WLBytes.
    /// ```
    ///
    /// **输入参数:**
    ///
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待写入数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：如果你需要传输一个大的数据，你可以考虑使用异步的 as_eb_write()。`
    ///
    pub fn eb_write(&self, start: i32, size: i32, buff: &mut [u8]) -> Result<()> {
        let res = unsafe {
            Cli_EBWrite(
                self.handle,
                start as c_int,
                size as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 从 PLC 内部标志位(Merkers)读取数据。
    ///
    /// ```text
    /// 这是 read_area() 的一个精简函数，它从内部调用了 read_area(), 其内容为:
    ///     area = S7AreaMK.
    ///     word_len = S7WLBytes.
    /// ```
    ///
    /// **输入参数:**
    ///
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待写入数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：如果你需要传输一个大的数据，你可以考虑使用异步的 as_mb_read()。`
    ///
    pub fn mb_read(&self, start: i32, size: i32, buff: &mut [u8]) -> Result<()> {
        let res = unsafe {
            Cli_MBRead(
                self.handle,
                start as c_int,
                size as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 向 PLC 内部标志位(Merkers)写入数据。
    ///
    /// ```markdown
    /// 这是 write_area() 的一个精简函数，它从内部调用了 write_area(), 其内容为:
    ///     area = S7AreaMK.
    ///     word_len = S7WLBytes.
    /// ```
    ///
    /// **输入参数:**
    ///
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待写入数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：如果你需要传输一个大的数据，你可以考虑使用异步的 as_mb_write()。`
    ///
    pub fn mb_write(&self, start: i32, size: i32, buff: &mut [u8]) -> Result<()> {
        let res = unsafe {
            Cli_MBWrite(
                self.handle,
                start as c_int,
                size as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 读取 PLC 定时器数据。
    ///
    /// ```text
    /// 这是 read_area() 的一个精简函数，它从内部调用了 read_area(), 其内容为:
    ///     area = S7AreaTM.
    ///     word_len = S7WLTimer.
    /// ```
    ///
    /// **输入参数:**
    ///
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待写入数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：如果你需要传输一个大的数据，你可以考虑使用异步的 as_tm_read()。`
    /// `    缓冲区大小 = size * 2`
    ///
    pub fn tm_read(&self, start: i32, size: i32, buff: &mut [u8]) -> Result<()> {
        let res = unsafe {
            Cli_TMRead(
                self.handle,
                start as c_int,
                size as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 向 PLC 定时器写入数据。
    ///
    /// ```markdown
    /// 这是 write_area() 的一个精简函数，它从内部调用了 write_area(), 其内容为:
    ///     area = S7AreaTM.
    ///     word_len = S7WLTimer.
    /// ```
    ///
    /// **输入参数:**
    ///
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待写入数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：如果你需要传输一个大的数据，你可以考虑使用异步的 as_tm_write()。`
    /// `    缓冲区大小 = size * 2`
    ///
    pub fn tm_write(&self, start: i32, size: i32, buff: &mut [u8]) -> Result<()> {
        let res = unsafe {
            Cli_TMWrite(
                self.handle,
                start as c_int,
                size as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 读取 PLC 计数器数据。
    ///
    /// ```text
    /// 这是 read_area() 的一个精简函数，它从内部调用了 read_area(), 其内容为:
    ///     area = S7AreaCT.
    ///     word_len = S7WLCounter.
    /// ```
    ///
    /// **输入参数:**
    ///
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待写入数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：如果你需要传输一个大的数据，你可以考虑使用异步的 as_ct_read()。`
    /// `    缓冲区大小 = size * 2`
    ///
    pub fn ct_read(&self, start: i32, size: i32, buff: &mut [u8]) -> Result<()> {
        let res = unsafe {
            Cli_CTRead(
                self.handle,
                start as c_int,
                size as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 向 PLC 计数器写入数据。
    ///
    /// ```markdown
    /// 这是 write_area() 的一个精简函数，它从内部调用了 write_area(), 其内容为:
    ///     area = S7AreaCT.
    ///     word_len = S7WLCounter.
    /// ```
    ///
    /// **输入参数:**
    ///
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待写入数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：如果你需要传输一个大的数据，你可以考虑使用异步的 as_ct_write()。`
    /// `    缓冲区大小 = size * 2`
    ///
    pub fn ct_write(&self, start: i32, size: i32, buff: &mut [u8]) -> Result<()> {
        let res = unsafe {
            Cli_CTWrite(
                self.handle,
                start as c_int,
                size as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 在一次调用中从 PLC 读取不同区域的数据。
    ///
    /// **输入参数:**
    ///
    ///  - item: TS7DataItem 数组
    ///  - items_count: 要读取的区域数量
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：由于涉及到不同区域的变量，这个函数没有分割功能，所以最大数据量不能超过PDU的大小。`
    /// `因此，这个函数没有对应的异步函数。当你有许多非连续的小变量需要读取时，这个函数的优势就会变得很大。`
    ///
    /// # Examples
    /// ```ignore
    /// use std::os::raw::*;
    ///
    /// let mut db1 = [0u8; 2];
    /// let mut in1 = [0u8; 1];
    /// let item0 = TS7DataItem {
    ///     Area: AreaTable::S7AreaDB as c_int,
    ///     WordLen: WordLenTable::S7WLByte as c_int,
    ///     Result: 0,
    ///     DBNumber: 1,
    ///     Start: 0,
    ///     Amount: 2,
    ///     pdata: &mut db1 as *mut [u8] as *mut c_void,
    /// };
    /// let item1 = TS7DataItem {
    ///     Area: AreaTable::S7AreaPA as c_int,
    ///     WordLen: WordLenTable::S7WLBit as c_int,
    ///     Result: 0,
    ///     DBNumber: 0,
    ///     Start: 0,
    ///     Amount: 1,
    ///     pdata: &mut in1 as *mut [u8] as *mut c_void,
    /// };
    /// let mut item = [item0, item1];
    /// client.read_multi_vars(&mut item, 2);
    /// ```
    pub fn read_multi_vars(&self, item: &mut [TS7DataItem], items_count: i32) -> Result<()> {
        let res = unsafe {
            Cli_ReadMultiVars(
                self.handle,
                &mut item[0] as *mut TS7DataItem,
                items_count as c_int,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 在一次调用中向 PLC 的不同区域写入数据。
    ///
    /// **输入参数:**
    ///
    ///  - item: TS7DataItem 数组
    ///  - items_count: 要写入的区域数量
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn write_multi_vars(&self, item: &mut [TS7DataItem], items_count: i32) -> Result<()> {
        let res = unsafe {
            Cli_WriteMultiVars(
                self.handle,
                &mut item[0] as *mut TS7DataItem,
                items_count as c_int,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 该函数返回按类型划分的 AG 块数量。
    ///
    /// **输入参数:**
    ///
    ///  - ts7_blocks_list: TS7BlocksList 结构体
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn list_blocks(&self, ts7_blocks_list: &mut TS7BlocksList) -> Result<()> {
        let res = unsafe { Cli_ListBlocks(self.handle, ts7_blocks_list as *mut TS7BlocksList) };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 该函数返回指定区块类型的 AG 列表。
    ///
    /// **输入参数:**
    ///
    ///  - block_type: 要获取的区块类型
    ///  - buff: 待写入数据缓冲区
    ///  - items_count: 在输入中表示用户缓冲区的容量，在输出中表示找到了多少个项目
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// # Examples
    /// ```ignore
    ///    let mut buff: TS7BlocksOfType = [0; 8192];
    ///    let mut items_count = buff.len() as i32;
    ///    client.list_blocks_of_type(BlockType::BlockDB, &mut buff, &mut items_count);
    /// ```
    ///
    pub fn list_blocks_of_type(
        &self,
        block_type: BlockType,
        buff: &mut TS7BlocksOfType,
        items_count: &mut i32,
    ) -> Result<()> {
        let res = unsafe {
            Cli_ListBlocksOfType(
                self.handle,
                block_type as c_int,
                buff as *mut TS7BlocksOfType,
                items_count as *mut c_int,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 返回一个给定 AG 块的详细信息。如果你需要在一个事先不知道大小的 DB 中读或写数据，这个函数就非常有用（见 MC7Size 字段）。
    ///
    /// **输入参数:**
    ///
    ///  - block_type: 要获取的区块类型
    ///  - block_num: 要获取的区块数量
    ///  - ts7_block_info: TS7BlockInfo 结构体
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn get_ag_block_info(
        &self,
        block_type: BlockType,
        block_num: i32,
        ts7_block_info: &mut TS7BlockInfo,
    ) -> Result<()> {
        let res = unsafe {
            Cli_GetAgBlockInfo(
                self.handle,
                block_type as c_int,
                block_num as c_int,
                ts7_block_info as *mut TS7BlockInfo,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 返回一个区块的详细信息到用户缓冲区中。这个函数通常与 full_upload() 一起使用。
    ///
    /// **输入参数:**
    ///
    ///  - buff: 用户缓冲区
    ///  - ts7_block_info: TS7BlockInfo 结构体
    ///  - size: 缓冲区大小(字节)
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn get_pg_block_info(
        &self,
        buff: &mut [u8],
        ts7_block_info: &mut TS7BlockInfo,
        size: i32,
    ) -> Result<()> {
        let res = unsafe {
            Cli_GetPgBlockInfo(
                self.handle,
                buff as *mut [u8] as *mut c_void,
                ts7_block_info as *mut TS7BlockInfo,
                size as c_int,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 从 AG 上传一个区块。将整个区块复制到用户缓冲区。
    ///
    /// **输入参数:**
    ///
    ///  - block_type: 要获取的区块类型
    ///  - block_num: 要获取的区块号
    ///  - buff: 用户缓冲区
    ///  - size: 在输入中表示缓冲区大小，在输出中表示上传的字节数
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// # Examples
    /// ```ignore
    ///    let mut buff = [0; 4096];
    ///    let mut size = buff.len() as i32;
    ///    client.full_upload(BlockType::BlockSDB, 0, &mut buff, &mut size);
    /// ```
    ///
    pub fn full_upload(
        &self,
        block_type: BlockType,
        block_num: i32,
        buff: &mut [u8],
        size: &mut i32,
    ) -> Result<()> {
        let res = unsafe {
            Cli_FullUpload(
                self.handle,
                block_type as c_int,
                block_num as c_int,
                buff as *mut [u8] as *mut c_void,
                size as *mut c_int,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 从 AG 上传一个区块主体。只将区块主体复制到用户缓冲区。
    ///
    /// **输入参数:**
    ///
    ///  - block_type: 要获取的区块类型
    ///  - block_num: 要获取的区块号
    ///  - buff: 用户缓冲区
    ///  - size: 在输入中表示缓冲区大小，在输出中表示上传的字节数
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// # Examples
    /// ```ignore
    ///    let mut buff = [0; 4096];
    ///    let mut size = buff.len() as i32;
    ///    client.upload(BlockType::BlockSDB, 0, &mut buff, &mut size);
    /// ```
    ///
    pub fn upload(
        &self,
        block_type: BlockType,
        block_num: i32,
        buff: &mut [u8],
        size: &mut i32,
    ) -> Result<()> {
        let res = unsafe {
            Cli_Upload(
                self.handle,
                block_type as c_int,
                block_num as c_int,
                buff as *mut [u8] as *mut c_void,
                size as *mut c_int,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 从 AG 下载一个区块。将用户缓冲区复制到整个区块。
    ///
    /// **输入参数:**
    ///
    ///  - block_num: 新区块编号，或 -1
    ///  - buff: 用户缓冲区
    ///  - size: 缓冲区大小
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注:一个准备被下载的区块已经包含了关于区块类型和区块编号的信息。 如果参数 block_num 为 -1，则区块编号不会被改变，否则区块将以设置的编号被下载。`
    ///
    pub fn download(&self, block_num: i32, buff: &mut [u8], size: i32) -> Result<()> {
        let res = unsafe {
            Cli_Download(
                self.handle,
                block_num as c_int,
                buff as *mut [u8] as *mut c_void,
                size as c_int,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 从 AG 删除一个区块。
    ///
    /// **警告: 一旦执行无法撤销！！！**
    ///
    /// **输入参数:**
    ///
    ///  - block_type: 要删除的区块类型
    ///  - block_num: 要删除的区块编号
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn delete(&self, block_type: BlockType, block_num: i32) -> Result<()> {
        let res = unsafe { Cli_Delete(self.handle, block_type as c_int, block_num as c_int) };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 从 AG 上传一个 DB，这个函数等同于 upload() 的参数 block_type = Block_DB，
    /// 但是它使用了一个不同的方法，所以它不受安全级别设置的限制。这个方法只上传数据。
    ///
    /// **输入参数:**
    ///
    ///  - block_num: 要上传的 DB 块编号
    ///  - buff: 用户缓冲区
    ///  - size: 在输入中表示缓冲区大小，在输出中表示上传的字节数
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn db_get(&self, block_num: i32, buff: &mut [u8], size: &mut i32) -> Result<()> {
        let res = unsafe {
            Cli_DBGet(
                self.handle,
                block_num as c_int,
                buff as *mut [u8] as *mut c_void,
                size as *mut c_int,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 用一个给定的字节填充 AG 中的一个 DB，而不需要指定其大小。
    ///
    /// **输入参数:**
    ///
    ///  - block_num: 要填充的 DB 块编号
    ///  - fill_char: 要填充的字节
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    ///  `注：出于效率考虑，fill_char 是一个整数，且只有最低的字节被使用`
    ///
    pub fn db_fill(&self, block_num: i32, fill_char: i32) -> Result<()> {
        let res = unsafe { Cli_DBFill(self.handle, block_num as c_int, fill_char as c_int) };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 读取 PLC 的日期和时间。
    ///
    /// **输入参数:**
    ///
    ///  - date_time: DateTime 结构体
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn get_plc_date_time(&self, date_time: &mut DateTime) -> Result<()> {
        let res = unsafe { Cli_GetPlcDateTime(self.handle, date_time as *mut DateTime) };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 设置 PLC 的日期和时间。
    ///
    /// **输入参数:**
    ///
    ///  - date_time: DateTime 结构体
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn set_plc_date_time(&self, date_time: &mut DateTime) -> Result<()> {
        let res = unsafe { Cli_SetPlcDateTime(self.handle, date_time as *mut DateTime) };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 设置 PLC 的日期和时间与 PC 一致。
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn set_plc_system_date_time(&self) -> Result<()> {
        let res = unsafe { Cli_SetPlcSystemDateTime(self.handle) };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 通过一个给定 ID 和 INDEX 读取局部系统状态列表。
    ///
    /// **输入参数:**
    ///
    /// - id: 列表 ID
    /// - index: 列表 INDEX
    /// - ts7szl: TS7SZL 结构体
    /// - size: 输入时为缓冲区大小，输出时为读取到的字节数
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn read_szl(&self, id: i32, index: i32, ts7szl: &mut TS7SZL, size: &mut i32) -> Result<()> {
        let res = unsafe {
            Cli_ReadSZL(
                self.handle,
                id,
                index,
                ts7szl as *mut TS7SZL,
                size as *mut c_int,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 读取局部系统状态列表的目录。
    ///
    /// **输入参数:**
    ///
    /// - ts7szl_list: TS7SZLList 结构体
    /// - items_count: 输入时为缓冲区大小，输出时为发现的项目数量
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn read_szl_list(&self, ts7szl_list: &mut TS7SZLList, items_count: &mut i32) -> Result<()> {
        let res = unsafe {
            Cli_ReadSZLList(
                self.handle,
                ts7szl_list as *mut TS7SZLList,
                items_count as *mut c_int,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 获取 CPU 商品码和版本信息。
    ///
    /// **输入参数:**
    ///
    /// - ts7_order_code: TS7OrderCode 结构体
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn get_order_code(&self, ts7_order_code: &mut TS7OrderCode) -> Result<()> {
        let res = unsafe { Cli_GetOrderCode(self.handle, ts7_order_code as *mut TS7OrderCode) };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 获取 CPU 模块名称、序列号和其他信息。
    ///
    /// **输入参数:**
    ///
    /// - ts7_cpu_info: TS7CpuInfo 结构体
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn get_cpu_info(&self, ts7_cpu_info: &mut TS7CpuInfo) -> Result<()> {
        let res = unsafe { Cli_GetCpuInfo(self.handle, ts7_cpu_info as *mut TS7CpuInfo) };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 获取 CP（通信处理器）信息。
    ///
    /// **输入参数:**
    ///
    /// - ts7_cp_info: TS7CpInfo 结构体
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn get_cp_info(&self, ts7_cp_info: &mut TS7CpInfo) -> Result<()> {
        let res = unsafe { Cli_GetCpInfo(self.handle, ts7_cp_info as *mut TS7CpInfo) };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 将 CPU 置于 RUN 模式，执行热启动。
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    ///  `注：该功能受制于设定的安全级别。`
    ///
    pub fn plc_hot_start(&self) -> Result<()> {
        let res = unsafe { Cli_PlcHotStart(self.handle) };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 将 CPU 置于 RUN 模式，执行冷启动。
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    ///  `注：该功能受制于设定的安全级别。`
    ///
    pub fn plc_cold_start(&self) -> Result<()> {
        let res = unsafe { Cli_PlcColdStart(self.handle) };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 将 CPU 置于 STOP 模式。
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    ///  `注：该功能受制于设定的安全级别。`
    ///
    pub fn plc_stop(&self) -> Result<()> {
        let res = unsafe { Cli_PlcStop(self.handle) };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 执行复制 RAM 到 ROM。
    ///
    /// **输入参数:**
    ///
    /// - timeout: 预期完成操作的最大时间(ms)
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    ///  `注：不是所有的 CPU 都支持这个操作，CPU 必须处于 STOP 模式。`
    ///
    pub fn copy_ram_to_rom(&self, timeout: i32) -> Result<()> {
        let res = unsafe { Cli_CopyRamToRom(self.handle, timeout) };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 执行内存压缩。
    ///
    /// **输入参数:**
    ///
    /// - timeout: 预期完成操作的最大时间(ms)
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    ///  `注：不是所有的 CPU 都支持这个操作，CPU 必须处于 STOP 模式。`
    ///
    pub fn compress(&self, timeout: i32) -> Result<()> {
        let res = unsafe { Cli_Compress(self.handle, timeout) };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 获取 PLC 状态。
    ///
    /// **输入参数**
    ///
    ///  - status: PLC 状态
    ///     - 0x00: 状态未知
    ///     - 0x08：运行
    ///     - 0x04：停止
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn get_plc_status(&self, status: &mut i32) -> Result<()> {
        let res = unsafe { Cli_GetPlcStatus(self.handle, status as *mut c_int) };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 向 PLC 发送密码，以满足其安全要求。
    ///
    /// **输入参数**
    ///
    ///  - password: 密码
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn set_session_password(&self, password: &str) -> Result<()> {
        let password = CString::new(password).unwrap();
        let res = unsafe { Cli_SetSessionPassword(self.handle, password.into_raw()) };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 清除为当前会话设置的密码（注销）。
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn clear_session_password(&self) -> Result<()> {
        let res = unsafe { Cli_ClearSessionPassword(self.handle) };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 获取 CPU 安全级别信息。
    ///
    /// **输入参数**
    ///
    ///  - ts7_protection: TS7Protection 结构体
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn get_protection(&self, ts7_protection: &mut TS7Protection) -> Result<()> {
        let res = unsafe { Cli_GetProtection(self.handle, ts7_protection as *mut TS7Protection) };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 与 CPU 交换一个给定的 S7 PDU（协议数据单元）。
    ///
    /// **输入参数**
    ///
    ///  - buff: 用户缓冲区
    ///  - size: 输入时为用户缓冲区大小，输出时为回复报文大小
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn iso_exchange_buffer(&self, buff: &mut [u8], size: &mut i32) -> Result<()> {
        let res = unsafe {
            Cli_IsoExchangeBuffer(
                self.handle,
                buff as *mut [u8] as *mut c_void,
                size as *mut c_int,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 返回最后的作业执行时间，单位是毫秒。
    ///
    /// **输入参数**
    ///
    ///  - time: 执行时间(ms)
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn get_exec_time(&self, time: &mut i32) -> Result<()> {
        let res = unsafe { Cli_GetExecTime(self.handle, time as *mut c_int) };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 返回最后的工作结果。
    ///
    /// **输入参数:**
    ///
    ///  - last_error: 最后一次工作的返回结果
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn get_last_error(&self, last_error: &mut i32) -> Result<()> {
        unsafe {
            let res = Cli_GetLastError(self.handle, last_error as *mut i32);
            if res == 0 {
                return Ok(());
            }
            bail!("{}", Self::error_text(res))
        }
    }

    ///
    /// 返回有关 PDU 长度的信息。
    ///
    /// **输入参数:**
    ///
    ///  - requested: 要求的 PDU 长度
    ///  - negotiated: 协商的 PDU 长度
    ///
    /// **返回值:**
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn get_pdu_length(&self, requested: &mut i32, negotiated: &mut i32) -> Result<()> {
        unsafe {
            let res = Cli_GetPduLength(
                self.handle,
                requested as *mut c_int,
                negotiated as *mut c_int,
            );
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
            Cli_ErrorText(error, &mut chars as *mut c_char, 1024);
            CStr::from_ptr(&chars as *const c_char)
                .to_string_lossy()
                .into_owned()
        }
    }

    ///
    /// 获取连接状态。
    ///
    /// **输入参数**
    ///
    ///  - is_connected: 0 未连接，!=0 已连接
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn get_connected(&self, is_connected: &mut i32) -> Result<()> {
        let res = unsafe { Cli_GetConnected(self.handle, is_connected as *mut c_int) };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 设置客户端在异步数据传输完成时的用户回调。。
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
    /// client.set_as_callback(Some(|_, op_code, op_result| {
    ///     println!("op_code: {}", op_code);
    ///     println!("op_result: {:?}", S7Client::error_text(op_result));
    /// })).unwrap();
    /// ```
    pub fn set_as_callback<F>(&self, callback: Option<F>) -> Result<()>
    where
        F: FnMut(*mut c_void, c_int, c_int) + 'static,
    {
        if callback.is_some() {
            unsafe {
                let data = Box::into_raw(Box::new(callback));
                let res =
                    Cli_SetAsCallback(self.handle, Some(call_as_closure::<F>), data as *mut c_void);
                if res == 0 {
                    return Ok(());
                }
                bail!("{}", Self::error_text(res))
            }
        } else {
            unsafe {
                let res = Cli_SetAsCallback(self.handle, None, std::ptr::null_mut() as *mut c_void);
                if res == 0 {
                    return Ok(());
                }
                bail!("{}", Self::error_text(res))
            }
        }
    }

    ///
    /// 检查当前的异步任务是否完成并立即返回。
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
    /// // 如果不想使用循环，可以考虑使用 wait_as_completion() 函数;
    /// loop {
    ///     let mut op = -1;
    ///     if partner.check_as_completion(&mut op) == 0 {
    ///         println!("{}", op);
    ///         break;
    ///     }
    ///     std::thread::sleep(std::time::Duration::from_millis(100));
    /// }
    /// ```
    pub fn check_as_completion(&self, op_result: &mut i32) -> i32 {
        unsafe { Cli_CheckAsCompletion(self.handle, op_result as *mut c_int) }
    }

    ///
    /// 等待直到当前的异步任务完成或超时结束。
    ///
    /// **输入参数:**
    ///
    ///  - timeout: 超时，单位 ms
    ///
    /// **返回值:**
    ///  - 0: 已完成
    ///  - 0x02200000：任务超时
    ///  - 其它值: 见错误代码
    ///
    ///  `注：这个函数使用本地操作系统原语（事件、信号...），以避免浪费CPU时间。`
    ///
    pub fn wait_as_completion(&self, timeout: i32) -> i32 {
        unsafe { Cli_WaitAsCompletion(self.handle, timeout) }
    }

    ///
    /// 从 PLC 中异步读取数据, 你可以读取数据块(DB)、输入、输出、内部标志位(Merkers)、定时器和计数器。
    ///
    /// **输入参数:**
    ///
    ///  - area: 要读取的区域
    ///  - db_number: 要读取的数据块(DB)编号。如果区域不为 S7AreaDB 则被忽略，值为 0。
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待读取数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注:
    /// (1) 如果你需要传输一个小于PDU长度的数据，应可以考虑使用同步的 read_area()。
    /// (2) 当 word_len=S7WLBit 时，Offset(start) 必须以比特表示。
    /// 示例: DB4.DBX 10.3 的起点是 (10*8)+3=83`
    ///
    pub fn as_read_area(
        &self,
        area: AreaTable,
        db_number: i32,
        start: i32,
        size: i32,
        word_len: WordLenTable,
        buff: &mut [u8],
    ) -> Result<()> {
        let res = unsafe {
            Cli_AsReadArea(
                self.handle,
                area as c_int,
                db_number as c_int,
                start as c_int,
                size as c_int,
                word_len as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 将数据异步写入到 PLC, 这是 as_read_area() 的补充函数。
    ///
    /// **输入参数:**
    ///
    ///  - area: 要读取的区域
    ///  - db_number: 要读取的数据块(DB)编号。如果区域不为 S7AreaDB 则被忽略，值为 0。
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待写入数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：
    /// (1) 如果你需要传输一个小于PDU长度的数据，应可以考虑使用同步的 write_area()。
    /// (2) 当 word_len=S7WLBit 时，Offset(start) 必须以比特表示。
    /// 示例: DB4.DBX 10.3 的起点是 (10*8)+3=83`
    ///
    pub fn as_write_area(
        &self,
        area: AreaTable,
        db_number: i32,
        start: i32,
        size: i32,
        word_len: WordLenTable,
        buff: &mut [u8],
    ) -> Result<()> {
        let res = unsafe {
            Cli_AsWriteArea(
                self.handle,
                area as c_int,
                db_number as c_int,
                start as c_int,
                size as c_int,
                word_len as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 从 PLC DB 区异步读取数据。
    ///
    /// ```text
    /// 这是 as_read_area() 的一个精简函数，它从内部调用了 as_read_area(), 其内容为:
    ///     area = S7AreaDB.
    ///     word_len = S7WLBytes.
    /// ```
    ///
    /// **输入参数:**
    ///
    ///  - db_number: 要读取的数据块(DB)编号
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待写入数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：如果你需要传输一个小于 PDU 大小的数据，应考虑使用同步的 db_read()。`
    ///
    pub fn as_db_read(&self, db_number: i32, start: i32, size: i32, buff: &mut [u8]) -> Result<()> {
        let res = unsafe {
            Cli_AsDBRead(
                self.handle,
                db_number as c_int,
                start as c_int,
                size as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 向 PLC DB 区异步写入数据。
    ///
    /// ```markdown
    /// 这是 as_write_area() 的一个精简函数，它从内部调用了 as_write_area(), 其内容为:
    ///     area = S7AreaDB.
    ///     word_len = S7WLBytes.
    /// ```
    ///
    /// **输入参数:**
    ///
    ///  - db_number: 要读取的数据块(DB)编号
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待写入数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：如果你需要传输一个小于 PDU 大小的数据，应考虑使用同步的 db_write()。`
    ///
    pub fn as_db_write(
        &self,
        db_number: i32,
        start: i32,
        size: i32,
        buff: &mut [u8],
    ) -> Result<()> {
        let res = unsafe {
            Cli_AsDBWrite(
                self.handle,
                db_number as c_int,
                start as c_int,
                size as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 从 PLC 输出区异步读取数据。
    ///
    /// ```text
    /// 这是 as_read_area() 的一个精简函数，它从内部调用了 as_read_area(), 其内容为:
    ///     area = S7AreaPA.
    ///     word_len = S7WLBytes.
    /// ```
    ///
    /// **输入参数:**
    ///
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待写入数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：如果你需要传输一个小于 PDU 大小的数据，应考虑使用同步的 ab_read()。`
    ///
    pub fn as_ab_read(&self, start: i32, size: i32, buff: &mut [u8]) -> Result<()> {
        let res = unsafe {
            Cli_AsABRead(
                self.handle,
                start as c_int,
                size as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 向 PLC 输出区异步写入数据。
    ///
    /// ```markdown
    /// 这是 as_write_area() 的一个精简函数，它从内部调用了 as_write_area(), 其内容为:
    ///     area = S7AreaPA.
    ///     word_len = S7WLBytes.
    /// ```
    ///
    /// **输入参数:**
    ///
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待写入数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：如果你需要传输一个小于 PDU 大小的数据，应考虑使用同步的 ab_write()。`
    ///
    pub fn as_ab_write(&self, start: i32, size: i32, buff: &mut [u8]) -> Result<()> {
        let res = unsafe {
            Cli_AsABWrite(
                self.handle,
                start as c_int,
                size as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 从 PLC 输入区异步读取数据。
    ///
    /// ```text
    /// 这是 as_read_area() 的一个精简函数，它从内部调用了 as_read_area(), 其内容为:
    ///     area = S7AreaPE.
    ///     word_len = S7WLBytes.
    /// ```
    ///
    /// **输入参数:**
    ///
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待写入数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：如果你需要传输一个小于 PDU 大小的数据，应考虑使用同步的 eb_read()。`
    ///
    pub fn as_eb_read(&self, start: i32, size: i32, buff: &mut [u8]) -> Result<()> {
        let res = unsafe {
            Cli_AsEBRead(
                self.handle,
                start as c_int,
                size as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 向 PLC 输入区异步写入数据。
    ///
    /// ```markdown
    /// 这是 as_write_area() 的一个精简函数，它从内部调用了 as_write_area(), 其内容为:
    ///     area = S7AreaPE.
    ///     word_len = S7WLBytes.
    /// ```
    ///
    /// **输入参数:**
    ///
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待写入数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：如果你需要传输一个小于 PDU 大小的数据，应考虑使用同步的 eb_write()。`
    ///
    pub fn as_eb_write(&self, start: i32, size: i32, buff: &mut [u8]) -> Result<()> {
        let res = unsafe {
            Cli_AsEBWrite(
                self.handle,
                start as c_int,
                size as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 从 PLC 内部标志位(Merkers)异步读取数据。
    ///
    /// ```text
    /// 这是 as_read_area() 的一个精简函数，它从内部调用了 as_read_area(), 其内容为:
    ///     area = S7AreaMK.
    ///     word_len = S7WLBytes.
    /// ```
    ///
    /// **输入参数:**
    ///
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待写入数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：如果你需要传输一个小于 PDU 大小的数据，应考虑使用同步的 mb_read()。`
    ///
    pub fn as_mb_read(&self, start: i32, size: i32, buff: &mut [u8]) -> Result<()> {
        let res = unsafe {
            Cli_AsMBRead(
                self.handle,
                start as c_int,
                size as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 向 PLC 内部标志位(Merkers)异步写入数据。
    ///
    /// ```markdown
    /// 这是 as_write_area() 的一个精简函数，它从内部调用了 as_write_area(), 其内容为:
    ///     area = S7AreaMK.
    ///     word_len = S7WLBytes.
    /// ```
    ///
    /// **输入参数:**
    ///
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待写入数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：如果你需要传输一个小于 PDU 大小的数据，应考虑使用同步的 mb_write()。`
    ///
    pub fn as_mb_write(&self, start: i32, size: i32, buff: &mut [u8]) -> Result<()> {
        let res = unsafe {
            Cli_AsMBWrite(
                self.handle,
                start as c_int,
                size as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 异步读取 PLC 定时器数据。
    ///
    /// ```text
    /// 这是 as_read_area() 的一个精简函数，它从内部调用了 as_read_area(), 其内容为:
    ///     area = S7AreaTM.
    ///     word_len = S7WLTimer.
    /// ```
    ///
    /// **输入参数:**
    ///
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待写入数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：如果你需要传输一个小于 PDU 大小的数据，应考虑使用同步的 tm_read()。`
    /// `    缓冲区大小 = size * 2`
    ///
    pub fn as_tm_read(&self, start: i32, size: i32, buff: &mut [u8]) -> Result<()> {
        let res = unsafe {
            Cli_AsTMRead(
                self.handle,
                start as c_int,
                size as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 向 PLC 定时器异步写入数据。
    ///
    /// ```markdown
    /// 这是 as_write_area() 的一个精简函数，它从内部调用了 as_write_area(), 其内容为:
    ///     area = S7AreaTM.
    ///     word_len = S7WLTimer.
    /// ```
    ///
    /// **输入参数:**
    ///
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待写入数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：如果你需要传输一个小于 PDU 大小的数据，应考虑使用同步的 tm_write()。`
    /// `    缓冲区大小 = size * 2`
    ///
    pub fn as_tm_write(&self, start: i32, size: i32, buff: &mut [u8]) -> Result<()> {
        let res = unsafe {
            Cli_AsTMWrite(
                self.handle,
                start as c_int,
                size as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 异步读取 PLC 计数器数据。
    ///
    /// ```text
    /// 这是 as_read_area() 的一个精简函数，它从内部调用了 as_read_area(), 其内容为:
    ///     area = S7AreaCT.
    ///     word_len = S7WLCounter.
    /// ```
    ///
    /// **输入参数:**
    ///
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待写入数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：如果你需要传输一个小于 PDU 大小的数据，应考虑使用同步的 ct_read()。`
    /// `    缓冲区大小 = size * 2`
    ///
    pub fn as_ct_read(&self, start: i32, size: i32, buff: &mut [u8]) -> Result<()> {
        let res = unsafe {
            Cli_AsCTRead(
                self.handle,
                start as c_int,
                size as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 向 PLC 计数器异步写入数据。
    ///
    /// ```markdown
    /// 这是 as_write_area() 的一个精简函数，它从内部调用了 as_write_area(), 其内容为:
    ///     area = S7AreaCT.
    ///     word_len = S7WLCounter.
    /// ```
    ///
    /// **输入参数:**
    ///
    ///  - start: 开始读取的字节索引
    ///  - size: 要读取的字节长度
    ///  - buff: 待写入数据缓冲区
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注：如果你需要传输一个小于 PDU 大小的数据，应考虑使用同步的 ct_write()。`
    /// `    缓冲区大小 = size * 2`
    ///
    pub fn as_ct_write(&self, start: i32, size: i32, buff: &mut [u8]) -> Result<()> {
        let res = unsafe {
            Cli_AsCTWrite(
                self.handle,
                start as c_int,
                size as c_int,
                buff as *mut [u8] as *mut c_void,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 该函数异步返回指定区块类型的 AG 列表。
    ///
    /// **输入参数:**
    ///
    ///  - block_type: 要获取的区块类型
    ///  - buff: 待写入数据缓冲区
    ///  - items_count: 在输入中表示用户缓冲区的容量，在输出中表示找到了多少个项目
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// # Examples
    /// ```ignore
    ///    let mut buff: TS7BlocksOfType = [0; 8192];
    ///    let mut items_count = buff.len() as i32;
    ///    client.as_list_blocks_of_type(BlockType::BlockDB, &mut buff, &mut items_count);
    /// ```
    ///
    pub fn as_list_blocks_of_type(
        &self,
        block_type: BlockType,
        buff: &mut TS7BlocksOfType,
        items_count: &mut i32,
    ) -> Result<()> {
        let res = unsafe {
            Cli_AsListBlocksOfType(
                self.handle,
                block_type as c_int,
                buff as *mut TS7BlocksOfType,
                items_count as *mut c_int,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 通过一个给定 ID 和 INDEX 异步读取局部系统状态列表。
    ///
    /// **输入参数:**
    ///
    /// - id: 列表 ID
    /// - index: 列表 INDEX
    /// - ts7szl: TS7SZL 结构体
    /// - size: 输入时为缓冲区大小，输出时为读取到的字节数
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn as_read_szl(
        &self,
        id: i32,
        index: i32,
        ts7szl: &mut TS7SZL,
        size: &mut i32,
    ) -> Result<()> {
        let res = unsafe {
            Cli_AsReadSZL(
                self.handle,
                id,
                index,
                ts7szl as *mut TS7SZL,
                size as *mut c_int,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 异步读取局部系统状态列表的目录。
    ///
    /// **输入参数:**
    ///
    /// - ts7szl_list: TS7SZLList 结构体
    /// - items_count: 输入时为缓冲区大小，输出时为发现的项目数量
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn as_read_szl_list(
        &self,
        ts7szl_list: &mut TS7SZLList,
        items_count: &mut i32,
    ) -> Result<()> {
        let res = unsafe {
            Cli_AsReadSZLList(
                self.handle,
                ts7szl_list as *mut TS7SZLList,
                items_count as *mut c_int,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 从 AG 异步上传一个区块。将整个区块复制到用户缓冲区。
    ///
    /// **输入参数:**
    ///
    ///  - block_type: 要获取的区块类型
    ///  - block_num: 要获取的区块号
    ///  - buff: 用户缓冲区
    ///  - size: 在输入中表示缓冲区大小，在输出中表示上传的字节数
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// # Examples
    /// ```ignore
    ///    let mut buff = [0; 4096];
    ///    let mut size = buff.len() as i32;
    ///    client.as_full_upload(BlockType::BlockSDB, 0, &mut buff, &mut size);
    /// ```
    ///
    pub fn as_full_upload(
        &self,
        block_type: BlockType,
        block_num: i32,
        buff: &mut [u8],
        size: &mut i32,
    ) -> Result<()> {
        let res = unsafe {
            Cli_AsFullUpload(
                self.handle,
                block_type as c_int,
                block_num as c_int,
                buff as *mut [u8] as *mut c_void,
                size as *mut c_int,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 从 AG 异步上传一个区块主体。只将区块主体复制到用户缓冲区。
    ///
    /// **输入参数:**
    ///
    ///  - block_type: 要获取的区块类型
    ///  - block_num: 要获取的区块号
    ///  - buff: 用户缓冲区
    ///  - size: 在输入中表示缓冲区大小，在输出中表示上传的字节数
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// # Examples
    /// ```ignore
    ///    let mut buff = [0; 4096];
    ///    let mut size = buff.len() as i32;
    ///    client.as_upload(BlockType::BlockSDB, 0, &mut buff, &mut size);
    /// ```
    ///
    pub fn as_upload(
        &self,
        block_type: BlockType,
        block_num: i32,
        buff: &mut [u8],
        size: &mut i32,
    ) -> Result<()> {
        let res = unsafe {
            Cli_AsUpload(
                self.handle,
                block_type as c_int,
                block_num as c_int,
                buff as *mut [u8] as *mut c_void,
                size as *mut c_int,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 从 AG 异步下载一个区块。将用户缓冲区复制到整个区块。
    ///
    /// **输入参数:**
    ///
    ///  - block_num: 新区块编号，或 -1
    ///  - buff: 用户缓冲区
    ///  - size: 缓冲区大小
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    /// `注:一个准备被下载的区块已经包含了关于区块类型和区块编号的信息。 如果参数 block_num 为 -1，则区块编号不会被改变，否则区块将以设置的编号被下载。`
    ///
    pub fn as_download(&self, block_num: i32, buff: &mut [u8], size: i32) -> Result<()> {
        let res = unsafe {
            Cli_AsDownload(
                self.handle,
                block_num as c_int,
                buff as *mut [u8] as *mut c_void,
                size as c_int,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 从 AG 异步上传一个 DB，这个函数等同于 upload() 的参数 block_type = Block_DB，
    /// 但是它使用了一个不同的方法，所以它不受安全级别设置的限制。这个方法只上传数据。
    ///
    /// **输入参数:**
    ///
    ///  - block_num: 要上传的 DB 块编号
    ///  - buff: 用户缓冲区
    ///  - size: 在输入中表示缓冲区大小，在输出中表示上传的字节数
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    pub fn as_db_get(&self, block_num: i32, buff: &mut [u8], size: &mut i32) -> Result<()> {
        let res = unsafe {
            Cli_AsDBGet(
                self.handle,
                block_num as c_int,
                buff as *mut [u8] as *mut c_void,
                size as *mut c_int,
            )
        };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 用一个给定的字节异步填充 AG 中的一个 DB，而不需要指定其大小。
    ///
    /// **输入参数:**
    ///
    ///  - block_num: 要填充的 DB 块编号
    ///  - fill_char: 要填充的字节
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    ///  `注：出于效率考虑，fill_char 是一个整数，且只有最低的字节被使用`
    ///
    pub fn as_db_fill(&self, block_num: i32, fill_char: i32) -> Result<()> {
        let res = unsafe { Cli_AsDBFill(self.handle, block_num as c_int, fill_char as c_int) };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 异步执行复制 RAM 到 ROM。
    ///
    /// **输入参数:**
    ///
    /// - timeout: 预期完成操作的最大时间(ms)
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    ///  `注：不是所有的 CPU 都支持这个操作，CPU 必须处于 STOP 模式。`
    ///
    pub fn as_copy_ram_to_rom(&self, timeout: i32) -> Result<()> {
        let res = unsafe { Cli_AsCopyRamToRom(self.handle, timeout) };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }

    ///
    /// 异步执行内存压缩。
    ///
    /// **输入参数:**
    ///
    /// - timeout: 预期完成操作的最大时间(ms)
    ///
    /// **返回值:**
    ///
    ///  - Ok: 操作成功
    ///  - Err: 操作失败
    ///
    ///  `注：不是所有的 CPU 都支持这个操作，CPU 必须处于 STOP 模式。`
    ///
    pub fn as_compress(&self, timeout: i32) -> Result<()> {
        let res = unsafe { Cli_AsCompress(self.handle, timeout) };
        if res == 0 {
            return Ok(());
        }
        bail!("{}", Self::error_text(res))
    }
}

unsafe extern "C" fn call_as_closure<F>(usr_ptr: *mut c_void, op_code: c_int, op_result: c_int)
where
    F: FnMut(*mut c_void, c_int, c_int),
{
    let callback_ptr = usr_ptr as *mut F;
    let callback = &mut *callback_ptr;
    callback(usr_ptr, op_code, op_result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client() {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let client = S7Client::create();

        client
            .set_as_callback(Some(|_, op_code, op_result| {
                println!("op_code: {}", op_code);
                println!("op_result: {:?}", S7Client::error_text(op_result));
            }))
            .unwrap();

        let value = InternalParamValue::U16(7878);
        assert!(client.set_param(InternalParam::RemotePort, value).is_ok());
        if let Err(e) = client.connect_to("127.0.0.1", 0, 1) {
            dbg!(e);
            return;
        }
        client
            .set_param(InternalParam::PingTimeout, InternalParamValue::I32(777))
            .unwrap();
        let mut value = InternalParamValue::U16(0);
        assert!(client
            .get_param(InternalParam::RemotePort, &mut value)
            .is_ok());
        println!("RemotePort: {:?}", value);
        assert!(client
            .get_param(InternalParam::PingTimeout, &mut value)
            .is_ok());
        println!("PingTimeout: {:?}", value);
        assert!(client
            .get_param(InternalParam::SendTimeout, &mut value)
            .is_ok());
        println!("SendTimeout: {:?}", value);
        assert!(client
            .get_param(InternalParam::RecvTimeout, &mut value)
            .is_ok());
        println!("RecvTimeout: {:?}", value);
        assert!(client.get_param(InternalParam::SrcRef, &mut value).is_ok());
        println!("SrcRef: {:?}", value);
        assert!(client.get_param(InternalParam::DstRef, &mut value).is_ok());
        println!("DstRef: {:?}", value);
        assert!(client.get_param(InternalParam::SrcTSap, &mut value).is_ok());
        println!("SrcTSap: {:?}", value);
        assert!(client
            .get_param(InternalParam::PDURequest, &mut value)
            .is_ok());
        println!("PDURequest: {:?}", value);
        let mut buff = [0u8; 2];
        assert!(client
            .read_area(
                AreaTable::S7AreaDB,
                1,
                0,
                2,
                WordLenTable::S7WLWord,
                &mut buff,
            )
            .is_ok());
        println!("word: {}", u16::from_be_bytes([buff[0], buff[1]]));

        let mut buff = [1u8; 1];
        assert!(client
            .write_area(
                AreaTable::S7AreaDB,
                1,
                81,
                1,
                WordLenTable::S7WLBit,
                &mut buff,
            )
            .is_ok());

        let mut buff = [0u8; 1];
        assert!(client
            .read_area(
                AreaTable::S7AreaDB,
                1,
                81,
                1,
                WordLenTable::S7WLBit,
                &mut buff,
            )
            .is_ok());
        println!("bit: {:#x?}", &buff);

        let mut buff = 13.14f32.to_be_bytes();
        assert!(client
            .write_area(
                AreaTable::S7AreaDB,
                1,
                24,
                1,
                WordLenTable::S7WLDWord,
                &mut buff,
            )
            .is_ok());
        println!("{:#x?}", &buff);

        let mut buff = [0u8; 4];
        assert!(client
            .read_area(
                AreaTable::S7AreaDB,
                1,
                24,
                1,
                WordLenTable::S7WLDWord,
                &mut buff,
            )
            .is_ok());
        println!("{:#x?}", &buff);
        println!(
            "dword: {}",
            f32::from_be_bytes([buff[0], buff[1], buff[2], buff[3]])
        );

        let mut buff = 13.14f32.to_be_bytes();
        assert!(client.db_write(1, 20, 4, &mut buff,).is_ok());
        println!("{:#x?}", &buff);

        let mut buff = [0u8; 4];
        assert!(client.db_read(1, 20, 4, &mut buff).is_ok());
        println!("{:#x?}", &buff);
        println!(
            "dword: {}",
            f32::from_be_bytes([buff[0], buff[1], buff[2], buff[3]])
        );

        let mut buff = 77u16.to_be_bytes();
        assert!(client.db_write(1, 0, 2, &mut buff,).is_ok());
        println!("{:#x?}", &buff);

        let mut buff = [0u8; 2];
        assert!(client.db_read(1, 0, 2, &mut buff).is_ok());
        println!("{:#x?}", &buff);
        println!("dword: {}", u16::from_be_bytes([buff[0], buff[1]]));

        let mut buff = [0b0u8; 1];
        assert!(client.ab_write(0, 1, &mut buff).is_ok());

        let mut buff = [0u8; 1];
        assert!(client.ab_read(0, 1, &mut buff).is_ok());
        println!("{:#b}", &buff[0]);

        let mut buff = [0b1u8; 1];
        assert!(client.eb_write(0, 1, &mut buff).is_ok());

        let mut buff = [0u8; 1];
        assert!(client.eb_read(0, 1, &mut buff).is_ok());
        println!("{:#x?}", &buff);

        let mut buff = [0b1u8; 1];
        assert!(client.mb_write(0, 1, &mut buff).is_ok());

        let mut buff = [0u8; 1];
        assert!(client.mb_read(0, 1, &mut buff).is_ok());
        println!("{:#x?}", &buff);

        let mut db1 = [0u8; 2];
        let mut in1 = [0u8; 1];
        let item0 = TS7DataItem {
            Area: AreaTable::S7AreaDB as c_int,
            WordLen: WordLenTable::S7WLByte as c_int,
            Result: 0,
            DBNumber: 1,
            Start: 0,
            Amount: 2,
            pdata: &mut db1 as *mut [u8] as *mut c_void,
        };
        let item1 = TS7DataItem {
            Area: AreaTable::S7AreaPA as c_int,
            WordLen: WordLenTable::S7WLBit as c_int,
            Result: 0,
            DBNumber: 0,
            Start: 0,
            Amount: 1,
            pdata: &mut in1 as *mut [u8] as *mut c_void,
        };
        let mut item = [item0, item1];
        assert!(client.read_multi_vars(&mut item, 2).is_ok());
        println!("{:?}", u16::from_be_bytes([db1[0], db1[1]]));
        println!("{:?}", in1);

        let mut ts7_blocks_list = TS7BlocksList::default();
        assert!(client.list_blocks(&mut ts7_blocks_list).is_ok());
        println!("{:?}", &ts7_blocks_list);

        let mut buff: TS7BlocksOfType = [0; 8192];
        let mut items_count = buff.len() as i32;
        assert!(client
            .list_blocks_of_type(BlockType::BlockDB, &mut buff, &mut items_count)
            .is_ok());
        println!("buff: {:?}", &buff[0..items_count as usize]);
        println!("items_count: {:?}", items_count);

        // let mut buff = [0u8; 4096];
        // let mut size = buff.len() as i32;
        // assert!(client.full_upload(BlockType::BlockSDB, 0, &mut buff, &mut size));
        // println!("buff: {:?}", &buff[0..size as usize]);
        // println!("size: {:?}", size);

        // let mut buff = [0u8; 4096];
        // let mut size = buff.len() as i32;
        // assert!(client.upload(BlockType::BlockSDB, 0, &mut buff, &mut size));
        // println!("buff: {:?}", &buff[0..size as usize]);
        // println!("size: {:?}", size);

        assert!(client.set_plc_system_date_time().is_ok());

        let mut date_time = DateTime::default();
        assert!(client.get_plc_date_time(&mut date_time).is_ok());
        println!("{:?}", date_time);

        let mut is_connected = 0;
        assert!(client.get_connected(&mut is_connected).is_ok());
        println!("connected: {:?}", is_connected);

        let mut status = 0;
        if let Err(e) = client.get_plc_status(&mut status) {
            dbg!(e);
        }
        println!("plc status: {:?}", status);

        let mut buff = [0u8; 2];
        if let Err(e) = client.as_read_area(
            AreaTable::S7AreaDB,
            1,
            0,
            2,
            WordLenTable::S7WLWord,
            &mut buff,
        ) {
            dbg!(e);
        }
        println!(
            "as_read_area(word): {}",
            u16::from_be_bytes([buff[0], buff[1]])
        );
        loop {
            let mut op = 0;
            if client.check_as_completion(&mut op) == 0 {
                println!(
                    "as_read_area(word): {}",
                    u16::from_be_bytes([buff[0], buff[1]])
                );
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        let mut buff = [0u8; 4];
        if let Err(e) = client.as_read_area(
            AreaTable::S7AreaDB,
            1,
            20,
            1,
            WordLenTable::S7WLDWord,
            &mut buff,
        ) {
            dbg!(e);
        }
        println!(
            "as_read_area(float): {}",
            f32::from_be_bytes([buff[0], buff[1], buff[2], buff[3]])
        );
        dbg!(client.wait_as_completion(100));
        println!(
            "as_read_area(float): {}",
            f32::from_be_bytes([buff[0], buff[1], buff[2], buff[3]])
        );
    }
}
