#![feature(proc_macro_expand)]


use proc_macro::*;

use quote::{quote, ToTokens};
use syn::{
    {
        Token, Ident, TypePath, Stmt, Block, Visibility,
        braced,
        parenthesized
    },
    Result, Type
};
use syn::{
    DeriveInput,
    parse_macro_input,
    parse::{Parse, ParseStream}
};


/// A list of parsed functions.
struct JavaFunctions {
    /// The name of the struct that represents a Java class.
    struct_name : Ident,
    /// The list of parsed functions.
    funcs       : Vec<JavaFunction>
}
impl Parse for JavaFunctions {
    fn parse(input : ParseStream) -> Result<Self> {
        input.parse::<Token![impl]>()?;

        // Get the name of the struct.
        let struct_name = input.parse::<Ident>()?;

        // Parse all of the functions.
        let funcs_content;
        braced!(funcs_content in input);
        let mut funcs = Vec::new();
        loop {
            // Get the visibility of the function.
            let vis = if funcs_content.lookahead1().peek(Token![fn]) {
                None
            } else if funcs_content.lookahead1().peek(Token![pub]) {
                Some(funcs_content.parse::<Visibility>()?)
            } else {
                break;
            };

            funcs_content.parse::<Token![fn]>()?;

            // Get the name of the function.
            let name = funcs_content.parse::<Ident>()?;

            // Get the function arguments.
            let args_content;
            parenthesized!(args_content in funcs_content);
            let mut arg_names = Vec::new();
            let mut arg_types = Vec::new();
            let mut args      = quote::__private::TokenStream::new();
            loop {

                // Get the mutability of the argument.
                if args_content.lookahead1().peek(Token![mut]) {
                    extend(&mut args, &args_content.parse::<Token![mut]>()?)
                } else {
                    if ! args_content.lookahead1().peek(Ident) {
                        break;
                    }
                }

                // Get the name of the argument.
                let ident = args_content.parse::<Ident>()?;
                extend(&mut args, &ident);

                arg_names.push(ident);
                let colon = args_content.parse::<Token![:]>()?;
                extend(&mut args, &colon);

                // Get the type of the argument.
                let typ = args_content.parse::<Type>()?;
                extend(&mut args, &typ);
                arg_types.push(typ);

                // Check if there are more arguments.
                if args_content.lookahead1().peek(Token![,]) {
                    extend(&mut args, &args_content.parse::<Token![,]>()?);
                } else {
                    break;
                }
            }
    
            // Get the return type.
            let ret = if funcs_content.lookahead1().peek(Token![->]) {
                funcs_content.parse::<Token![->]>()?;
                Some(funcs_content.parse::<TypePath>()?)
            } else {
                None
            };
    
            // Get the function body if there is one.
            let block = if funcs_content.lookahead1().peek(Token![;]) {
                funcs_content.parse::<Token![;]>()?;
                None
            } else {
                let block_content;
                braced!(block_content in funcs_content);
                Some(block_content.call(Block::parse_within)?)
            };

            // Create a new `JavaFunction` instance and return.
            funcs.push(JavaFunction {
                vis,
                name,
                arg_names,
                arg_types,
                args,
                ret,
                block
            });
        }

        // Create a new `JavaFunctions` instance and return.
        return Ok(JavaFunctions {
            struct_name,
            funcs
        }); 
    }
}

/// A parsed functions.
struct JavaFunction {
    /// The visibility of the function.
    vis       : Option<Visibility>,
    /// The name of the function.
    name      : Ident,
    /// The argument names of the function.
    arg_names : Vec<Ident>,
    /// The argument types of the function.
    arg_types : Vec<Type>,
    /// The argument tokens.
    args      : quote::__private::TokenStream,
    /// The return type.
    ret       : Option<TypePath>,
    /// The body.
    block     : Option<Vec<Stmt>>
}
impl JavaFunction {

