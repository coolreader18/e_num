extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate darling;

use proc_macro::TokenStream;

#[proc_macro_derive(ENum, attributes(e_num))]
pub fn e_num_derive(input: TokenStream) -> TokenStream {
  let ast = syn::parse(input).unwrap();

  impl_e_num(&ast)
}

#[derive(Debug)]
enum Variant {
  /// A single tuple variant with type T where T: ENum
  /// e.g.
  /// ```ignore
  /// A(usize),
  /// ```
  Field(syn::Type),
  /// A variant with nothing special about it
  /// e.g.
  /// ```ignore
  /// A,
  /// ```
  Unit,
  /// A variant with a discriminant
  /// e.g
  /// ```ignore
  /// A = 1,
  /// ```
  Disc(syn::Expr),
}

impl Variant {
  pub fn from_var(var: &syn::Variant) -> (syn::Ident, syn::Ident, Variant) {
    let variant = if let Some((_, disc)) = &var.discriminant {
      Variant::Disc(disc.clone())
    } else {
      match &var.fields {
        syn::Fields::Unnamed(data) => {
          if data.unnamed.len() != 1 {
            panic!("Invalid fields for");
          }
          Variant::Field(data.unnamed.first().unwrap().value().ty.clone())
        }
        syn::Fields::Unit => Variant::Unit,
        syn::Fields::Named(_) => panic!("ENum can't have named fields variant"),
      }
    };
    let const_ident = syn::Ident::new(
      &format!("{}_MASK", var.ident.to_string().to_uppercase()),
      syn::export::Span::call_site(),
    );
    (var.ident.clone(), const_ident, variant)
  }
}

fn round_up(num_to_round: usize) -> usize {
  let mut v = num_to_round;
  v -= 1;
  v |= v >> 1;
  v |= v >> 2;
  v |= v >> 4;
  v |= v >> 8;
  v |= v >> 16;
  v |= v >> 32;
  v += 1;
  v
}

fn impl_e_num(ast: &syn::DeriveInput) -> TokenStream {
  let name = &ast.ident;
  let data = match &ast.data {
    syn::Data::Enum(data) => data,
    _ => panic!("Can't derive struct for ENum"),
  };
  let Attr { start_at, .. } = Attr::from_derive_input(ast)
    .unwrap_or_else(|err| panic!("Error while parsing attributes: {}", err));
  let leading = (round_up(data.variants.len() + start_at) - 1).leading_zeros() as usize;
  let mask_size = 64 - leading;
  let vars = {
    let mut vec = data
      .variants
      .iter()
      .map(Variant::from_var)
      .collect::<Vec<_>>();
    vec.sort_by(|(_, _, a), (_, _, b)| {
      use std::cmp::Ordering;
      match (a, b) {
        (Variant::Disc(_), Variant::Disc(_)) => Ordering::Equal,
        (Variant::Disc(_), _) => Ordering::Greater,
        (_, Variant::Disc(_)) => Ordering::Less,
        _ => Ordering::Equal,
      }
    });
    vec
  };
  let const_names = vars.iter().map(|(_, name, _)| name);
  let const_vals = vars
    .iter()
    .enumerate()
    .map(|(i, (_, _, var))| match var {
      Variant::Disc(expr) => quote! { #expr },
      Variant::Unit | Variant::Field(_) => {
        let num = start_at + i;
        quote! { #num }
      }
    }).collect::<Vec<_>>();
  let const_defs = quote! {
    #(const #const_names: usize = #const_vals;)*
  };
  let checks = vars.iter().map(|(_, const_name, var)| match var {
    Variant::Disc(_) => quote! { num == #const_name },
    Variant::Field(_) | Variant::Unit => {
      quote! { num << #leading >> #leading == #const_name }
    }
  });
  let outputs = vars.iter().map(|(var_name, _, var)| match var {
    Variant::Disc(_) | Variant::Unit => quote! { Some(#name::#var_name) },
    Variant::Field(ty) => {
      quote! { <#ty as ENum>::try_from_num(num >> #mask_size).map(|val| #name::#var_name(val)) }
    }
  });
  let matches = vars.iter().map(|(var_name, _, var)| match var {
    Variant::Field(_) => quote! { #name::#var_name(val) },
    Variant::Disc(_) | Variant::Unit => quote! { #name::#var_name },
  });
  let converts = vars.iter().map(|(_, const_name, var)| match var {
    Variant::Field(ty) => quote! { <#ty as ENum>::to_num(val) << #mask_size | #const_name },
    Variant::Disc(_) | Variant::Unit => quote! { #const_name },
  });
  let gen = quote! {
    impl ENum for #name {
      fn try_from_num(num: usize) -> Option<Self> {
        #const_defs
        #(if #checks {
          #outputs
        } else)* {
          None
        }
      }
      fn from_num(num: usize) -> Self {
        if let Some(val) = Self::try_from_num(num) {
          val
        } else {
          panic!(concat!("Failure to parse number into ", stringify!(#name)));
        }
      }
      fn to_num(&self) -> usize {
        #const_defs
        match self {
          #(#matches => {
            #converts
          }),*
        }
      }
    }

    impl From<usize> for #name {
      fn from(num: usize) -> Self {
        Self::from_num(num)
      }
    }
  };
  gen.into()
}

use darling::{FromDeriveInput, FromVariant};

#[derive(FromVariant)]
struct Var {
  ident: syn::Ident,
  fields: darling::ast::Fields<syn::Type>,
}

#[derive(FromDeriveInput)]
#[darling(attributes(e_num), forward_attrs(allow, doc, cfg))]
struct Attr {
  #[darling(default)]
  pub start_at: usize,
  pub data: darling::ast::Data<Var, ()>,
}
