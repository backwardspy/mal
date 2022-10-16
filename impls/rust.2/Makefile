SOURCES = src/lib.rs src/parser.rs src/printer.rs src/reader.rs src/types.rs
STEPS = step0_repl step1_read_print

$(STEPS): %: src/bin/%.rs $(SOURCES)
	cargo build --release --bin $@
	cp target/release/$@ .
