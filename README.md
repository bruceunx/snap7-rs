# snap7-rs

这是 [snap7](http://snap7.sourceforge.net/) C++ 库的 Rust 绑定，通过静态链接到 snap7，无需额外依赖。

> 警告：本库未经过任何安全审查，使用后果自负。

[![Crates.io](https://img.shields.io/crates/v/snap7-rs.svg)](https://crates.io/crates/snap7-rs)
[![Documentation](https://docs.rs/snap7-rs/badge.svg)](https://docs.rs/snap7-rs/)
[![License](https://img.shields.io/badge/license-MulanPSL2-green)](http://license.coscl.org.cn/MulanPSL2/)

### 客户端示例
```rust
    use snap7_rs::S7Client;
    use std::ffi::*;
    use std::os::raw::*;

    // 创建 S7 客户端
    let client = S7Client::create();
    // 连接到 PLC
    if let Err(e) = client.connect_to("192.168.1.123", 0, 1) {
        println!("连接 PLC 失败: {:?}", e);
    } else {
        // 创建一个数据缓冲区
        let mut buff = [0u8; 2];
        // 从 PLC 读取 DB1.WDB20 的值
        if let Err(e) = client.db_read(1, 20, 2, &mut buff) {
            println!("读取 DB 失败: {:?}", e);
        } else {
            println!("DB1.W20: {}", u16::from_be_bytes([buff[0], buff[1]]));
        }
    }
```

### 服务端示例
```rust
    use snap7_rs::{AreaCode, InternalParam, InternalParamValue, S7Server, MaskKind};
    use std::ffi::*;
    use std::os::raw::*;

    // 创建 S7 服务端
    let server = S7Server::create();

    // 创建共享内存区
    let mut db_buff = [0u8; 1024];

    // 添加共享区块
    assert!(server
        .register_area(AreaCode::S7AreaDB, 1, &mut db_buff)
        .is_ok());

    // 过滤读和写
    assert!(server
        .set_mask(MaskKind::Event, 0x00020000 | 0x00040000)
        .is_ok());

    // 设置事件回调
    assert!(server
        .set_events_callback(Some(move |_, p_event, _| {
            if let Ok(text) = S7Server::event_text(p_event) {
                println!("{:?}", text);
            }
        }))
        .is_ok());

    // 启动服务
    if let Err(e) = server.start() {
        dbg!(e);
    }

    // 处理逻辑
    //loop {
       // ......
    //}

    // 关闭服务
    assert!(server.stop().is_ok());
```

### 被动伙伴示例
```rust
    use snap7_rs::S7Partner;
    use std::ffi::*;
    use std::os::raw::*;

    // 创建 S7 被动伙伴
    let partner = S7Partner::create(0);

    // 设置接收回调
    partner
        .set_recv_callback(Some(|_, op, r_id, p_data: *mut c_void, size: i32| unsafe {
            let buff = std::slice::from_raw_parts(p_data as *const u8, size as usize);
            println!("op: {}, r_id:{}, p_data:{:#x?}", op, r_id, buff);
        }))
        .unwrap();

    // 启动伙伴服务
    if let Err(e) = partner.start_to("0.0.0.0", "127.0.0.1", 0x1002, 0x1002) {
        dbg!(e);
    }

    // 业务逻辑
    //loop {
    //    ...
    //}

    // 停止服务
    partner.stop().unwrap();
```

### 主动伙伴示例
```rust
    use snap7_rs::S7Partner;
    use std::ffi::*;
    use std::os::raw::*;

    // 创建 S7 主动伙伴
    let partner = S7Partner::create(1);

    // 设置发送回调
    partner
        .set_send_callback(Some(|_, op| {
            dbg!(S7Partner::error_text(op));
        }))
        .unwrap();

    // 启动伙伴服务
    if let Err(e) = partner.start_to("0.0.0.0", "127.0.0.1", 0x1002, 0x1002) {
        dbg!(e);
    }

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

    dbg!(S7Partner::error_text(partner.wait_as_b_send_completion(10)));

    // 业务逻辑
    //loop {
    //    ...
    //}

    // 停止服务
    partner.stop().unwrap();
```

### License
本项目源码和文档采用[木兰宽松许可证](LICENSE) (MulanPSL-2.0)。

[snap7](http://snap7.sourceforge.net/) itself is licensed under the terms of the [GNU Lesser General Public License](https://www.gnu.org/licenses/lgpl-3.0.html) (LGPL v3+).
