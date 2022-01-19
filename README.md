# inline-mod
Inline modules at macro expansion time!

## Note
Initial declaration *requires* a `path` attribute *relative to the project root* because on stable it is impossible to get the path of the file where the macro is invoked.  
Every nested module with a path attribute will also be treated relative to the project root, although **that might change in a future update**.  
If a path attribute is not present on a nested non-inlined module,
the macro will try to resolve it the same way the Rust compiler does - look for `module_name.rs` or `module_name/mod.rs` file relative to the current module.

## Why?
Currently, even if you are on nighly and can use an attribute macro on a non-inlined module, you will not accomplish anything because the items get inlined after macro expansion.
This macro inlines the module for you, so you can use macros on non-inlined (even nested) modules.

## Example

```rust
// main.rs
use inline_mod::inline_mod;

inline_mod! {
	#[my_attr]
	#[path = "src/foo.rs"]
	mod foo;
}

// foo.rs
struct Bar(i32);

pub mod baz;

// foo/baz/mod.rs
pub struct Hi;
```

Will get expanded to

```rust
#[my_attr]
mod foo {
	struct Bar(i32);

	pub mod baz {
		pub struct Hi;
	}
}
```

All before `my_attr` gets executed.
