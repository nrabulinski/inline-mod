#![cfg_attr(feature = "docs",
   cfg_attr(all(), doc = include_str!("../README.md")),
)]
use std::{
	fs::read_to_string,
	path::{Path, PathBuf},
};

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{
	parse2, parse_file, spanned::Spanned, Error, File, Item, ItemMod, Lit, LitStr, Meta,
	MetaNameValue, Result,
};

#[proc_macro]
pub fn inline_mod(input: TokenStream) -> TokenStream {
	inline_mod_impl(input.into(), None)
		.unwrap_or_else(|err| err.to_compile_error())
		.into()
}

fn inline_mod_impl(input: TokenStream2, default_path: Option<LitStr>) -> Result<TokenStream2> {
	let input: ItemMod = parse2(input)?;
	if input.content.is_some() {
		return Ok(quote! {
			::core::compile_error!("This macro only accepts non-inlined modules")
		});
	}
	let path = input
		.attrs
		.iter()
		.find_map(|attr| match attr.parse_meta() {
			Ok(Meta::NameValue(MetaNameValue {
				path,
				lit: Lit::Str(lit),
				..
			})) if path.is_ident("path") => Some(lit),
			_ => None,
		})
		.or(default_path)
		.ok_or_else(|| Error::new(Span::call_site(), "Path attribute is required"))?;
	let (path, path_span) = (path.value(), path.span());
	let mut path = PathBuf::from(path);
	if path.is_relative() {
		path = Path::new(
			&std::env::var_os("CARGO_MANIFEST_DIR").expect("Missing `CARGO_MANIFEST_DIR` variable"),
		)
		.join(path);
	}
	let path_str = path.to_str().unwrap();
	let root = path_str.strip_suffix("/mod.rs").or_else(|| path_str.strip_suffix(".rs")).unwrap_or(path_str);
	let root = Path::new(root);
	let ItemMod {
		ident, vis, attrs, ..
	} = input;

	let File {
		attrs: file_attrs,
		items,
		..
	} = {
		let content = read_to_string(&path).map_err(|err| {
			Error::new(
				path_span,
				format!(
					"Error reading module `{}` (path = `{:?}`): {}",
					&ident, path, err
				),
			)
		})?;
		parse_file(&content)?
	};

	let items = items.into_iter().map(|item| match item {
		Item::Mod(module) if module.content.is_none() => {
			let mut mod_path = root.join(format!("{}.rs", module.ident));
			if !mod_path.is_file() {
				mod_path = root.join(format!("{}/mod.rs", module.ident));
			}
			let mod_path = LitStr::new(mod_path.to_str().unwrap(), module.span());
			inline_mod_impl(module.into_token_stream(), Some(mod_path))
				.unwrap_or_else(|err| err.to_compile_error())
		}
		_ => item.into_token_stream(),
	});

	Ok(quote! {
		const _: &[u8] = ::core::include_bytes!( #path_str ).as_slice();
		#( #attrs )*
		#vis mod #ident {
			#( #file_attrs )*
			#( #items )*
		}
	})
}