    /// Generate the java-to-rust function.
    pub fn generate_bridge(&self, struct_name : &Ident, tokens : &mut quote::__private::TokenStream) {
        // If it is a rust-to-java function, cancel.
        if let None = self.block {
            return;
        }
        let name              = &self.name;
        let name_str          = name.to_string();
        let arg_names = &self.arg_names;
        let arg_types = &self.arg_types;
        tokens.extend::<quote::__private::TokenStream>(quote!{
            ::javawithrust::prelude::paste::item!{
                #[::javawithrust::call_from_java_expand(concat!([<__jwrs_classname_ #struct_name>]!(), ".", #name_str))]
                fn #name(#(#arg_names : ::javawithrust::prelude::Instance),*) -> Result<::javawithrust::prelude::Instance, String> {
                    // Get the active JVM.
                    let __jwrs_jvm = ::javawithrust::prelude::Jvm::attach_thread().unwrap();
                    // Convert all of arguments from Java objects to Rust structs.
                    #(let #arg_names =
                        __jwrs_jvm.to_rust::<#arg_types>(#arg_names)
                            .map_err(|error| format!("{}", error))?;
                    )*
                    // Call the frontend function and convert the result from a Rust struct to a Java object.
                    return ::javawithrust::prelude::Instance::try_from(
                        ::javawithrust::prelude::InvocationArg::try_from(
                            #struct_name :: #name(#(#arg_names),*)?
                        )
                            .map_err(|error| format!("{}", error))?
                    )
                        .map_err(|error| format!("{}", error));
                }
            }
        });

    }


    /// Generate the callable function.
    fn generate_frontend(&self, struct_name : &Ident, tokens : &mut quote::__private::TokenStream) {

        // Add the function information.
        if let Some(vis) = &self.vis {
            extend(tokens, vis);
        }
        let name = &self.name;
        let args = &self.args;
        tokens.extend::<quote::__private::TokenStream>(quote!{
            fn #name(#args)
        }.into());
        tokens.extend::<quote::__private::TokenStream>(
            if let Some(ret) = &self.ret {
                quote!{
                    -> Result<#ret, String>
                }.into()
            } else {
                quote!{
                    -> Result<(), String>
                }.into()
            }
        );
        let name_str  = name.to_string();
        let arg_names = &self.arg_names;
        let ret       = if let Some(ret) = &self.ret {
            quote!{#ret}
        } else {
            quote!{()}
        };
        tokens.extend::<quote::__private::TokenStream>(match &self.block {
            Some(block) => {
                // If it is a java-to-rust function, add the body.
                quote!{
                    {#(#block)*}
                }
            },
            None => {quote!{
                // If it is a rust-to-java function:
                {
                    // Get the active JVM.
                    let __jwrs_jvm = ::javawithrust::prelude::Jvm::attach_thread().unwrap();
                    // Convert all of arguments from Rust structs to Java objects.
                    #(let #arg_names =
                        ::javawithrust::prelude::Instance::try_from(
                            ::javawithrust::prelude::InvocationArg::try_from(
                                #arg_names
                            )
                                .map_err(|error| format!("{}", error))?
                        )
                        .map_err(|error| format!("{}", error));
                    )*
                    // Call the java function and convert the result from a Java object to a Rust struct.
                    ::javawithrust::prelude::paste::item!{
                        return __jwrs_jvm.to_rust::<#ret>(
                            __jwrs_jvm.invoke_static([<__jwrs_classname_ #struct_name>]!(), #name_str, &[#(#arg_names),*])
                                .map_err(|error| format!("{}", error))?
                        )
                            .map_err(|error| format!("{}", error));
                    }
                }
            }}
        }.into());

    }

}


/// Extends a `quote::__private::TokenStream` with a `ToTokens` object.
fn extend<T : ToTokens>(tokens : &mut quote::__private::TokenStream, value : &T) {
    let mut token = quote::__private::TokenStream::new();
    value.to_tokens(&mut token);
    tokens.extend(token);
}


/// Define a Java class.
/// 
/// # Examples
/// 
/// ```
/// // Rust
/// 
/// use javawithrust::prelude::*;
/// 
/// #[jclass(io.example.MyJavaClass)]
/// struct MyJavaClass;
/// ```
/// 
/// # Warning
/// 
/// Class name can not contain any special
/// characters or underscores except for
/// the `.` separator.
#[proc_macro_attribute]
pub fn jclass(attr : TokenStream, item : TokenStream) -> TokenStream {
    let parse_item = item.clone();
    let DeriveInput {
        attrs    : _struct_attrs,
        vis      : _struct_vis,
        ident    : struct_name,
        generics : _struct_generics,
        data     : _struct_data
    } = parse_macro_input!(parse_item);
    let     class_name = attr.to_string();
    let mut item2 = quote!{
        #[derive(::javawithrust::prelude::serde::Serialize)]
        #[derive(::javawithrust::prelude::serde::Deserialize)]
    };
    item2.extend::<quote::__private::TokenStream>(item.into());
    item2.extend::<quote::__private::TokenStream>(quote!{
        ::javawithrust::prelude::paste::item!{
            macro_rules! [<__jwrs_classname_ #struct_name>] {
                () => {#class_name}
            }
            use [<__jwrs_classname_ #struct_name>];
        }
        impl TryFrom<#struct_name> for ::javawithrust::prelude::InvocationArg {
            type Error = ::javawithrust::prelude::errors::J4RsError;
            fn try_from(arg : #struct_name) -> ::javawithrust::prelude::errors::Result<::javawithrust::prelude::InvocationArg> {
                return Ok(::javawithrust::prelude::InvocationArg::new(&arg, #class_name));
            }
        }

    }.into());
    return item2.into();
}


/// Define static Java class functions to bind.
/// 
/// # Examples
/// 
/// ```
/// // Rust
/// 
/// use javawithrust::prelude::*;
/// 
/// #[jclass(io.example.MyJavaClass)]
/// struct MyJavaClass;
/// 
/// #[jfuncs]
/// impl J2RS {
///     // If no body is provided, this will be a rust-to-java function.
///     fn hello();
///     // If a body is provided, this will be a java-to-rust function.
///     // However, this can still be called in Rust using `ClassName::funcName`.
///     fn sum(a : i32, b : i32) -> /*Result<*/i32/*, String>*/ {
///         return Ok(a + b);
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn jfuncs(attr : TokenStream, item : TokenStream) -> TokenStream {
    if ! attr.is_empty() {
        panic!("`jfuncs` attribute takes 0 parameters.")
    }
    let parse_item = item.clone();
    let JavaFunctions {
        struct_name,
        funcs,
        ..
    } = parse_macro_input!(parse_item as JavaFunctions);

    let mut bridge_funcs   = quote::__private::TokenStream::new();
    let mut frontend_funcs = quote::__private::TokenStream::new();
    for func in funcs {
        let mut bridge_func   = quote::__private::TokenStream::new();
        let mut frontend_func = quote::__private::TokenStream::new();
        func.generate_bridge(&struct_name, &mut bridge_func);
        func.generate_frontend(&struct_name, &mut frontend_func);
        bridge_funcs.extend(bridge_func);
        frontend_funcs.extend(frontend_func);
    }

    return quote!{
        ::javawithrust::prelude::paste::item!{
            mod [<__jwrs_ #struct_name>] {
                use super::*;
                #bridge_funcs
            }
        }
        impl #struct_name {
            #frontend_funcs
        }
    }.into();
}


/// INTERNAL
/// 
/// Used for attaching the `j4rs_derive::call_from_java`
/// attribute macro, after expanding contained
/// macros.
#[proc_macro_attribute]
pub fn call_from_java_expand(mut attr : TokenStream, item : TokenStream) -> TokenStream {
    attr = attr.expand_expr().unwrap();
    let mut attr2 = quote::__private::TokenStream::new();
    attr2.extend::<quote::__private::TokenStream>(attr.into());
    let mut tokens = TokenStream::new();
    tokens.extend::<TokenStream>(quote!{
        #[::javawithrust::prelude::call_from_java(#attr2)]
    }.into());
    tokens.extend(item);
    return tokens;
}
