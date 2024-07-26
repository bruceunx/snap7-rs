# snap7-rs -> rust-snap7

> [!IMPORTANT]
> [2024 Jun 25] crate name change to `rust-snap7`

## This repository add utils module like in `python-snap7` to handle data from PLC or vice verse.

### Utils Example

```rust
use snap7_rs::utils;
use snap7_rs::S7Client;
use snap7_rs::{InternalParam, InternalParamValue};

#[allow(dead_code)]
fn connect_to_plc() -> Result<(), String> {
    let client = S7Client::create();
    client
        .set_param(InternalParam::RemotePort, InternalParamValue::U16(1102))
        .expect("failed to set remote port value! Exiting..");
    if let Err(e) = client.connect_to("127.0.0.1", 1, 1) {
        println!("Connection to PLC failed: {:?}", e);
    } else {
        let mut buff = [0u8; 2];
        if let Err(e) = client.db_read(1, 20, 2, &mut buff) {
            println!("Failed to read DB: {:?}", e);
        } else {
            let data = utils::getters::get_int(&buff.to_vec(), 0);
            println!("DB1.W20: {}", data);
        }
        client
            .disconnect()
            .expect("client disconnect failed! Exiting..");
    }
    Ok(())
}
fn main() {
    connect_to_plc().expect("conenct fn error");
}

```

This is a Rust binding of the [snap7](http://snap7.sourceforge.net/) C++ library, linked statically to snap7 with no additional dependencies.

> Warning: This library has not undergone any security clearance and is to be used at your own risk.

**Note**: This repository is based on the original [snap7-rs](https://gitee.com/gmg137/snap7-rs.git), and was created to translate some stuff in the original repository to English and fix some compilation errors.

### Client Example

```rust
    use snap7_rs::S7Client;
    use std::ffi::*;
    use std::os::raw::*;

    // Creating an S7 Client
    let client = S7Client::create();
    // Connection to PLC
    if let Err(e) = client.connect_to("192.168.1.123", 0, 1) {
        println!("Connection to PLC failed: {:?}", e);
    } else {
        // Create a data buffer
        let mut buff = [0u8; 2];
        // Read the value of DB1.WDB20 from the PLC
        if let Err(e) = client.db_read(1, 20, 2, &mut buff) {
            println!("Failed to read DB: {:?}", e);
        } else {
            println!("DB1.W20: {}", u16::from_be_bytes([buff[0], buff[1]]));
        }
    }
```

### Server-side example

```rust
    use snap7_rs::{AreaCode, InternalParam, InternalParamValue, S7Server, MaskKind};
    use std::ffi::*;
    use std::os::raw::*;

    // Creating the S7 Server
    let server = S7Server::create();

    // Creating shared memory areas
    let mut db_buff = [0u8; 1024];

    // Adding Shared Blocks
    assert!(server
        .register_area(AreaCode::S7AreaDB, 1, &mut db_buff)
        .is_ok());

    // Filtering reads and writes
    assert!(server
        .set_mask(MaskKind::Event, 0x00020000 | 0x00040000)
        .is_ok());

    // Setting event callbacks
    assert!(server
        .set_events_callback(Some(move |_, p_event, _| {
            if let Ok(text) = S7Server::event_text(p_event) {
                println!("{:?}", text);
            }
        }))
        .is_ok());

    // Start Service
    if let Err(e) = server.start() {
        dbg!(e);
    }

    // Business Logic
    //loop {
       // ......
    //}

    // Close service
    assert!(server.stop().is_ok());
```

### Passive partner example

```rust
    use snap7_rs::S7Partner;
    use std::ffi::*;
    use std::os::raw::*;

    // Create S7 Passive Partners
    let partner = S7Partner::create(0);

    // Set the receive callback
    partner
        .set_recv_callback(Some(|_, op, r_id, p_data: *mut c_void, size: i32| unsafe {
            let buff = std::slice::from_raw_parts(p_data as *const u8, size as usize);
            println!("op: {}, r_id:{}, p_data:{:#x?}", op, r_id, buff);
        }))
        .unwrap();

    // Launch Partner Services
    if let Err(e) = partner.start_to("0.0.0.0", "127.0.0.1", 0x1002, 0x1002) {
        dbg!(e);
    }

    // Business Logic
    //loop {
    //    ...
    //}

    // Stop service
    partner.stop().unwrap();
```

### Active partner example

```rust
    use snap7_rs::S7Partner;
    use std::ffi::*;
    use std::os::raw::*;

    // Create S7 Active Partners
    let partner = S7Partner::create(1);

    // Set the send callback
    partner
        .set_send_callback(Some(|_, op| {
            dbg!(S7Partner::error_text(op));
        }))
        .unwrap();

    // Launch Partner Services
    if let Err(e) = partner.start_to("0.0.0.0", "127.0.0.1", 0x1002, 0x1002) {
        dbg!(e);
    }

    let mut buff = [0x01u8, 0x02, 0x03, 0x04, 0x05, 0x06];
    if let Err(e) = partner.b_send(1, &mut buff) {
        dbg!(e);
    } else {
        dbg!("Sync send successful!");
    }

    let mut buff = [0x07u8, 0x08, 0x09, 0x0a, 0x0b, 0x0c];
    if let Err(e) = partner.as_b_send(1, &mut buff) {
        dbg!(e);
    } else {
        dbg!("Asynchronous sending...");
    }

    dbg!(S7Partner::error_text(partner.wait_as_b_send_completion(10)));

    // Business Logic
    //loop {
    //    ...
    //}

    // Stop service
    partner.stop().unwrap();
```

### License

The source code and documentation for this project are under the [Mulan Loose License](LICENSE) (MulanPSL-2.0).

[snap7](http://snap7.sourceforge.net/) itself is licensed under the terms of the [GNU Lesser General Public License](https://www.gnu.org/licenses/lgpl-3.0.html) (LGPL v3+).
