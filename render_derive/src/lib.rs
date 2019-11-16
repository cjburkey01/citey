#![recursion_limit = "128"]

#[macro_use]
extern crate quote;
extern crate proc_macro;

use proc_macro::TokenStream;
use syn::export::TokenStream2;

#[proc_macro_derive(VertexAttribPointers, attributes(location))]
pub fn vertex_attrib_pointers_derive(input: TokenStream) -> TokenStream {
    generate_impl(syn::parse(input).unwrap()).into()
}

fn generate_impl(ast: syn::DeriveInput) -> TokenStream2 {
    let ident = &ast.ident;
    let generics = &ast.generics;
    let where_clause = &ast.generics.where_clause;

    let fields_pieces = generate_vertex_attrib_pointer_calls(&ast.data);

    let mut fields_enable: Vec<TokenStream2> = Vec::with_capacity(fields_pieces.len());
    let mut fields_disable: Vec<TokenStream2> = Vec::with_capacity(fields_pieces.len());
    let mut fields_vertex_attrib_pointer: Vec<TokenStream2> =
        Vec::with_capacity(fields_pieces.len());

    for field in fields_pieces {
        fields_enable.push(field.0);
        fields_disable.push(field.1);
        fields_vertex_attrib_pointer.push(field.2);
    }

    // Why do I need to explicitly return here?
    return quote! {
        impl #generics crate::render::VertexAttrib for #ident #generics #where_clause {
            fn setup_attrib_pointer(gl: &crate::Gl) {
                let stride = ::std::mem::size_of::<Self>(); // byte offset between consecutive attributes
                let mut offset = 0;

                #(#fields_vertex_attrib_pointer)*
            }

            fn enable_attribs(gl: &crate::Gl) {
                #(#fields_enable)*
            }

            fn disable_attribs(gl: &crate::Gl) {
                #(#fields_disable)*
            }
        }
    };
}

fn generate_vertex_attrib_pointer_calls(
    body: &syn::Data,
) -> Vec<(TokenStream2, TokenStream2, TokenStream2)> {
    match body {
        &syn::Data::Enum(_) => panic!("VertexAttribPointers cannot be implemented for enums"),
        &syn::Data::Union(_) => panic!("VertexAttribPointers cannot be implemented for unions"),
        syn::Data::Struct(struct_data) => match struct_data.fields {
            syn::Fields::Unit => {
                panic!("VertexAttribPointers cannot be implemented for unit structs")
            }
            syn::Fields::Unnamed(_) => {
                panic!("VertexAttribPointers cannot be implemented for tuples")
            }
            syn::Fields::Named(ref fields) => fields
                .named
                .iter()
                .map(generate_struct_field_vertex_attrib_pointer_call)
                .collect(),
        },
    }
}

fn generate_struct_field_vertex_attrib_pointer_call(
    field: &syn::Field,
) -> (TokenStream2, TokenStream2, TokenStream2) {
    let field_name = match field.ident {
        Some(ref i) => format!("{}", i),
        None => String::from(""),
    };

    let location_attr = field
        .attrs
        .iter()
        .filter(|a| get_path_string(&a.path) == "location".to_owned())
        .next()
        .unwrap_or_else(|| {
            panic!(
                "Field {:?} is missing #[location = ?] attribute",
                field_name
            )
        });

    let loc_attr_meta = match location_attr.parse_meta().unwrap() {
        syn::Meta::NameValue(meta_name_value) => meta_name_value,
        _ => panic!("Invalid location attribute"),
    };

    let location_value = match loc_attr_meta.lit {
        syn::Lit::Int(value) => value.base10_parse::<u32>().unwrap(),
        _ => panic!("Invalid location attribute value"),
    };

    let field_type = &field.ty;

    (
        quote! {
            unsafe { gl.EnableVertexAttribArray(#location_value) };
        },
        quote! {
            unsafe { gl.DisableVertexAttribArray(#location_value) };
        },
        quote! {
            #field_type::attrib_pointer(gl, #location_value, stride, offset as i32);

            offset += ::std::mem::size_of::<#field_type>();
        },
    )
}

fn get_path_string(path: &syn::Path) -> String {
    let mut string = String::new();

    if path.leading_colon.is_some() {
        string.push_str("::");
    }
    for segment in &path.segments {
        string.push_str(&segment.ident.to_string());
        string.push_str("::");
    }
    string.pop();
    string.pop();

    string
}
