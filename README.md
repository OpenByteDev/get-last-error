# get-last-error

[![CI](https://github.com/OpenByteDev/get-last-error/actions/workflows/ci.yml/badge.svg)](https://github.com/OpenByteDev/get-last-error/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/get-last-error.svg)](https://crates.io/crates/get-last-error)
[![Documentation](https://docs.rs/get-last-error/badge.svg)](https://docs.rs/get-last-error)
[![dependency status](https://deps.rs/repo/github/openbytedev/get-last-error/status.svg)](https://deps.rs/repo/github/openbytedev/get-last-error)
[![MIT](https://img.shields.io/crates/l/get-last-error.svg)](https://github.com/OpenByteDev/get-last-error/blob/master/LICENSE)

An error wrapper over Win32 API errors.

## Examples

A `Win32Error` can be constructed from an arbitrary `DWORD`:

```rust
use get_last_error::Win32Error;

let err = Win32Error::new(0);
println!("{}", err); // prints "The operation completed successfully."
```

The `Win32Error::get_last_error` retrieves the last error code for the current thread:

```rust
use get_last_error::Win32Error;
use winapi::um::{winnt::HANDLE, processthreadsapi::OpenProcess};

fn open_process() -> Result<HANDLE, Win32Error> {
    let result = unsafe { OpenProcess(0, 0, 0) }; // some windows api call
    if result.is_null() { // null indicates failure.
        Err(Win32Error::get_last_error())
    } else {
        Ok(result)
    }
}
```

## License
Licensed under MIT license ([LICENSE](https://github.com/OpenByteDev/get-last-error/blob/master/LICENSE) or http://opensource.org/licenses/MIT)
