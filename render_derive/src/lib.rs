#![recursion_limit = "128"]

#[macro_use]
extern crate quote;
extern crate proc_macro;

use proc_macro::TokenStream;
use syn::export::TokenStream2;
use syn::DeriveInput;

#[proc_macro_derive(VertexAttribPointers, attributes(location))]
pub fn vertex_attrib_pointers_derive(input: TokenStream) -> TokenStream {
    generate_impl(syn::parse_macro_input!(input as DeriveInput)).into()
}

fn generate_impl(ast: syn::DeriveInput) -> TokenStream2 {
    // Generate all the calls to be inserted in the implementation functions
    let fields_pieces = generate_vertex_calls(&ast.data);

    // Separate the vector of tuples into a tuple of vectors
    let len = fields_pieces.len();
    let (fields_enable, fields_disable, fields_vertex_attrib_pointer) =
        fields_pieces.into_iter().fold(
            (
                Vec::with_capacity(len),
                Vec::with_capacity(len),
                Vec::with_capacity(len),
            ),
            |mut acc, val| {
                acc.0.push(val.0);
                acc.1.push(val.1);
                acc.2.push(val.2);
                acc
            },
        );

    // Get struct info
    let ident = &ast.ident;
    let generics = &ast.generics;
    let where_clause = &ast.generics.where_clause;
    quote! {
        // Implement this vertex attrib type for this struct
        impl #generics ::render::VertexAttrib for #ident #generics #where_clause {
            fn setup_attrib_pointer(gl: &::gl_bindings::Gl) {
                // The byte size of each vertex
                let stride = ::std::mem::size_of::<Self>();

                // The current offset (in bytes) from the start of the buffer
                // for the given vertex attribute pointer (this is incremented
                // by the size of each component)
                let mut offset = 0;

                // Call the vertex attribute pointer for each attribute
                #(#fields_vertex_attrib_pointer)*
            }

            fn enable_attribs(gl: &::gl_bindings::Gl) {
                // Enable all of the attribute locations
                unsafe {
                    #(#fields_enable)*
                }
            }

            fn disable_attribs(gl: &::gl_bindings::Gl) {
                // Disable all of the attribute locations
                unsafe {
                    #(#fields_disable)*
                }
            }
        }
    }
}

fn generate_vertex_calls(body: &syn::Data) -> Vec<(TokenStream2, TokenStream2, TokenStream2)> {
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
            // Only structs with named fields are applicable to become vertices
            // Collect all of the fields with their associated data and the
            // code to generate for each
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
    // Get the name of this field within the struct
    let field_name = match field.ident {
        Some(ref i) => format!("{}", i),
        None => panic!("Field missing name"),
    };

    // Get the attribute responsible for reporting the vertex attribute
    // location of this field
    let location_attr = field
        .attrs
        .iter()
        // Look for the attribute called "location"
        .filter(|a| get_path_string(&a.path) == "location".to_owned())
        // Get the "first" one (the only one)
        .next()
        .unwrap_or_else(|| {
            panic!(
                "Field {:?} is missing #[location = ?] attribute",
                field_name
            )
        });

    // Get the information for this attribute
    let loc_attr_meta = match location_attr.parse_meta().unwrap() {
        syn::Meta::NameValue(meta_name_value) => meta_name_value,
        _ => panic!("Invalid named attribute"),
    };

    // Get the raw unsigned integer value of this attribute (the real location)
    let location_value = match loc_attr_meta.lit {
        syn::Lit::Int(value) => value.base10_parse::<u32>().unwrap(),
        _ => panic!("Invalid location attribute value"),
    };

    // Cache the field location so it can be used within the quote macro
    let field_type = &field.ty;

    // Return a tuple
    (
        // Enable all the vertex attrib locations
        quote! {
            unsafe { gl.EnableVertexAttribArray(#location_value) };
        },
        // Disable all the vertex attrib locations
        quote! {
            unsafe { gl.DisableVertexAttribArray(#location_value) };
        },
        // Create the vertex attrib pointer
        quote! {
            <#field_type as ::render::VertComponent>::attrib_pointer(gl, #location_value, stride, offset as i32);

            // Increment the offset
            offset += ::std::mem::size_of::<#field_type>();
        },
    )
}

fn get_path_string(path: &syn::Path) -> String {
    let mut string = String::new();

    // Push the leading color
    if path.leading_colon.is_some() {
        string.push_str("::");
    }

    // Append each segment with a double colon separator
    for segment in &path.segments {
        string.push_str(&segment.ident.to_string());
        string.push_str("::");
    }

    // Remove the final two (excess) colons
    string.pop();
    string.pop();

    // Return the string
    string
}
