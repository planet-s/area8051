ASM=$(wildcard *.a51)
C=$(wildcard *.c)
IHX=\
	$(patsubst %.a51,%.tmp/8051.ihx,$(ASM)) \
	$(patsubst %.c,%.tmp/8051.ihx,$(C))
BIN=\
	$(patsubst %.ihx,%.bin,$(IHX))

.PHONY: all clean test

all: $(IHX) $(BIN)

clean:
	rm -rf *.tmp

test: $(BIN)
	for bin in $^; do \
		dir="$$(dirname "$$bin")" && \
		echo "$${dir%.tmp}" && \
		RUST_BACKTRACE=1 cargo run \
		 	--quiet \
			--manifest-path ../Cargo.toml \
			--no-default-features \
			-- $$bin | \
		tee "$${bin%.bin}.stdout" ; \
	done

%.tmp/8051.ihx: %.a51
	rm -rf $*.tmp
	mkdir -p $*.tmp
	cd $*.tmp && \
	as31 -O8051.ihx ../$<

%.tmp/8051.ihx: %.c
	rm -rf $*.tmp
	mkdir -p $*.tmp
	cd $*.tmp && \
	sdcc -mmcs51 -o 8051.ihx ../$<

%.tmp/8051.bin: %.tmp/8051.ihx
	makebin -p < $< > $@
	d52 -t $@
	expand $*.tmp/8051.d52 > $*.tmp/8051.d52.expanded
	mv $*.tmp/8051.d52.expanded $*.tmp/8051.d52
