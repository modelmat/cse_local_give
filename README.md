# cse_local_give

To install the program, run `cargo install --path .`, which adds it into
`~/.cargo/bin`, which you should add to `$PATH`.

To run the program, first add your zID (zXXXXXXX) into the file
`~/.config/cse_local_give/zID` (strictly speaking, `$XDG_CONFIG_DIR`).

Then, you may use the program as:

```bash
$ cse_local_give class_code assignment filename [filenames...]
```

## Possible Improvements

- Proper helptext (e.g. `cse_local_give --help`)
- Completions (e.g. to help select file, or for choosing an assignment). 
- Requesting & saving zID from command line, rather than erroring out.

