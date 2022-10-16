//! Transfer function calls between Rust and Java in a rust-like (ish) way.
//! 
//! # Features
//! 
//! * Call static java functions from rust.
//! * Call associated rust functions from java.
//! * Convert java objects to rust structs.
//! 
//! # Limitations
//! 
//! * Can not call java methods from rust.
//! * Can not call rust methods from java.
//! * Can only convert the following rust types to java objects:
//!     * `String`
//!     * `i8`
//!     * `i16`
//!     * `i32`
//!     * `i64`
//!     * `f32`
//!     * `f64`
//! 
//! # Examples
//! 
//! ```
//! // Rust
//! 
//! use javawithrust::prelude::*;
//! 
//! #[jclass(io.example.CustomJavaClass)]
//! pub struct CustomRustStruct;
//! 
//! #[jfuncs]
//! impl CustomRustStruct {
//! 
//!     fn hello() { // Will return `Result<(), String>`
//!         println!("hello");
//!         CustomRustStruct::world()?;
//!         return Ok(());
//!     }
//! 
//!     // Will call `io.example.CustomJavaClass.world()`.
//!     fn world();
//! 
//!     // Callable from java.
//!     fn sum(a : i32, b : i32) -> i32 { // Will return `Result<i32, String>`
//!         return Ok(a + b);
//!     }
//! 
//! }
//! ```
//! ```java
//! // Java
//! 
//! package io.example;
//! 
//! import org.astonbitecode.j4rs.api.Instance;
//! import org.astonbitecode.j4rs.api.java2rust.Java2RustUtils;
//! 
//! public class CustomJavaClass
//! {
//!     // Load the dynamic library.
//!     // The library is loaded from the same directory as the jar.
//!     static {
//!         String os=System.getProperty("os.name").toLowerCase();String path=J2RS.class.getProtectionDomain().getCodeSource().getLocation().getPath();path=path.substring(0,path.lastIndexOf("/")).replaceAll("%20"," ")+"/"+
//!         "robot"; // Name of the dynamic library file. `.so` and `.dll` are added automatically.
//!         if (os.contains("linux") || os.contains("unix") || os.contains("android")) {
//!             path += ".so";
//!         } else if (os.contains("win")) {
//!             path += ".dll";
//!         }
//!         else {throw new UnsatisfiedLinkError("Can not load dynamic library in unknown operating system `" + os + "`");}
//!         System.load(path);
//!     }
//! 
//!     public static native void hello();
//! 
//!     public static void world() {
//!         System.out.println("world!");
//!     }
//! 
//!     private static native Instance<Integer> sum(Instance<Integer> i1, Instance<Integer> i2);
//!     public static Integer sumFromRust(Integer a, Integer b) {
//!         // Convert the objects.
//!         return Java2RustUtils.getObjectCasted(J2RS.sum(
//!             Java2RustUtils.createInstance(a),
//!             Java2RustUtils.createInstance(b)
//!         ));
//!     }
//! 
//! }
//! ```


/// Pulls in all required objects and types.
/// 
/// # Examples
/// 
/// ```
/// use javawithrust::prelude::*;
/// ```
pub mod prelude {
    pub use super::jclass;
    pub use super::jfuncs;
    pub use j4rs::{
        errors,
        prelude::*,
        InvocationArg
    };
    pub use j4rs_derive::call_from_java;
    pub use paste;
    pub use serde;
}

pub use javawithrust_macro::*;
