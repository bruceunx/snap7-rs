# snap7-rs -> rust-snap7

> [!IMPORTANT] > [2024 Jun 25] crate name change to `rust-snap7`

## This repository add utils module like in `python-snap7` to handle data from PLC or vice verse.

### Utils Example

```rust
use std::thread;
use std::time::Duration;

use rust_snap7::utils;
use rust_snap7::AreaCode;
use rust_snap7::MaskKind;
use rust_snap7::S7Client;
use rust_snap7::S7Server;
use rust_snap7::{InternalParam, InternalParamValue};

fn create_s7_server() -> Result<(), Box<dyn std::error::Error>> {
    // Creating the S7 Server
    let server = S7Server::create();

    // Creating shared memory areas
    let mut db_buff = [0u8; 1024];

    // Adding Shared Blocks
    server
        .register_area(AreaCode::S7AreaDB, 1, &mut db_buff)
        .expect("Failed to register area");

    // Filtering reads and writes
    server
        .set_mask(MaskKind::Event, 0x00020000 | 0x00040000)
        .expect("Failed to set mask");

    // Setting event callbacks
    server
        .set_events_callback(Some(move |_, p_event, _| {
            if let Ok(text) = S7Server::event_text(p_event) {
                println!("Server Event: {:?}", text);
            }
        }))
        .expect("Failed to set events callback");

    // Start Service
    server.start().expect("Failed to start server");

    // Business Logic would go here
    // For this example, we'll just wait a bit
    std::thread::sleep(std::time::Duration::from_secs(10));

    // Close service
    server.stop().expect("Failed to stop server");

    Ok(())
}

fn connect_to_plc() -> Result<(), String> {
    let client = S7Client::create();
    // Configure client parameters
    client
        .set_param(InternalParam::RemotePort, InternalParamValue::U16(102))
        .expect("Failed to set remote port value");

    // Attempt to connect to PLC
    match client.connect_to("127.0.0.1", 0, 1) {
        Ok(_) => {
            let mut buff = [0u8; 2];

            // Read from DB
            match client.db_read(1, 20, 2, &mut buff) {
                Ok(_) => {
                    let data = utils::getters::get_int(buff.to_vec().as_ref(), 0);
                    println!("Successfully read from DB1.W20: {}", data);
                }
                Err(e) => println!("Failed to read DB: {:?}", e),
            }

            // Disconnect client
            client.disconnect().expect("Client disconnect failed");
        }
        Err(e) => println!("Connection to PLC failed: {:?}", e),
    }

    Ok(())
}

fn main() {
    std::thread::spawn(|| {
        if let Err(e) = create_s7_server() {
            eprintln!("S7 Server Error: {}", e);
        }
    });

    thread::sleep(Duration::from_millis(500));

    connect_to_plc().expect("conenct fn error");
}


```

This is a Rust binding of the [snap7](http://snap7.sourceforge.net/) C++ library, linked statically to snap7 with no additional dependencies.

> Warning: This library has not undergone any security clearance and is to be used at your own risk.

**Note**: This repository is based on the original [snap7-rs](https://gitee.com/gmg137/snap7-rs.git), and was created to translate some stuff in the original repository to English and fix some compilation errors.

### License

The source code and documentation for this project are under the [Mulan Loose License](LICENSE) (MulanPSL-2.0).

[snap7](http://snap7.sourceforge.net/) itself is licensed under the terms of the [GNU Lesser General Public License](https://www.gnu.org/licenses/lgpl-3.0.html) (LGPL v3+).
