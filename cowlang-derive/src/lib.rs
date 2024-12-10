extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ImplItem, ItemImpl};

#[proc_macro_attribute]
pub fn cow_module(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as ItemImpl);

    let name = if let syn::Type::Path(p) = *ast.self_ty {
        p.path.get_ident().unwrap().clone()
    } else {
        panic!("self_ty is not a path!");
    };

    if let Some((_, ref path, _)) = ast.trait_ {
        syn::Error::new_spanned(
            path,
            "#[cow_module] can not be used with a trait impl block",
        )
        .to_compile_error();
    } else if ast.generics != Default::default() {
        syn::Error::new_spanned(
            ast.generics.clone(),
            "#[cow_module] can not be used with lifetime parameters or generics",
        )
        .to_compile_error();
    }

    let mut method_names = Vec::new();
    let mut method_outputs = Vec::new();
    let mut internal_method_names = Vec::new();
    let mut method_return_conversions = Vec::new();
    let mut method_args = Vec::new();
    let mut method_attrs = Vec::new();
    let mut method_structs = Vec::new();
    let mut method_blocks = Vec::new();

    let mut constant_names = Vec::new();
    let mut constant_literals = Vec::new();
    let mut constant_expressions = Vec::new();

    for item in ast.items.iter() {
        match item {
            ImplItem::Fn(meth) => {
                let ident = &meth.sig.ident;
                let mut args = Vec::new();

                let mut returns_object = false;
                let mut attrs_out = Vec::new();

                for attr in meth.attrs.iter() {
                    if attr.path().segments.len() == 1
                        && &attr.path().segments[0].ident == "returns_object"
                    {
                        returns_object = true;
                    } else {
                        attrs_out.push(attr.clone());
                    }
                }

                method_names.push(ident.to_string());
                internal_method_names.push(format_ident!("_internal_{}", ident));

                method_attrs.push(attrs_out);
                method_blocks.push(meth.block.clone());
                method_outputs.push(meth.sig.output.clone());

                let return_conversion = if meth.sig.output == syn::ReturnType::Default {
                    quote! {
                        cowlang::interpreter::Handle::wrap_value( cowlang::Value::None )
                    }
                } else if returns_object {
                    quote! {
                        cowlang::interpreter::Handle::Object( std::rc::Rc::new( result ) )
                    }
                } else {
                    quote! {
                        cowlang::interpreter::Handle::wrap_value( result.into() )
                    }
                };

                method_return_conversions.push(return_conversion);

                for arg in meth.sig.inputs.iter() {
                    // ignore self values etc
                    if let syn::FnArg::Typed(typed) = arg {
                        if let syn::Pat::Ident(syn::PatIdent { ident, .. }) = &*typed.pat {
                            args.push(ident.clone());
                        } else {
                            panic!("Unsupported pattern");
                        }
                    }
                }

                method_args.push(args);

                method_structs.push(format_ident!("MethodCall_{}_{}", name, ident));
            }
            ImplItem::Const(constant) => {
                if let syn::Expr::Lit(lit) = &constant.expr {
                    constant_names.push(constant.ident.to_string());
                    constant_literals.push(lit.lit.clone());
                } else {
                    panic!("Unsupported expression: {:?}", constant.expr);
                }

                constant_expressions.push(constant.clone());
            }
            _ => {
                panic!("ImplItem type not supported: {:?}", item);
            }
        }
    }

    let method_struct_defs = method_structs.iter();
    let method_impl_names = method_structs.iter();
    let method_struct_defs2 = method_structs.iter();

    let mut arg_strings1 = Vec::new();
    let mut arg_strings2 = Vec::new();

    let name_string = name.to_string();
    let mut arg_conversions = Vec::new();
    let mut arg_lens = Vec::new();

    let name_iter = std::iter::repeat(name.clone());
    let name_iter2 = std::iter::repeat(name.clone());

    for args in method_args {
        let conv = quote! {
            let mut _internal_argv = _internal_args.drain(..);

            #(
            let #args = _internal_argv.next().unwrap();
            )*
        };

        let arg_str1 = quote! {
            ( #( #args),*)
        };

        let arg_str2 = quote! {
            (&self, #( #args: cowlang::Value),*)
        };

        arg_strings1.push(arg_str1);
        arg_strings2.push(arg_str2);

        arg_lens.push(args.len());
        arg_conversions.push(conv);
    }

    let expanded = quote! {
        #(

        #( #method_attrs )*
        #[allow(non_camel_case_types)]
        struct #method_struct_defs {
            self_ref: std::rc::Rc<dyn cowlang::Module>
        }

        #( #method_attrs )*
        impl cowlang::interpreter::Callable for #method_impl_names {
            fn call(&self, mut _internal_args: Vec<cowlang::Value>) -> cowlang::interpreter::Handle {
                //FIXME find a way to do this without raw pointers

                let self_rc = self.self_ref.clone();

                let self_ptr = std::rc::Rc::into_raw(self_rc);
                let self_ref = unsafe{ &*(self_ptr as *const #name_iter) };

                if _internal_args.len() != #arg_lens {
                    panic!("Invalid number of arguments!");
                }

                #arg_conversions

                let result = self_ref.#internal_method_names #arg_strings1;

                // Avoid memory leak
                unsafe {
                    std::rc::Rc::from_raw(self_ptr);
                };

                #method_return_conversions
            }
        }
        )*

        impl #name {
            #( #constant_expressions )*
        }

        #(
        impl #name_iter2 {

            #[inline]
            #( #method_attrs )*
            fn #internal_method_names #arg_strings2 #method_outputs #method_blocks
        }
        )*

        impl cowlang::Module for #name {
            fn get_member(&self, self_ref: &std::rc::Rc<dyn cowlang::Module>, member_name: &str) -> cowlang::interpreter::Handle {
                #(
                if member_name == #method_names {

                    #( #method_attrs )*
                    {
                        return cowlang::interpreter::Handle::Callable( Box::new(
                                #method_struct_defs2{ self_ref: self_ref.clone() }
                        ));
                    }
                }
                )*
                #(
                if member_name == #constant_names {
                    return cowlang::interpreter::Handle::wrap_value( #constant_literals.into() );
                }
                )*

                panic!("No such member {}::{}", #name_string, member_name);
            }
        }
    };

    TokenStream::from(expanded)
}
