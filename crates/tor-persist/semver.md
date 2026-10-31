BREAKING: Remove the standalone `fslock-guard` package. Its `LockFileGuard`
now belongs to `tor-persist` and is re-exported at the crate root. The existing
`state_dir::LockFileGuard` re-export remains. The guard's `lock`, `try_lock`,
`delete_lock_file`, `Debug` and `Drop` behavior and signatures are unchanged.
The three platform-gated, formerly crate-visible `lockfile_has_path` helpers
are now visible only to the private file-lock module.
