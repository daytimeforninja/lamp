import Lake
open Lake DSL

package lampSync where
  leanOptions := #[⟨`autoImplicit, false⟩]

@[default_target]
lean_lib LampSync where
  srcDir := "."
