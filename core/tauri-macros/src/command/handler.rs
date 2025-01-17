// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use syn::{
  parse::{Parse, ParseBuffer},
  Ident, Path, Token,
};

/// The items parsed from [`generate_handle!`](crate::generate_handle).
pub struct Handler {
  paths: Vec<Path>,
  commands: Vec<Ident>,
  wrappers: Vec<Path>,
}

impl Parse for Handler {
  fn parse(input: &ParseBuffer) -> syn::Result<Self> {
    let paths = input.parse_terminated::<Path, Token![,]>(Path::parse)?;

    // parse the command names and wrappers from the passed paths
    let (commands, wrappers) = paths
      .iter()
      .map(|path| {
        let mut wrapper = path.clone();
        let last = super::path_to_command(&mut wrapper);

        // the name of the actual command function
        let command = last.ident.clone();

        // set the path to the command function wrapper
        last.ident = super::format_command_wrapper(&command);

        (command, wrapper)
      })
      .unzip();

    Ok(Self {
      paths: paths.into_iter().collect(), // remove punctuation separators
      commands,
      wrappers,
    })
  }
}

impl From<Handler> for proc_macro::TokenStream {
  fn from(
    Handler {
      paths,
      commands,
      wrappers,
    }: Handler,
  ) -> Self {
    quote::quote!(move |__tauri_invoke__| {
      let __tauri_cmd__ = __tauri_invoke__.message.command();
      match __tauri_cmd__ {
        #(stringify!(#commands) => #wrappers!(#paths, __tauri_invoke__),)*
        _ => {
          __tauri_invoke__.resolver.reject(format!("command {} not found", __tauri_cmd__))
        },
      }
    })
    .into()
  }
}
