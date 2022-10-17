# JavaWithRust-Lib

Transfer function calls between Rust and Java in a rust-like (ish) way.

- Github.com : [https://github.com/Totobird-Creations/JavaWithRust-Lib](https://github.com/Totobird-Creations/JavaWithRust-Lib)
- Crates.io  : [https://crates.io/crates/javawithrust](https://crates.io/crates/javawithrust)
- Docs.rs    : [https://docs.rs/javawithrust](https://docs.rs/javawithrust)

## Features

* Call static java functions from rust.
* Call associated rust functions from java.
* Convert java objects to rust structs.

## Limitations

* Can not call java methods from rust.
* Can not call rust methods from java.

## Setup

Create a dynamic library crate.
```bash
# Bash

cargo init . --lib
```
```toml
# Cargo.toml

[dependencies]
javawithrust = "0.1"
serde        = {version = "1.0", features = ["derive"]}

[lib]
crate_type = ["cdylib"]
```

## Usage

Get the needed stuff.
```rust
// Rust

use javawithrust::prelude::*;
```

Link a new Java class. This will link `JavaObject` (in Rust) and `io.example.JavaClassName` (in Java).
```rust
// Rust

#[jclass(io.example.JavaClassName)]
pub struct JavaObject;
```

Link the functions.
```rust
// Rust

#[jfuncs]
impl JavaObject {

}
```

## Examples

```rust
// Rust

use javawithrust::prelude::*;

#[jclass(io.example.CustomJavaClass)]
pub struct CustomRustStruct;

#[jfuncs]
impl CustomRustStruct {

    fn hello() { // Will return `Result<(), String>`
        println!("hello");
        CustomRustStruct::world()?;
        return Ok(());
    }

    // Will call `io.example.CustomJavaClass.world()`.
    fn world();

    // Callable from java.
    fn sum(a : i32, b : i32) -> i32 { // Will return `Result<i32, String>`
        return Ok(a + b);
    }

}
```
```java
// Java

package io.example;

import org.astonbitecode.j4rs.api.Instance;
import org.astonbitecode.j4rs.api.dtos.InvocationArg;
import org.astonbitecode.j4rs.api.java2rust.Java2RustUtils;
import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.ObjectMapper;

public class ConvHelper {

    public static <T> Instance<T> j2rs(T from) {
        return Java2RustUtils.createInstance(from);
    }

    @SuppressWarnings("unchecked")
    public static <T> T rs2j(Instance<T> from) {
        if (from instanceof InvocationArg) {
            InvocationArg from_ia = ((InvocationArg)from);
            if (from_ia.isSerialized()) {
                ObjectMapper mapper = new ObjectMapper();
                try {
                    return mapper.<T>readValue(from_ia.getJson(), (Class<T>)(Class.forName(from_ia.getClassName())));
                } catch (JsonProcessingException | ClassNotFoundException e) {
                    e.printStackTrace();
                    System.exit(1);
                }
            }
        }
        return Java2RustUtils.getObjectCasted(from);
    }

}
```
```java
// Java

package io.example;

import org.astonbitecode.j4rs.api.Instance;
import org.astonbitecode.j4rs.api.java2rust.Java2RustUtils;

public class CustomJavaClass
{
    // Load the dynamic library.
    // The library is loaded from the same directory as the jar.
    static {
        String os=System.getProperty("os.name").toLowerCase();String path=J2RS.class.getProtectionDomain().getCodeSource().getLocation().getPath();path=path.substring(0,path.lastIndexOf("/")).replaceAll("%20"," ")+"/"+
        "robot"; // Name of the dynamic library file. `.so` and `.dll` are added automatically.
        if (os.contains("linux") || os.contains("unix") || os.contains("android")) {
            path += ".so";
        } else if (os.contains("win")) {
            path += ".dll";
        }
        else {throw new UnsatisfiedLinkError("Can not load dynamic library in unknown operating system `" + os + "`");}
        System.load(path);
    }

    public static native void hello();

    public static void world() {
        System.out.println("world!");
    }

    private static native Instance<Integer> sum(Instance<Integer> i1, Instance<Integer> i2);
    public static Integer sumFromRust(Integer a, Integer b) {
        // Convert the objects.
        return ConvHelper.rs2j(J2RS.sum(
            ConvHelper.j2rs(a),
            ConvHelper.j2rs(b)
        ));
    }

}
```
